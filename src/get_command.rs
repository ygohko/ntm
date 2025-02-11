use std::fs;
use std::path::PathBuf;

use crate::error;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "get_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;
pub const ERROR_CODE_READING_REFERENCE_FAILED: ErrorCode = 2;

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

        let mut path = PathBuf::new();
        path.push("NTM/Backups/");
        path.push(&self.backup);
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
                let bytes = match fs::read(path) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_REFERENCE_FAILED)),
                };
                let id = match String::from_utf8(bytes) {
                    Ok(id) => id,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_REFERENCE_FAILED)),
                };
                println!("id: {}", id);
            }
        }

        Err(Error::new(error::ERROR_ID, error::ERROR_CODE_NOT_IMPLEMENTED))
    }
}
