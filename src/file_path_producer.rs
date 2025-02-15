use std::env::consts;
use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "file_path_producer";

pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_PRODUCING_FINISHED: ErrorCode = 1;

pub struct FilePathProducer {
    file_paths: Vec<String>,
    directory_paths: Vec<String>,
    prefix_length: usize,
}

impl FilePathProducer {
    // TODO: This argument should be AsRef<Path>?
    pub fn new(path: &str) -> FilePathProducer {
        let prefix_length = path.len() + 1;
        return FilePathProducer {
            file_paths: Vec::new(),
            directory_paths: vec![path.to_string()],
            prefix_length: prefix_length,
        };
    }

    pub fn next(&mut self) -> Result<String> {
        let done = false;
        while !done {
            if self.file_paths.len() > 0 {
                let mut path = self.file_paths.pop().unwrap();
                if consts::OS == "windows" {
                    path = path.replace("\\", "/");
                }

                return Ok(path);
            }

            if self.directory_paths.len() == 0 {
                return Err(Error::new(ERROR_ID, ERROR_CODE_PRODUCING_FINISHED));
            }
            let directory_path = self.directory_paths.pop().unwrap();

            let mut scan = true;
            let option = Path::new(&directory_path).file_name();
            if option.is_some() {
                let file_name = option.unwrap().to_string_lossy().to_string();
                if file_name == "NTM".to_string() {
                    scan = false;
                }
            }

            if scan {
                let read_dir = match fs::read_dir(directory_path) {
                    Ok(read_dir) => read_dir,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL)),
                };
                for result in read_dir {
                    if result.is_ok() {
                        let entry = result.unwrap();
                        let metadata = match fs::metadata(entry.path()) {
                            Ok(metadata) => metadata,
                            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL)),
                        };
                        let path = entry.path().to_string_lossy().to_string();
                        if metadata.is_file() {
                            let path = path[self.prefix_length..].to_string();
                            self.file_paths.push(path);
                        } else {
                            self.directory_paths.push(path);
                        }
                    }
                }
            }
        }

        Err(Error::new(ERROR_ID, ERROR_CODE_PRODUCING_FINISHED))
    }
}
