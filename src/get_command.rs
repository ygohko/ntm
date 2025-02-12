use std::fs;
use std::path;
use std::path::PathBuf;

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
pub const ERROR_CODE_READING_REFERENCE_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_BYTES_FAILED: ErrorCode = 3;

pub struct GetCommand {
    backup: String,
}

impl GetCommand {
    // TODO: Add path option.
    pub fn new(backup: &str) -> Self {
        GetCommand {
            backup: backup.to_string(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        // TODO: Implement this.
        let store = ObjectStore::new(&"NTM/Objects");
        let mut backup_path = PathBuf::new();
        backup_path.push("NTM/Backups/");
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

        let mut producer = FilePathProducer::new(&path.to_string_lossy());
        let mut done = false;
        while !done {
            let path = match producer.next() {
                Ok(path) => path,
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED {
                        done = true;
                    }
                    else {
                        return Err(error);
                    }

                    "".to_string()
                },
            };

            if !done {
                let mut entry_path = PathBuf::new();
                entry_path.push(&backup_path);
                entry_path.push(&path);
                println!("entry_path: {}", entry_path.display());
                let bytes = match fs::read(entry_path.clone()) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_REFERENCE_FAILED)),
                };
                let id = match String::from_utf8(bytes) {
                    Ok(id) => id,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_REFERENCE_FAILED)),
                };
                println!("id: {}", id);

                let bytes = match store.bytes(&id) {
                    Ok(bytes) => bytes,
                    // TODO: Skipping file that object is not found may be needed.
                    Err(error) => return Err(error),
                };
                let mut destination_path = PathBuf::new();
                destination_path.push(&self.backup);
                destination_path.push(&path);
                println!("destination_path: {}", destination_path.display());
                let directries = directories_from_path(&destination_path.to_string_lossy().to_string());
                match fs::create_dir_all(&directries) {
                    Ok(_) => (),
                    // TODO: Skipping file that writing is failed may be needed.
                    Err(_) => return  Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                }                
                match fs::write(destination_path, bytes) {
                    Ok(_) => (),
                    // TODO: Skipping file that writing is failed may be needed.
                    Err(_) => return  Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                };
            }
        }

        Ok(())
    }
}

// TODO: Move to commons.
fn directories_from_path(path: &str) -> String {
    let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
    if split.len() < 1 {
        return "".to_string();
    }
    split.pop();

    split.join(path::MAIN_SEPARATOR_STR)
}
