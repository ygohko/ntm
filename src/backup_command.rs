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

use chrono::DateTime;
use chrono::Local;
use hex_string::HexString;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::commons::OperatePath;
use crate::config::Config;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

pub const ERROR_ID: ErrorId = "backup_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_CONFIG_FAILED: ErrorCode = 1;
pub const ERROR_CODE_READING_SOURCE_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_DESTINATION_FAILED: ErrorCode = 3;

pub struct BackupCommand {
    pub date_time: String,
    destination_path: String,
    bytes_id_threshold_min: i64,
    bytes_id_threshold_max: i64,
    excluded_directories: Vec<String>,
    processed_count: i64,
    added_count: i64,
    count: i32,
}

impl BackupCommand {
    pub fn new() -> Self {
        Self {
            date_time: "".to_string(),
            destination_path: ".".to_string(),
            bytes_id_threshold_min: 0,
            bytes_id_threshold_max: 100 * 1024 * 1024,
            excluded_directories: vec![],
            processed_count: 0,
            added_count: 0,
            count: 0,
        }
    }

    pub fn execute(&mut self) -> Result<()> {
        let path = self.destination_path.pushed("Objects");
        let store = ObjectStore::new(&path);
        let now: DateTime<Local> = Local::now();
        self.date_time = now.format("%Y%m%d-%H%M").to_string();
        let path = self.destination_path.pushed("ntm.toml");
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let string = match String::from_utf8(bytes) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let result = toml::from_str(&string);
        let config: Config;
        if result.is_ok() {
            config = result.unwrap();
            // println!("config.source_path: {}", config.source_path);
        } else {
            config = Config::new();
        }
        self.bytes_id_threshold_min = match config.bytes_id_threshold_min {
            Some(min) => min,
            None => 0,
        };
        self.bytes_id_threshold_max = match config.bytes_id_threshold_max {
            Some(max) => max,
            None => 0,
        };
        self.excluded_directories = match config.excluded_directories {
            Some(directories) => directories,
            None => vec![],
        };

        let mut producer = FilePathProducer::new(&config.source_path);
        if self.excluded_directories.len() > 0 {
            producer.set_excluded_directories(&self.excluded_directories);
        }
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
                if let Err(error) = self.process_file(&path, &store, &config.source_path) {
                    println!("process_file() failed: error: {}", error);
                }
            }
        }

        println!("{} object(s) added.", self.added_count);

        Ok(())
    }

    fn process_file(
        &mut self,
        path: &String,
        store: &ObjectStore,
        source_path: &String,
    ) -> Result<()> {
        if self.count == 0 {
            println!(
                "Processing ({}, {}): {}",
                self.processed_count, self.added_count, path
            );
        }
        self.count += 1;
        self.count %= 100;
        let mut path_buf = PathBuf::new();
        path_buf.push(&source_path);
        path_buf.push(path.clone());
        let metadata = match path_buf.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
        };
        let mut bytes: Option<Vec<u8>> = None;
        let id: String;
        let file_size = metadata.len();
        if file_size >= (self.bytes_id_threshold_min as u64)
            && file_size <= (self.bytes_id_threshold_max as u64)
        {
            bytes = match fs::read(path_buf.clone()) {
                Ok(bytes) => Some(bytes),
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
            };
            let mut id_bytes = b"b,".to_vec();
            id_bytes = [id_bytes, bytes.clone().unwrap()].concat();
            // println!("id_bytes.len(): {}", id_bytes.len());
            id = object_id(&id_bytes);
        } else {
            let mut modified: u64 = 0;
            let result = metadata.modified();
            if result.is_ok() {
                let result = result.unwrap().duration_since(SystemTime::UNIX_EPOCH);
                if result.is_ok() {
                    modified = result.unwrap().as_secs();
                }
            }
            let string = format!(
                "p,{},{},{}",
                path_buf.to_string_lossy().to_string(),
                modified,
                file_size
            );
            id = object_id(&string.as_bytes().to_vec());
        }

        let exists = match store.exists(&id) {
            Ok(exists) => exists,
            Err(error) => return Err(error),
        };
        if !exists {
            if bytes.is_none() {
                bytes = match fs::read(path_buf.clone()) {
                    Ok(bytes) => Some(bytes),
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
                };
            }
            match store.add(&id, &bytes.unwrap()) {
                Ok(_) => (),
                Err(error) => return Err(error),
            };
            self.added_count += 1;
        }
        self.processed_count += 1;

        let mut entry_path = PathBuf::from(&self.destination_path);
        entry_path.push("Backups");
        entry_path.push(self.date_time.clone());
        entry_path.push(path.directories());
        match fs::create_dir_all(entry_path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED)),
        };
        entry_path.push(&path.file_name());
        // TODO: Set other fields.
        let entry = Entry {
            id: id,
            last_modified: 0,
            permission: 0,
            uid: 0,
            gid: 0,
        };
        let string = match serde_json::to_string(&entry) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED)),
        };
        match fs::write(entry_path, string.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED)),
        };

        Ok(())
    }

    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }
}

fn object_id(bytes: &Vec<u8>) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(bytes.clone());
    let hash = sha256.finalize();
    let hash_values = hash.to_vec();
    let hex = HexString::from_bytes(&hash_values);

    hex.as_string()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::commons::ConvertPath;
    use crate::backup_command::BackupCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = BackupCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
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
    }
}
