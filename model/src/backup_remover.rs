/*
 * Copyright (c) 2025 Yasuaki Gohko
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE ABOVE LISTED COPYRIGHT HOLDER(S) BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

use camino::Utf8PathBuf;
use parking_lot::RwLock;
use std::fs;
use std::fs::DirEntry;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::commons::Notification;
use crate::commons::OperatePath;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::task;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "backup_remover";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_DIRECTORY_FAILED: ErrorCode = 1;

struct Private {
    join_handle: Option<JoinHandle<Result<()>>>,
    destination_path: String,
    removed_count: i64,
    count: i32,
    notification_sender: Option<Sender<Notification>>,
    termination_requested: bool,
}

impl Private {
    fn new() -> Self {
        Self {
            join_handle: None,
            destination_path: ".".to_string(),
            removed_count: 0,
            count: 0,
            notification_sender: None,
            termination_requested: false,
        }
    }
}

#[derive(Clone)]
pub struct BackupRemover {
    private: Arc<RwLock<Private>>,
}

impl Task for BackupRemover {
    /// Removes the backups marked as removed in the calling thread.
    ///
    /// Scans `<destination_path>/Backups` and, for each `.removed` directory,
    /// removes its files one by one and then the directory itself. The other
    /// entries are left as they are. Failing to remove a file or a directory is
    /// printed and the removal continues. If `terminate()` is called, the removal
    /// stops as described there.
    ///
    /// This blocks until the removal finishes. Use `execute_in_background()` to
    /// run it in another thread.
    ///
    /// # Errors
    ///
    /// * `Error::new(ERROR_ID, ERROR_CODE_READING_DIRECTORY_FAILED)`: If the
    ///   `Backups` directory cannot be read.
    /// * An error of `FilePathProducer` other than finishing is returned as it is.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the removal fails.
    fn execute(&mut self) -> Result<()> {
        let result = self.main();

        result
    }

    /// Spawns a new thread to execute a background task.
    ///
    /// This method clones the internal `private` state and moves it into a new
    /// thread. The `main` function is then called within this new thread using
    /// the cloned `private` state.
    ///
    /// A `JoinHandle` for the newly created thread is stored in `self.join_handle`,
    /// allowing the caller to later wait for the completion of the background
    /// task using the `join` method.
    ///
    /// This method returns `Ok(())` immediately after successfully spawning the
    /// thread, without waiting for the background task to complete. Any errors
    /// occurring within the background task itself will be captured by the
    /// `JoinHandle` and can be retrieved when `join` is called.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the thread was successfully spawned.
    fn execute_in_background(&mut self) -> Result<()> {
        let remover = self.clone();
        let mut private = self.private.write();
        private.join_handle = Some(thread::spawn(move || {
            let result = remover.main();

            result
        }));

        Ok(())
    }

    /// Spawns a new thread to execute the background task.
    ///
    /// This method clones the `private` state of the current object and moves it into a new thread.
    /// The `main` function is then called within this new thread, using the cloned `private` state.
    /// The `JoinHandle` for this thread is stored internally, allowing the task's completion
    /// and result to be awaited using the `join` method.
    ///
    /// # Returns
    /// - `Ok(())` if the background thread is successfully spawned.
    fn join(&mut self) -> Result<()> {
        let handle: Option<JoinHandle<Result<()>>>;
        {
            let mut private = self.private.write();
            handle = private.join_handle.take();
        }
        let Some(handle) = handle else {
            return Err(Error::new(task::ERROR_ID, task::ERROR_CODE_NOT_SUPPORTED));
        };
        let Ok(result) = handle.join() else {
            return Err(Error::new(task::ERROR_ID, task::ERROR_CODE_PANICKED));
        };

        result
    }
}

impl BackupRemover {
    /// Creates a new instance of `BackupRemover`.
    ///
    /// This method initializes a new instance with a `None` `join_handle`,
    /// indicating that no worker thread has been spawned or joined yet.
    /// It also sets up a new `Arc<RwLock<Private>>` for its internal
    /// private state, ensuring thread-safe shared access to its data.
    pub fn new() -> Self {
        Self {
            private: Arc::new(RwLock::new(Private::new())),
        }
    }

    /// Sets the destination path for the operation managed by this instance.
    ///
    /// This method acquires a write lock on the internal private data,
    /// updates the `destination_path` field, and then releases the lock.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the new destination path.
    ///            This path will be cloned and stored internally.
    pub fn set_destination_path(&self, path: &str) {
        let mut private = self.private.write();
        private.destination_path = path.to_string();
    }

    /// Sets the notification sender.
    ///
    /// # Arguments
    ///
    /// * `sender` - The sender to use sending notifications.
    pub fn set_notification_sender(&mut self, sender: Sender<Notification>) {
        self.private.write().notification_sender = Some(sender);
    }
    
    /// Requests the running removal to terminate.
    ///
    /// This only sets a flag and returns immediately. The removal checks the flag
    /// after each file is removed, and stops removing the remaining files and
    /// backups. The `.removed` directory being processed at that time is still
    /// removed with its remaining contents, and `execute()` or `join()` returns
    /// `Ok(())`. Use `join()` to wait for the termination when the removal is
    /// executed in the background.
    ///
    /// The flag is never reset, so calling this before the execution makes the
    /// removal stop after the first entry in the `Backups` directory is processed.
    pub fn terminate(&self) {
        self.private.write().termination_requested = true;
    }
    
    fn main(&self) -> Result<()> {
        let destination_path: String;
        {
            let private = self.private.read();
            destination_path = private.destination_path.clone();
        }
        let mut path = Utf8PathBuf::from(&destination_path);
        path.push("Backups");

        let Ok(read_dir) = fs::read_dir(&path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_READING_DIRECTORY_FAILED));
        };
        for result in read_dir {
            if let Ok(dir_entry) = result {
                self.process_dir_entry(&dir_entry)?;
            }
            if self.private.read().termination_requested {
                break;
            }
        }

        Ok(())
    }

    fn process_dir_entry(&self, dir_entry: &DirEntry) -> Result<()> {
        let Ok(metadata) = dir_entry.metadata() else {
            return Ok(());
        };
        if !metadata.is_dir() {
            return Ok(());
        }
        let path = dir_entry.path();
        let path = path.to_string_easy();
        if !path.ends_with(".removed") {
            return Ok(());
        }

        let mut producer = FilePathProducer::new(&path);
        let mut done = false;
        while !done {
            let file_path = match producer.next() {
                Ok(file_path) => file_path,
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID
                        && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED
                    {
                        done = true;
                    } else {
                        return Err(error);
                    }

                    "".to_string()
                }
            };

            if !done {
                let mut removing_path = Utf8PathBuf::from(&path);
                removing_path.push(&file_path);
                {
                    let private = self.private.read();
                    if private.count == 0 {
                        if let Some(ref sender) = self.private.read().notification_sender {
                            let string = format!("Removing ({}): {}", private.removed_count, removing_path);
                            let _ = sender.send(Notification::Progress(string));
                        }
                    }
                }
                if let Err(error) = fs::remove_file(&removing_path) {
                    if let Some(ref sender) = self.private.read().notification_sender {
                        let string = format!("Removing file {} failed. error: {}", removing_path, error);
                        let _ = sender.send(Notification::Error(string));
                    }
                }
                {
                    let mut private = self.private.write();
                    private.removed_count += 1;
                    private.count += 1;
                    private.count %= 1000;
                }
            }
            if self.private.read().termination_requested {
                done = true;
            }
        }

        if let Err(error) = fs::remove_dir_all(&path) {
            if let Some(ref sender) = self.private.read().notification_sender {
                let string = format!("Removing directory {} failed. error: {}", path, error);
                let _ = sender.send(Notification::Error(string));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env::consts;
    use std::fs;
    use std::path::Path;
    use tempdir::TempDir;

    use crate::backup_executer::BackupExecuter;
    use crate::backup_remover::BackupRemover;
    use crate::commons::OperatePath;
    use crate::destination::Destination;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _remover = BackupRemover::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = &temp_dir.path().to_path_buf();
        let mut source_path = temp_path.clone();
        source_path.push("source");
        fs::create_dir_all(&source_path).unwrap();
        let mut file_path = source_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "ABCDE").unwrap();

        let mut ntm_path = temp_path.clone();
        ntm_path.push("ntm");
        fs::create_dir_all(&ntm_path).unwrap();
        Destination::init(&ntm_path.to_string_easy()).unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let mut config = format!("source_path = \"{}\"", source_path.display());
        if consts::OS == "windows" {
            config = config.replace("\\", "\\\\");
        }
        fs::write(config_path, config).unwrap();

        let mut executer = BackupExecuter::new(&ntm_path.to_string_easy());
        executer.execute().unwrap();

        let mut from_path = ntm_path.clone();
        from_path.push("Backups");
        from_path.push(&executer.name());
        let mut to_path = ntm_path.clone();
        to_path.push("Backups");
        let mut file_name = executer.name().clone();
        file_name.push_str(".removed");
        to_path.push(&file_name);
        fs::rename(&from_path, &to_path).unwrap();
        let mut remover = BackupRemover::new();
        remover.set_destination_path(&ntm_path.to_string_easy());
        remover.execute().unwrap();
    }

    const FILE_COUNT: usize = 10;

    /// Creates `Backups/<name>` containing `FILE_COUNT` files for each name.
    fn create_backups(destination_path: &Path, names: &[String]) {
        for name in names {
            let backup_path = destination_path.join("Backups").join(name).join("dir");
            fs::create_dir_all(&backup_path).unwrap();
            for i in 0..FILE_COUNT {
                fs::write(backup_path.join(format!("{}.txt", i)), "entry").unwrap();
            }
        }
    }

    fn removed_names(count: usize) -> Vec<String> {
        (0..count).map(|i| format!("2026110{}-0000.removed", i)).collect()
    }

    fn existing_names(destination_path: &Path, names: &[String]) -> Vec<String> {
        names
            .iter()
            .filter(|name| destination_path.join("Backups").join(name).exists())
            .cloned()
            .collect()
    }

    fn file_count(backup_path: &Path) -> usize {
        fs::read_dir(backup_path.join("dir")).unwrap().count()
    }

    #[test]
    fn termination_before_execution_stops_after_first_backup() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("ntm");
        let names = removed_names(2);
        create_backups(&destination_path, &names);

        let mut remover = BackupRemover::new();
        remover.set_destination_path(&destination_path.to_string_easy());
        remover.terminate();
        remover.execute().unwrap();

        // The first backup is removed entirely even though the termination is
        // requested in the middle of it, and the next one is not processed.
        let remaining_names = existing_names(&destination_path, &names);
        assert_eq!(remaining_names.len(), 1);
        let remaining_path = destination_path.join("Backups").join(&remaining_names[0]);
        assert_eq!(file_count(&remaining_path), FILE_COUNT);
    }

    #[test]
    fn termination_in_background_is_joinable() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("ntm");
        let names = removed_names(10);
        create_backups(&destination_path, &names);

        let mut remover = BackupRemover::new();
        remover.set_destination_path(&destination_path.to_string_easy());
        remover.execute_in_background().unwrap();
        remover.terminate();
        remover.join().unwrap();

        // Removing all the files takes far longer than calling terminate(), so
        // the termination is requested before all the backups are removed.
        assert!(!existing_names(&destination_path, &names).is_empty());
    }
}
