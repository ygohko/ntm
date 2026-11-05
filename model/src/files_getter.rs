/*
 * Copyright (c) 2026 Yasuaki Gohko
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
use std::fs::File;
use std::ops::Add;
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use std::time::SystemTime;

use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorId;
use crate::error::ErrorCode;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "files_getter";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;
pub const ERROR_CODE_READING_ENTRY_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_BYTES_FAILED: ErrorCode = 3;
pub const ERROR_CODE_WRITING_METADATA_FAILED: ErrorCode = 4;

pub struct FilesGetter {
    name: String,
    extraction_path: String,
    destination_path: String,
    gotten_path: String,
    processed_count: i64,
    count: i32,
}

impl Task for FilesGetter {
    /// Executes the files getter.
    ///
    /// This method retrieves backed-up files and directories from the object store
    /// and restores them to the specified destination.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
    fn execute(&mut self) -> Result<()> {
        // TODO: Return error if path is invalid.
        let mut path = Utf8PathBuf::from(&self.destination_path);
        path.push("Objects");
        let store = ObjectStore::new(&path.to_string_easy());
        let mut backup_path = Utf8PathBuf::from(&self.destination_path);
        backup_path.push("Backups");
        backup_path.push(&self.name);
        let mut path = Utf8PathBuf::new();
        path.push(&backup_path);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)),
        };
        if !exists {
            return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND));
        }

        // TODO: Do single file extraction if extraction_path points a file.

        // TODO: Move to other method.
        if self.extraction_path != "".to_string() {
            path.push(&self.extraction_path);
        }
        let mut producer = FilePathProducer::new(&path.to_string_easy());
        let mut done = false;
        while !done {
            let path = match producer.next() {
                Ok(path) => path,
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
                let mut entry_path = Utf8PathBuf::new();
                entry_path.push(&backup_path);
                if self.extraction_path != "".to_string() {
                    entry_path.push(&self.extraction_path);
                }
                entry_path.push(&path);
                if self.count == 0 {
                    println!("Processing ({}): {}", self.processed_count, entry_path);
                }
                self.count += 1;
                self.count %= 100;
                let string = match fs::read_to_string(entry_path.clone()) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
                };

                let entry: Entry = match serde_json::from_str(&string) {
                    Ok(entry) => entry,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
                };

                let result = store.bytes(&entry.id);
                if let Err(ref error) = result {
                    println!("store.bytes() failed. error: {}", error);
                }
                if let Ok(bytes) = result {
                    let mut gotten_path = Utf8PathBuf::from(&self.gotten_path);
                    gotten_path.push(&self.name);
                    if self.extraction_path != "".to_string() {
                        gotten_path.push(&self.extraction_path);
                    }
                    gotten_path.push(&path);
                    let directories = gotten_path.parent_or_empty();
                    match fs::create_dir_all(&directories) {
                        Ok(_) => (),
                        // TODO: Skipping file that writing is failed may be needed.
                        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                    }
                    match fs::write(&gotten_path, bytes) {
                        Ok(_) => (),
                        // TODO: Skipping file that writing is failed may be needed.
                        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                    };
                    if let Err(error) = apply_metadata(&gotten_path.to_string_easy(), &entry) {
                        println!("apply_metadata() failed: error: {}", error);
                    }

                    self.processed_count += 1;
                }
            }
        }
        println!("{} file(s) gotten.", self.processed_count);

        Ok(())
    }
}

impl FilesGetter {
    /// Creates a new `FilesGetter` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the backup to retrieve.
    ///
    /// # Returns
    ///
    /// * `GetCommand` - A new `GetCommand` instance.
    pub fn new(name: &str, extraction_path: &str, destination_path: &str, gotten_path: &str) -> Self {
        Self {
            name: name.to_string(),
            extraction_path: extraction_path.to_string(),
            destination_path: destination_path.to_string(),
            gotten_path: gotten_path.to_string(),
            processed_count: 0,
            count: 0,
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_metadata(path: &str, entry: &Entry) -> Result<()> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
    };
    let mut modified = SystemTime::UNIX_EPOCH;
    modified = modified.add(Duration::from_secs(entry.last_modified));
    if let Err(_) = file.set_modified(modified) {
        return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
    }

    if entry.permission != 0 {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
        };
        let mut permissions = metadata.permissions();
        permissions.set_mode(entry.permission);
        if let Err(_) = fs::set_permissions(path, permissions) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
        }
    }

    if entry.uid != 0 && entry.gid != 0 {
        if let Err(_) = unix_fs::chown(path, Some(entry.uid), Some(entry.gid)) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_metadata(path: &str, entry: &Entry) -> Result<()> {
    if entry.permission != 0 {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
        };
        let mut permissions = metadata.permissions();
        let readonly: bool;
        if (entry.permission & 0o200) != 0 {
            readonly = false;
        } else {
            readonly = true;
        }
        permissions.set_readonly(readonly);
    }

    Ok(())
}
