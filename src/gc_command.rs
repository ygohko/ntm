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

use std::fs;

use crate::commons::ConvertPath;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::Attributes;
use crate::object_store::MarkingResult;
use crate::object_store::ObjectStore;

pub const ERROR_ID: ErrorId = "gc_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_FINDING_BACKUP_FAILED: ErrorCode = 1;
pub const ERROR_CODE_PROCESSING_ENTRY_FAILED: ErrorCode = 2;
pub const ERROR_CODE_PROCESSING_OBJECT_FAILED: ErrorCode = 3;

pub struct GcCommand {
    destination_path: String,
    store: ObjectStore,
    backup_paths: Vec<String>,
    processed_count: i64,
    removed_count: i64,
    count: i32,
}

impl GcCommand {
    pub fn new() -> Self {
        Self {
            destination_path: ".".to_string(),
            store: ObjectStore::new(&"Objects"),
            backup_paths: Vec::new(),
            processed_count: 0,
            removed_count: 0,
            count: 0,
        }
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut backup_path = self.destination_path.clone();
        backup_path = backup_path.pushed("Backups");
        let Ok(read_dir) = fs::read_dir(&backup_path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_FINDING_BACKUP_FAILED));
        };
        for result in read_dir {
            if let Ok(entry) = result {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        self.backup_paths.push(String::from_path(&entry.path()));
                    }
                }
            }
        }

        let mut object_path = self.destination_path.clone();
        object_path = object_path.pushed("Objects");
        let mut producer = FilePathProducer::new(&object_path);
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
                if produced_path.extension() == "" {
                    if let Err(error) = self.process_object(&produced_path) {
                        println!(
                            "Warning: error caused when processing objects. error: {}",
                            error
                        );
                    }
                    self.processed_count += 1;
                }
            }
        }

        Ok(())
    }

    pub fn execute_old(&mut self) -> Result<()> {
        let path = self.destination_path.pushed("Objects");
        self.store = ObjectStore::new(&path);
        let backup_paths = match self.backup_paths() {
            Ok(backup_paths) => backup_paths,
            Err(error) => return Err(error),
        };
        for path in backup_paths {
            println!("Processing backup: {}", path);
            if let Err(error) = self.process_backup(&path) {
                println!("Processing backup {} failed. error: {}", path, error);
            }
        }
        self.store.sweep()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    fn process_object(&mut self, path: &str) -> Result<()> {
        if self.count == 0 {
            println!(
                "Processing ({}, {}): {}",
                self.processed_count, self.removed_count, path
            );
        }
        self.count += 1;
        self.count %= 100;

        let mut attributes_path = self.destination_path.clone();
        attributes_path = attributes_path.pushed("Objects");
        attributes_path = attributes_path.pushed(path);
        attributes_path += ".attributes";

        // println!("attributes_path: {}", attributes_path);

        let Ok(serialized) = fs::read_to_string(&attributes_path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_OBJECT_FAILED));
        };
        let attributes: Attributes = match serde_json::from_str(&serialized) {
            Ok(attributes) => attributes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_OBJECT_FAILED)),
        };

        let object_id = path.file_name();
        for backup_path in &self.backup_paths {
            let mut option: Option<String> = None;
            let entry_path = backup_path.pushed(&attributes.path);
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

        let mut object_path = self.destination_path.clone();
        object_path = object_path.pushed("Objects");
        object_path = object_path.pushed(path);
        if let Err(_) = fs::remove_file(&object_path) {
            println!("Warning: Removing {} failed.", object_path);
        }
        if let Err(_) = fs::remove_file(&attributes_path) {
            println!("Warning: Removing {} failed.", attributes_path);
        }
        self.removed_count += 1;

        Ok(())
    }

    fn process_backup(&mut self, path: &str) -> Result<()> {
        let mut producer = FilePathProducer::new(&path);
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
                    // TODO: Displaying errors would be needed.

                    None
                }
            };
            if let Some(produced_path) = option {
                let entry_path = path.pushed(&produced_path);
                if let Err(error) = self.process_entry(&entry_path) {
                    println!("Processing entry {} failed. error: {}", entry_path, error);
                }
            }
        }

        Ok(())
    }

    fn process_entry(&mut self, path: &str) -> Result<()> {
        if self.count == 0 {
            // println!("Processing ({}, {}): {}", self.processed_count, self.marked_count, path);
        }
        self.count += 1;
        self.count %= 100;
        let string = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_ENTRY_FAILED)),
        };
        let entry: Entry = match serde_json::from_str(&string) {
            Ok(entry) => entry,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_ENTRY_FAILED)),
        };
        let result = self.store.mark(&entry.id)?;
        self.processed_count += 1;
        if result == MarkingResult::Marked {
            // self.marked_count += 1;
        }

        Ok(())
    }

    fn backup_paths(&self) -> Result<Vec<String>> {
        let path = self.destination_path.pushed("Backups");
        let read_dir = match fs::read_dir(&path) {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_FINDING_BACKUP_FAILED)),
        };
        let mut backup_paths: Vec<String> = Vec::new();
        for result in read_dir {
            if result.is_ok() {
                let entry = result.unwrap();
                let path = entry.path();
                let result = entry.metadata();
                if result.is_ok() {
                    let metadata = result.unwrap();
                    if metadata.is_dir() && !metadata.is_symlink() {
                        backup_paths.push(String::from_path(&path));
                    }
                }
            }
        }

        Ok(backup_paths)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::ConvertPath;
    use crate::gc_command::GcCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = GcCommand::new();
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
        command.set_destination_path(&String::from_path(&ntm_path));
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.set_destination_path(&String::from_path(&ntm_path));
        command.execute().unwrap();

        let mut backup_path = ntm_path.clone();
        backup_path.push("Backups");
        backup_path.push(&command.name);
        fs::remove_dir_all(&backup_path).unwrap();
        let mut command = GcCommand::new();
        command.set_destination_path(&String::from_path(&ntm_path));
        command.execute().unwrap();
    }
}
