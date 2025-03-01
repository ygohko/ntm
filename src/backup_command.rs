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
    count: i32,
}

impl BackupCommand {
    pub fn new() -> Self {
        Self {
            count: 0,
        }
    }

    pub fn execute(&mut self) -> Result<()> {
        let store = ObjectStore::new(&"Objects");
        let now: DateTime<Local> = Local::now();
        let date_time = now.format("%Y%m%d-%H%M").to_string();
        let bytes = match fs::read("ntm.toml") {
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

        let mut producer = FilePathProducer::new(&config.source_path);
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
                match self.process_file(&path, &store, &config.source_path, &date_time) {
                    Ok(_) => (),
                    Err(error) => {
                        println!("process_file() failed: error: {}", error);

                        ()
                    },
                };
            }
        }

        Ok(())
    }

    fn process_file(&mut self, path: &String, store: &ObjectStore, source_path: &String, date_time: &String) -> Result<()> {
        self.count += 1;
        self.count %= 100;
        if self.count == 0 {
            println!("path: {}", path);
        }
        let mut path_buf = PathBuf::new();
        path_buf.push(&source_path);
        path_buf.push(path.clone());
        let metadata = match path_buf.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
        };
        let bytes = match fs::read(path_buf.clone()) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
        };
        let id: String;
        let file_size = metadata.len();
        if file_size < 100 * 1024 * 1024 {
            let mut id_bytes = b"b,".to_vec();
            id_bytes = [id_bytes, bytes.clone()].concat();
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
            let string = format!("p,{},{},{}", path_buf.to_string_lossy().to_string(), modified, file_size);
            id = object_id(&string.as_bytes().to_vec());
        }

        match store.add(&id, &bytes) {
            Ok(_) => (),
            Err(error) => return Err(error),
        };

        let mut entry_path = PathBuf::new();
        entry_path.push("Backups");
        entry_path.push(date_time.clone());
        entry_path.push(path.directories());
        match fs::create_dir_all(entry_path.clone()) {
            Ok(_) => (),
            Err(_) => {
                return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED))
            }
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
            Err(_) => {
                return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED))
            }
        };
        match fs::write(entry_path, string.as_bytes()) {
            Ok(_) => (),
            Err(_) => {
                return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED))
            }
        };

        Ok(())
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
    use std::env;
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let command = BackupCommand::new();
    }

    #[test]
    fn is_executable() {
        // TODO: Do not modofy current directory.
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

        env::set_current_dir(&previous_current_dir).unwrap();
    }
}
