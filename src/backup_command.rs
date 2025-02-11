use chrono::DateTime;
use chrono::Local;
use hex_string::HexString;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::path;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Error;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

pub const ERROR_ID: &'static str = "backup_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: i32 = 0;
pub const ERROR_CODE_READING_CONFIG_FAILED: i32 = 1;
pub const ERROR_CODE_READING_SOURCE_FAILED: i32 = 2;
pub const ERROR_CODE_WRITING_DESTINATION_FAILED: i32 = 3;

pub struct BackupCommand {}

impl BackupCommand {
    pub fn new() -> Self {
        BackupCommand {}
    }

    pub fn execute(&self) -> Result<(), Error> {
        let store = ObjectStore::new(&"NTM/Objects");
        let now: DateTime<Local> = Local::now();
        let date_time = now.format("%Y%m%d%H%M%S").to_string();
        let bytes = match fs::read("NTM/config.toml") {
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
            println!("config.source_path: {}", config.source_path);
        }
        else {
            config = Config::new();
        }

        let mut producer = FilePathProducer::new(config.source_path.clone());
        let mut done = false;
        while !done {
            let path = match producer.next() {
                Ok(path) => path,
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED {
                        done = true;
                    } else {
                        return Err(error);
                    }

                    "".to_string()
                }
            };

            if !done {
                println!("path: {}", path);
                let mut path_buf = PathBuf::new();
                path_buf.push(&config.source_path);
                path_buf.push(path.clone());

                let bytes = match fs::read(path_buf) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
                };

                let mut id_bytes = b"b,".to_vec();
                id_bytes = [id_bytes, bytes.clone()].concat();
                println!("id_bytes.len(): {}", id_bytes.len());

                let id = object_id(&id_bytes);
                match store.add(&id, &bytes) {
                    Ok(_) => (),
                    Err(error) => return Err(error),
                };

                let mut reference_path = PathBuf::new();
                reference_path.push("NTM/Backups");
                reference_path.push(date_time.clone());
                reference_path.push(reference_directories(&path));
                match fs::create_dir_all(reference_path.clone()) {
                    Ok(_) => (),
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED)),
                };
                reference_path.push(reference_file(&path));

                println!("reference_path: {}", reference_path.display());

                match fs::write(reference_path, id) {
                    Ok(_) => (),
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_DESTINATION_FAILED)),
                };
            }
        }

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

fn reference_directories(path: &str) -> String {
    let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
    if split.len() < 1 {
        return "".to_string();
    }
    split.pop();

    split.join(path::MAIN_SEPARATOR_STR)
}

fn reference_file(path: &str) -> String {
    let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
    if split.len() < 1 {
        return "".to_string();
    }

    split.pop().unwrap().to_string()
}
