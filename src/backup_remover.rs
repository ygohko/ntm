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
use std::fs;
use std::fs::DirEntry;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::RwLock;

use crate::commons::OperatePath;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "backup_remover";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_DIRECTORY_FAILED: ErrorCode = 1;

struct Private {
    destination_path: String,
    removed_count: i64,
    count: i32,
}

impl Private {
    fn new() -> Self {
        Self {
            destination_path: ".".to_string(),
            removed_count: 0,
            count: 0,
        }
    }
}

pub struct BackupRemover {
    join_handle: Option<JoinHandle<()>>,
    private: Arc<RwLock<Private>>,
}

impl Task for BackupRemover {
    /// Spawns a new thread to execute the main logic with a copy of the private data.
    ///
    /// This method first clones the `self.private` field to ensure that the data
    /// can be safely moved into the new thread's closure. A new thread is then
    /// spawned, and the cloned `private` data is moved into it. The new thread
    /// will then invoke the `main` function, passing a reference to this `private`
    /// data.
    ///
    /// The `JoinHandle` for the newly created thread is stored in `self.join_handle`,
    /// allowing the program to potentially wait for the thread's completion later
    /// if necessary.
    ///
    /// # Returns
    /// - `Ok(())` if the thread was successfully spawned.
    fn execute(&mut self) -> Result<()> {
        let private = self.private.clone();
        self.join_handle = Some(thread::spawn(move || {
            main(&private);
        }));

        Ok(())
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
            join_handle: None,
            private: Arc::new(RwLock::new(Private::new())),
        }
    }

    /// Joins the underlying thread if a join handle is present.
    ///
    /// This method consumes `self` and blocks the current thread until the
    /// associated thread (if any) has finished execution. If no join handle
    /// is present (e.g., the task was already joined or never started a thread),
    /// this method does nothing.
    ///
    /// After calling `join`, the `Task` object cannot be used further.
    pub fn join(mut self) {
        if self.join_handle.is_some() {
            let handle = self.join_handle.take();
            handle.unwrap().join();
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
    ///
    /// # Panics
    ///
    /// This method will panic if the `RwLock` is poisoned (i.e., a writer
    /// previously panicked while holding the lock).
    pub fn set_destination_path(&mut self, path: &str) {
        let mut private = self.private.write().unwrap();
        private.destination_path = path.to_string();
    }
}

fn main(private :&Arc<RwLock<Private>>) -> Result<()> {
    let destination_path: String;
    {
        let private = private.read().unwrap();
        destination_path = private.destination_path.clone();
    }
    let mut path = Utf8PathBuf::from(&destination_path);
    path.push("Backups");

    let Ok(read_dir) = fs::read_dir(&path) else {
        return Err(Error::new(ERROR_ID, ERROR_CODE_READING_DIRECTORY_FAILED));
    };
    for result in read_dir {
        if let Ok(dir_entry) = result {
            process_dir_entry(private, &dir_entry)?;
        }
    }

    Ok(())
}

fn process_dir_entry(private: &Arc<RwLock<Private>>, dir_entry: &DirEntry) -> Result<()> {
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
            },
        };

        if !done {
            let mut removing_path = Utf8PathBuf::from(&path);
            removing_path.push(&file_path);
            {
                let private = private.read().unwrap();
                if private.count == 0 {
                    println!("Removing ({}): {}", private.removed_count, removing_path);
                }
            }
            if let Err(error) = fs::remove_file(&removing_path) {
                println!("Removing file {} failed. error: {}", removing_path, error);
            }
            {
                let mut private = private.write().unwrap();
                private.removed_count += 1;
                private.count += 1;
                private.count %= 1000;
            }
        }
    }

    if let Err(error) = fs::remove_dir_all(&path) {
        println!("Removing directory {} failed. error: {}", path, error);
    }

    Ok(())
}
