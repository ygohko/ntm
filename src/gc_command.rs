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

use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;
use std::path::Path;

 use crate::backup_store::BackupStore;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "gc_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub last_processed_id: String,
}

impl State {
    pub fn new() -> Self {
        Self {
            last_processed_id: "".to_string(),
        }
    }
}

pub struct GcCommand {
    destination_path: String,
    limited_count: Option<i64>,
    object_store: Option<ObjectStore>,
    backup_paths: Vec<String>,
    state: State,
    processed_count: i64,
    removed_count: i64,
    count: i32,
}

impl GcCommand {
    pub fn new() -> Self {
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

    pub fn execute(&mut self) -> Result<()> {
        let path = self.destination_path.pushed("Objects");
        self.object_store = Some(ObjectStore::new(&path));
        
        let mut backups_path = self.destination_path.clone();
        backups_path = backups_path.pushed("Backups");
        let backup_store = BackupStore::new(&backups_path);
        let names = match backup_store.names() {
            Ok(names) => names,
            Err(error) => return Err(error),
        };
        for name in names {
            let backup_path = backups_path.pushed(&name);
            self.backup_paths.push(backup_path);
        }

        let mut offset = 0;
        let mut state_path = self.destination_path.clone();
        state_path = state_path.pushed("state.json");
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
            if let Err(error) = self.process_unit(index1, index2) {
                println!("Warning: Processing unit failed. error: {}", error);
            }

            if (i & 0xFF) == 0 {
                if let Ok(serialized) = serde_json::to_string(&self.state) {
                    let mut path = self.destination_path.clone();
                    path = path.pushed("state.json");
                    if let Err(_) = fs::write(&path, &serialized) {
                        println!("Warning: Writing state failed.");
                    }
                }
            }
            if let Some(count) = self.limited_count {
                if self.processed_count >= count {
                    break;
                }
            }
        }

        println!("{} object(s) removed.", self.removed_count);
        
        Ok(())
    }

    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    pub fn set_limited_count(&mut self, count: i64) {
        self.limited_count = Some(count);
    }

    fn process_unit(&mut self, index1: i32, index2: i32) -> Result<()> {
        let path1 = format!("{:02x}", index1);
        let path2 = format!("{:02x}", index2);
        let mut object_path = self.destination_path.clone();
        object_path = object_path.pushed("Objects");
        object_path = object_path.pushed(&path1);
        object_path = object_path.pushed(&path2);
        if !Path::new(&object_path).exists() {
            return Ok(());
        }
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
                    let mut path = path1.clone();
                    path = path.pushed(&path2);
                    path = path.pushed(&produced_path);
                    if let Err(error) = self.process_object(&path) {
                        println!(
                            "Warning: error caused when processing objects. error: {}",
                            error
                        );
                    }
                    self.processed_count += 1;
                    self.state.last_processed_id = produced_path.file_name();
                }
            }
        }

        Ok(())
    }

    fn process_object(&mut self, path: &str) -> Result<()> {
        let object_store = self.object_store.as_ref().unwrap();
        if self.count == 0 {
            println!(
                "Processing ({}, {}): {}",
                self.processed_count, self.removed_count, path
            );
        }
        self.count += 1;
        self.count %= 100;

        let object_id = path.file_name();
        let attributes = match object_store.attributes(&object_id) {
            Ok(attributes) => attributes,
            Err(error) => return Err(error),
        };

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

        if let Err(_) = object_store.remove(&object_id) {
            println!("Warning: Removing object {} failed.", object_id);
        }
        self.removed_count += 1;

        Ok(())
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
