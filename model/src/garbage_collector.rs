/*
 * Copyright (c) 2025 - 2026 Yasuaki Gohko
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
use chrono::NaiveDateTime;
use parking_lot::RwLock;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

use crate::attributes::Attributes;
use crate::backup_store::BackupStore;
use crate::commons::Notification;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;
use crate::task;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "garbage_collector";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;

#[derive(Serialize, Deserialize, Clone)]
struct State {
    pub last_processed_id: String,
}

impl State {
    fn new() -> Self {
        Self {
            last_processed_id: "".to_string(),
        }
    }
}

struct Private {
    join_handle: Option<JoinHandle<Result<()>>>,
    destination_path: String,
    limited_count: Option<i64>,
    object_store: Option<ObjectStore>,
    backup_paths: Vec<String>,
    state: State,
    processed_count: i64,
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
            limited_count: None,
            object_store: None,
            backup_paths: Vec::new(),
            state: State::new(),
            processed_count: 0,
            removed_count: 0,
            count: 0,
            notification_sender: None,
            termination_requested: false,
        }
    }
}

#[derive(Clone)]
pub struct GarbageCollector {
    private: Arc<RwLock<Private>>,
}

impl Task for GarbageCollector {
    /// Removes the objects that are no longer referred to by any backup, in the
    /// calling thread.
    ///
    /// For each object in `<destination_path>/Objects`, the backups created after
    /// the object was added are checked. If none of them has an entry at the
    /// object's path that refers to the object, the object is removed. Objects in
    /// the cache of the object store are kept without checking.
    ///
    /// The large objects are processed first, then the other objects. The other
    /// objects are processed from the unit next to the last processed one saved in
    /// `<destination_path>/state.json`, and the progress is saved there every 256
    /// units. The processing stops when the number of processed objects reaches the
    /// count set by `set_limited_count()`, or when `terminate()` is called.
    ///
    /// Failing to process or remove an object, to load the cache or to save the
    /// state is printed as a warning and the processing continues.
    ///
    /// This blocks until the processing finishes. Use `execute_in_background()` to
    /// run it in another thread.
    ///
    /// # Errors
    ///
    /// * `Error::new(backup_store::ERROR_ID, backup_store::ERROR_CODE_FINDING_BACKUP_FAILED)`:
    ///   If the `Backups` directory cannot be read.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the garbage collection fails.
    fn execute(&mut self) -> Result<()> {
        let result = self.main();

        result
    }

    /// Spawns a new thread to execute the main application logic.
    ///
    /// This method initializes a new background thread to perform the core operations.
    /// It first clones the `self.private` data, which is then moved into the new thread's
    /// closure. Inside this new thread, the `main` function is called, receiving a
    /// reference to this cloned private data.
    ///
    /// The `JoinHandle` for the newly spawned thread is stored in `self.join_handle`.
    /// This handle can later be used to await the thread's completion, check its status,
    /// or handle panics.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the thread was successfully spawned.
    ///
    /// The current implementation always returns `Ok(())` as `thread::spawn` itself
    /// returns a `JoinHandle` directly and does not typically fail to *spawn* the thread
    /// in a way that returns a `Result`. The `Result<()>` return type might be present
    /// for future error handling or to satisfy a trait's signature.
    fn execute_in_background(&mut self) -> Result<()> {
        let collector = self.clone();
        let mut private = self.private.write();
        private.join_handle = Some(thread::spawn(move || {
            let result = collector.main();

            result
        }));

        Ok(())
    }

    /// Waits for the previously spawned task or thread to complete and returns its result.
    ///
    /// This method takes ownership of the `join_handle` from the instance. If no
    /// handle is present (meaning no task was spawned or it was already joined),
    /// an error indicating an unsupported operation is returned. If the joined task
    /// panics, an error indicating a panic is returned.
    ///
    /// # Errors
    ///
    /// * `Error::new(task::ERROR_ID, task::ERROR_CODE_NOT_SUPPORTED)`: If there is no
    ///   `join_handle` available to join (e.g., the task was not spawned, or it was
    ///   already joined and the handle consumed).
    /// * `Error::new(task::ERROR_ID, task::ERROR_CODE_PANICKED)`: If the underlying
    ///   task or thread being joined panics during its execution.
    ///
    /// # Returns
    ///
    /// A `Result<()>` representing the outcome of the joined task. `Ok(())` on successful
    /// completion, or an `Err` if the task itself returned an error.
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

impl GarbageCollector {
    /// Creates a new instance of `GarbageCollector`.
    ///
    /// This initializes the internal state, specifically a `private` field, as a new
    /// `Private` instance wrapped in an `Arc<RwLock>`. This setup allows for
    /// shared, thread-safe, and mutable access to the private data.
    ///
    /// # Returns
    ///
    /// A new instance of `GarbageCollector`.
    pub fn new() -> Self {
        Self {
            private: Arc::new(RwLock::new(Private::new())),
        }
    }

    /// Sets the destination path where processed files or data will be stored.
    ///
    /// This operation acquires a write lock on the internal state to update the path.
    ///
    /// # Arguments
    ///
    /// * `path` - The new path as a string slice. The path is cloned and stored internally.
    pub fn set_destination_path(&self, path: &str) {
        let mut private = self.private.write();
        private.destination_path = path.to_string();
    }

    /// Sets an optional limited count for the internal private data.
    ///
    /// This method attempts to acquire a write lock on the private data.
    /// If the lock is successfully acquired, the `limited_count` is updated to `Some(count)`.
    ///
    /// # Arguments
    ///
    /// * `count` - An `i64` value representing the new limited count.
    pub fn set_limited_count(&self, count: i64) {
        let mut private = self.private.write();
        private.limited_count = Some(count);
    }

    /// Sets the notification sender.
    ///
    /// # Arguments
    ///
    /// * `sender` - The sender to use sending notifications.
    pub fn set_notification_sender(&mut self, sender: Sender<Notification>) {
        self.private.write().notification_sender = Some(sender);
    }

    /// Requests the running garbage collection to terminate.
    ///
    /// This only sets a flag and returns immediately. The flag is checked after
    /// the large objects are processed and after each unit of the other objects
    /// is processed, so the garbage collection stops at that point and
    /// `execute()` or `join()` returns `Ok(())`. Processing the large objects is
    /// not interrupted on the way. Use `join()` to wait for the termination when
    /// the garbage collection is executed in the background.
    ///
    /// The progress is saved to `state.json` only every 256 units, so the next
    /// garbage collection resumes from the last saved point, not from the point
    /// where it was terminated.
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
        path.push("Objects");
        {
            let mut private = self.private.write();
            private.object_store = Some(ObjectStore::new(&path.to_string_easy()));
            if let Some(ref mut object_store) = private.object_store {
                if let Err(error) = object_store.load_cache() {
                    if let Some(ref sender) = private.notification_sender {
                        let string = format!("Loading existing IDs failed. error: {}", error);
                        let _ = sender.send(Notification::Error(string));
                    }
                }
            }
        }
        let mut backups_path = Utf8PathBuf::from(&destination_path);
        backups_path.push("Backups");
        let backup_store = BackupStore::new(&backups_path.to_string_easy());
        let names = match backup_store.names() {
            Ok(names) => names,
            Err(error) => return Err(error),
        };

        {
            let mut private = self.private.write();
            for name in names {
                let backup_path = backups_path.join(&name);
                private.backup_paths.push(backup_path.to_string_easy());
            }
            private.backup_paths.sort_by(|a, b| b.cmp(a));
        }

        let mut offset = 0;
        let mut state_path = Utf8PathBuf::from(&destination_path);
        state_path.push("state.json");
        if let Ok(serialized) = fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<State>(&serialized) {
                let string = &state.last_processed_id[0..4];
                if let Ok(mut value) = u32::from_str_radix(&string, 16) {
                    value += 1;
                    offset = (value & 0xFFFF) as i32;
                }
            }
        }

        for i in 0..65536 {
            let index1 = (i / 0x100) & 0xFF;
            let index2 = i & 0xFF;
            if let Err(error) = self.process_unit(index1, index2, true) {
                if let Some(ref sender) = self.private.read().notification_sender {
                    let string = format!("Warning: Processing unit failed. error: {}", error);
                    let _ = sender.send(Notification::Error(string));
                }
            }
        }
        if self.private.read().termination_requested {
            return Ok(());
        }
        
        for i in 0..65536 {
            let index = (i as i32) + offset;
            let index1 = (index / 0x100) & 0xFF;
            let index2 = index & 0xFF;
            if let Err(error) = self.process_unit(index1, index2, false) {
                if let Some(ref sender) = self.private.read().notification_sender {
                    let string = format!("Warning: Processing unit failed. error: {}", error);
                    let _ = sender.send(Notification::Error(string));
                }
            }

            if (i & 0xFF) == 0 {
                let private = self.private.read();
                if let Ok(serialized) = serde_json::to_string(&private.state) {
                    let mut path = Utf8PathBuf::from(&destination_path);
                    path.push("state.json");
                    if let Err(_) = fs::write(&path, &serialized) {
                        if let Some(ref sender) = self.private.read().notification_sender {
                            let string = format!("Warning: Writing state failed.");
                            let _ = sender.send(Notification::Error(string));
                        }
                    }
                }
            }
            {
                let private = self.private.read();
                if let Some(count) = private.limited_count {
                    if private.processed_count >= count {
                        break;
                    }
                }
                if private.termination_requested {
                    break;
                }
            }
        }

        if let Some(ref sender) = self.private.read().notification_sender
        {
            let count = self.private.read().removed_count;
            let string = format!("{} object(s) removed.", count);
            let _ = sender.send(Notification::Progress(string));
        }

        Ok(())
    }

    fn process_unit(&self, index1: i32, index2: i32, large: bool) -> Result<()> {
        let destination_path: String;
        {
            let private = self.private.read();
            destination_path = private.destination_path.clone();
        }
        let mut directory1 = format!("{:02x}", index1);
        if large {
            directory1 = "l".to_string() + &directory1;
        }
        let directory2 = format!("{:02x}", index2);
        let mut object_path = Utf8PathBuf::from(&destination_path);
        object_path.push("Objects");
        object_path.push(&directory1);
        object_path.push(&directory2);
        if !Path::new(&object_path).exists() {
            return Ok(());
        }
        let mut producer = FilePathProducer::new(&object_path.to_string_easy());
        let mut done = false;
        while !done {
            let option = match producer.next() {
                Ok(path) => Some(path),
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID
                        && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED
                    {
                        done = true;
                    }

                    None
                }
            };

            if let Some(produced_path) = option {
                let path = Utf8PathBuf::from(&produced_path);
                let extension = path.extension_or_empty();
                let file_name = path.file_name_or_empty();
                if extension == "" {
                    let mut path = Utf8PathBuf::from(&directory1);
                    path.push(&directory2);
                    path.push(&produced_path);
                    if let Err(error) = self.process_object(&path.to_string_easy()) {
                        if let Some(ref sender) = self.private.read().notification_sender {
                            let string = format!(
                                "Warning: error caused when processing objects. error: {}",
                                error
                            );
                            let _ = sender.send(Notification::Error(string));
                        }
                    }
                    {
                        let mut private = self.private.write();
                        private.processed_count += 1;
                        private.state.last_processed_id = file_name;
                    }
                }
            }
        }

        Ok(())
    }

    fn process_object(&self, path: &str) -> Result<()> {
        let backup_paths: Vec<String>;
        {
            let private = self.private.read();
            if private.count == 0 {
                if let Some(ref sender) = private.notification_sender {
                    let string = format!(
                        "Processing ({}, {}): {}",
                        private.processed_count, private.removed_count, path
                    );
                    let _ = sender.send(Notification::Progress(string));
                }
            }
            backup_paths = private.backup_paths.clone();
        }
        {
            let mut private = self.private.write();
            private.count += 1;
            private.count %= 100;
        }

        let path1 = Utf8PathBuf::from(&path);
        let object_id = path1.file_name_or_empty();
        let attributes: Attributes;
        {
            let private = self.private.read();
            if let Some(object_store) = &private.object_store {
                if object_store.cached(&object_id)? {
                    return Ok(());
                }
                attributes = object_store.attributes(&object_id)?;
            } else {
                panic!();
            }
        }

        for backup_path in &backup_paths {
            let path2 = Utf8PathBuf::from(&backup_path);
            let backup_name = path2.file_name_or_empty();
            let backup_created = match NaiveDateTime::parse_from_str(&backup_name, "%Y%m%d-%H%M") {
                // Add 25 hours because backup_created does not have timezone.
                Ok(created) => created.and_utc().timestamp() + 25 * 60 * 60,
                Err(_) => attributes.added,
            };
            if (backup_created - attributes.added) < 0 {
                // All backups after this object is added are checked.
                // println!("Checking skipped. object_id: {}", object_id);

                break;
            }

            let mut option: Option<String> = None;
            let mut entry_path = Utf8PathBuf::from(&backup_path);
            entry_path.push(&attributes.path);
            if let Ok(serialized) = fs::read_to_string(&entry_path) {
                option = match serde_json::from_str::<Entry>(&serialized) {
                    Ok(entry) => Some(entry.id),
                    Err(_) => None,
                }
            }

            if let Some(entry_object_id) = option {
                if entry_object_id == object_id {
                    // println!("Object {} keeped.", path);

                    return Ok(());
                }
            }
        }

        {
            let private = self.private.read();
            if let Some(object_store) = &private.object_store {
                if let Err(_) = object_store.remove(&object_id) {
                    if let Some(ref sender) = self.private.read().notification_sender {
                        let string = format!("Warning: Removing object {} failed.", object_id);
                        let _ = sender.send(Notification::Error(string));
                    }
                }
            }
        }
        {
            let mut private = self.private.write();
            private.removed_count += 1;
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

    use crate::attributes::Attributes;
    use crate::backup_executer::BackupExecuter;
    use crate::object_store::ObjectStore;
    use crate::commons::OperatePath;
    use crate::garbage_collector::GarbageCollector;
    use crate::destination::Destination;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _collector = GarbageCollector::new();
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
        Destination::init(&ntm_path.to_string_easy(), None).unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let mut config = format!("source_path = \"{}\"", source_path.display());
        if consts::OS == "windows" {
            config = config.replace("\\", "\\\\");
        }
        fs::write(config_path, config).unwrap();

        let mut executer = BackupExecuter::new(&ntm_path.to_string_easy());
        executer.execute().unwrap();

        let mut backup_path = ntm_path.clone();
        backup_path.push("Backups");
        backup_path.push(&executer.name());
        fs::remove_dir_all(&backup_path).unwrap();
        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&ntm_path.to_string_easy());
        collector.execute().unwrap();
    }

    const NORMAL_OBJECT_IDS: [&str; 2] = ["0101010101010101", "0202020202020202"];
    const LARGE_OBJECT_ID: &str = "0303030303030303";

    /// Creates a destination with no backups. Its objects are not referred to by
    /// any backup, so all of them are garbage.
    fn create_destination(path: &Path) {
        Destination::init(&path.to_string_easy(), None).unwrap();
        let objects_path = path.join("Objects");
        let mut store = ObjectStore::new(&objects_path.to_string_easy());
        for id in NORMAL_OBJECT_IDS {
            store.add(id, &b"normal".to_vec(), &Attributes::new("normal.txt", 0)).unwrap();
        }

        // Adding a large object needs 1 GiB of bytes, so it is placed directly.
        let large_path = objects_path.join("l03").join("03").join("03").join("03");
        fs::create_dir_all(&large_path).unwrap();
        fs::write(large_path.join(LARGE_OBJECT_ID), "large").unwrap();
        let attributes = serde_json::to_string(&Attributes::new("large.bin", 0)).unwrap();
        fs::write(large_path.join(format!("{}.attributes", LARGE_OBJECT_ID)), attributes).unwrap();
    }

    fn object_exists(destination_path: &Path, id: &str) -> bool {
        let store = ObjectStore::new(&destination_path.join("Objects").to_string_easy());
        store.bytes(id).is_ok()
    }

    #[test]
    fn garbage_objects_are_removed_without_termination() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("ntm");
        create_destination(&destination_path);

        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&destination_path.to_string_easy());
        collector.execute().unwrap();

        for id in NORMAL_OBJECT_IDS {
            assert!(!object_exists(&destination_path, id));
        }
        assert!(!object_exists(&destination_path, LARGE_OBJECT_ID));
        assert!(destination_path.join("state.json").exists());
    }

    #[test]
    fn termination_stops_after_large_objects() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("ntm");
        create_destination(&destination_path);

        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&destination_path.to_string_easy());
        collector.terminate();
        collector.execute().unwrap();

        // Processing the large objects is not interrupted.
        assert!(!object_exists(&destination_path, LARGE_OBJECT_ID));
        // The other objects are not processed.
        for id in NORMAL_OBJECT_IDS {
            assert!(object_exists(&destination_path, id));
        }
        // The state is not saved because no other objects are processed.
        assert!(!destination_path.join("state.json").exists());
    }

    #[test]
    fn termination_in_background_is_joinable() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("ntm");
        create_destination(&destination_path);

        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&destination_path.to_string_easy());
        collector.execute_in_background().unwrap();
        collector.terminate();
        collector.join().unwrap();

        // Scanning all units of the large objects takes far longer than calling
        // terminate(), so the termination is requested before the other objects.
        for id in NORMAL_OBJECT_IDS {
            assert!(object_exists(&destination_path, id));
        }
    }
}
