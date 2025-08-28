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
use chrono::NaiveDateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use std::sync::RwLock;

use crate::attributes::Attributes;
use crate::backup_store::BackupStore;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;
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
    destination_path: String,
    limited_count: Option<i64>,
    object_store: Option<ObjectStore>,
    backup_paths: Vec<String>,
    state: State,
    processed_count: i64,
    removed_count: i64,
    count: i32,
}

impl Private {
    fn new() -> Self {
        Self {
            destination_path: ".".to_string(),
            limited_count: None,
            object_store: None,
            backup_paths: Vec::new(),
            state: State::new(),
            processed_count: 0,
            removed_count: 0,
            count: 0,
        }
    }
}

pub struct GarbageCollector {
    private: Arc<RwLock<Private>>,
}

impl Task for GarbageCollector {
    /// Executes the configured operation in a new background thread.
    ///
    /// This method creates a new thread to run the core logic, represented by the `main` function.
    /// It safely shares the internal `Private` state (which holds configuration like
    /// `destination_path` and `limited_count`) with the new thread by cloning the `Arc<RwLock>`.
    ///
    /// The method blocks the current thread until the spawned background thread completes its execution.
    /// Errors potentially returned by the `main` function within the background thread are currently ignored.
    ///
    /// # Returns
    ///
    /// `Ok(())` upon successful completion of the background thread.
    ///
    /// # Panics
    ///
    /// This method will panic if the background thread itself panics during its execution.
    fn execute(&mut self) -> Result<()> {
        let private = self.private.clone();
        let handle = thread::spawn(move || {
            let _ = main(&private);
        });
        let _ = handle.join();

        Ok(())
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
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned, indicating a previous operation
    /// on the protected data failed catastrophically.
    pub fn set_destination_path(&mut self, path: &str) {
        let mut private = self.private.write().unwrap();
        private.destination_path = path.to_string();
    }

    /// Sets an optional limited count for the internal private data.
    ///
    /// This method attempts to acquire a write lock on the private data.
    /// If the lock is successfully acquired, the `limited_count` is updated to `Some(count)`.
    /// If acquiring the lock fails (e.g., due to the lock being poisoned),
    /// the operation silently fails, and the count is not updated.
    ///
    /// # Arguments
    ///
    /// * `count` - An `i64` value representing the new limited count.
    pub fn set_limited_count(&mut self, count: i64) {
        let mut private = self.private.write().unwrap();
        private.limited_count = Some(count);
    }
}

fn main(private: &Arc<RwLock<Private>>) -> Result<()> {
    let destination_path: String;
    {
        let private = private.read().unwrap();
        destination_path = private.destination_path.clone();
    }
    let mut path = Utf8PathBuf::from(&destination_path);
    path.push("Objects");
    {
        let mut private = private.write().unwrap();
        private.object_store = Some(ObjectStore::new(&path.to_string_easy()));
        if let Some(object_store) = private.object_store.as_mut() {
            if let Err(error) = object_store.load_cache() {
                println!("Loading existing IDs failed. error: {}", error);
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
        let mut private = private.write().unwrap();
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
        let index = (i as i32) + offset;
        let index1 = (index / 0x100) & 0xFF;
        let index2 = index & 0xFF;
        if let Err(error) = process_unit(private, index1, index2) {
            println!("Warning: Processing unit failed. error: {}", error);
        }

        if (i & 0xFF) == 0 {
            let private = private.read().unwrap();
            if let Ok(serialized) = serde_json::to_string(&private.state) {
                let mut path = Utf8PathBuf::from(&destination_path);
                path.push("state.json");
                if let Err(_) = fs::write(&path, &serialized) {
                    println!("Warning: Writing state failed.");
                }
            }
        }
        {
            let private = private.read().unwrap();
            if let Some(count) = private.limited_count {
                if private.processed_count >= count {
                    break;
                }
            }
        }
    }

    {
        let private = private.read().unwrap();
        println!("{} object(s) removed.", private.removed_count);
    }

    Ok(())
}

fn process_unit(private: &Arc<RwLock<Private>>, index1: i32, index2: i32) -> Result<()> {
    let destination_path: String;
    {
        let private = private.read().unwrap();
        destination_path = private.destination_path.clone();
    }
    let directory1 = format!("{:02x}", index1);
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
                if let Err(error) = process_object(private, &path.to_string_easy()) {
                    println!(
                        "Warning: error caused when processing objects. error: {}",
                        error
                    );
                }
                {
                    let mut private = private.write().unwrap();
                    private.processed_count += 1;
                    private.state.last_processed_id = file_name;
                }
            }
        }
    }

    Ok(())
}

fn process_object(private: &Arc<RwLock<Private>>, path: &str) -> Result<()> {
    let backup_paths: Vec<String>;
    {
        let private = private.read().unwrap();
        if private.count == 0 {
            println!(
                "Processing ({}, {}): {}",
                private.processed_count, private.removed_count, path
            );
        }
        backup_paths = private.backup_paths.clone();
    }
    {
        let mut private = private.write().unwrap();
        private.count += 1;
        private.count %= 100;
    }

    let path1 = Utf8PathBuf::from(&path);
    let object_id = path1.file_name_or_empty();
    let attributes: Attributes;
    {
        let private = private.read().unwrap();
        let object_store = private.object_store.as_ref().unwrap();
        if object_store.cached(&object_id)? {
            return Ok(());
        }
        attributes = object_store.attributes(&object_id)?;
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
        let private = private.read().unwrap();
        if let Some(object_store) = &private.object_store {
            if let Err(_) = object_store.remove(&object_id) {
                println!("Warning: Removing object {} failed.", object_id);
            }
        }
    }
    {
        let mut private = private.write().unwrap();
        private.removed_count += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::OperatePath;
    use crate::garbage_collector::GarbageCollector;
    use crate::init_command::InitCommand;
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
        let mut command = InitCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut backup_path = ntm_path.clone();
        backup_path.push("Backups");
        backup_path.push(&command.name());
        fs::remove_dir_all(&backup_path).unwrap();
        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&ntm_path.to_string_easy());
        collector.execute().unwrap();
    }
}
