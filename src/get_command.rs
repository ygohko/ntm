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
use std::path::PathBuf;

use crate::commons::ConvertPath;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

pub const ERROR_ID: ErrorId = "get_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;
pub const ERROR_CODE_READING_ENTRY_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_BYTES_FAILED: ErrorCode = 3;

pub struct GetCommand {
    backup: String,
    limited_path: String,
}

impl GetCommand {
    pub fn new(backup: &str) -> Self {
        GetCommand {
            backup: backup.to_string(),
            limited_path: "".to_string(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        // TODO: Return error if path is invalid.
        let store = ObjectStore::new(&"Objects");
        let mut backup_path = PathBuf::new();
        backup_path.push("Backups");
        backup_path.push(&self.backup);
        let mut path = PathBuf::new();
        path.push(&backup_path);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)),
        };
        if !exists {
            return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND));
        }

        // TODO: Produce file paths only in specified path when argument is given.
        if self.limited_path != "".to_string() {
            path.push(&self.limited_path);
        }
        let mut producer = FilePathProducer::new(&String::from_path(&path));
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
                let mut entry_path = PathBuf::new();
                entry_path.push(&backup_path);
                if self.limited_path != "".to_string() {
                    entry_path.push(&self.limited_path);
                }
                entry_path.push(&path);
                println!("entry_path: {}", entry_path.display());
                let string = match fs::read_to_string(entry_path.clone()) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED))
                    }
                };

                let entry: Entry = match serde_json::from_str(&string) {
                    Ok(entry) => entry,
                    Err(_) => {
                        return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED))
                    }
                };

                println!("entry.id: {}", entry.id);

                let bytes = match store.bytes(&entry.id) {
                    Ok(bytes) => bytes,
                    // TODO: Skipping file that object is not found may be needed.
                    Err(error) => return Err(error),
                };
                let mut destination_path = PathBuf::new();
                destination_path.push(&self.backup);
                destination_path.push(&path);
                println!("destination_path: {}", destination_path.display());
                let directories = String::from_path(&destination_path).directories();
                match fs::create_dir_all(&directories) {
                    Ok(_) => (),
                    // TODO: Skipping file that writing is failed may be needed.
                    Err(_) => {
                        return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED))
                    }
                }
                match fs::write(destination_path, bytes) {
                    Ok(_) => (),
                    // TODO: Skipping file that writing is failed may be needed.
                    Err(_) => {
                        return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED))
                    }
                };
            }
        }

        Ok(())
    }

    pub fn set_limited_path(&mut self, path: &str) -> () {
        self.limited_path = path.to_string();
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::get_command::GetCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = GetCommand::new("");
    }

    #[test]
    fn is_executable() {
        // TODO: Do not modify current directory.
        let temp_dir = TempDir::new("test").unwrap();
        let previous_current_dir = env::current_dir().unwrap();

        let mut temp_path = previous_current_dir.clone();
        temp_path.push(&temp_dir.path());
        let mut source_path = temp_path.clone();
        source_path.push("source");
        fs::create_dir_all(&source_path).unwrap();
        let mut file_path = source_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "ABCDE").unwrap();

        let mut ntm_path = temp_path.clone();
        ntm_path.push("ntm");
        fs::create_dir_all(&ntm_path).unwrap();
        env::set_current_dir(&ntm_path).unwrap();
        let command = InitCommand::new();
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.execute().unwrap();
        let date_time = command.date_time;

        let command = GetCommand::new(&date_time);
        command.execute().unwrap();

        env::set_current_dir(&previous_current_dir).unwrap();
    }
}
