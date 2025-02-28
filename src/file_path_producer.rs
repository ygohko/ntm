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
                        let mut is_file = false;
                        let mut is_dir = false;
                        match fs::symlink_metadata(entry.path()) {
                            Ok(metadata) => {
                                is_file = metadata.is_file();
                                is_dir = metadata.is_dir() && !metadata.is_symlink();
                            },
                            Err(error) => {
                                println!("error: {}", error);
                            },
                        };
                        let path = entry.path().to_string_lossy().to_string();
                        if is_file {
                            let path = path[self.prefix_length..].to_string();
                            self.file_paths.push(path);
                        } else if is_dir {
                            self.directory_paths.push(path);
                        }
                    }
                }
            }
        }

        Err(Error::new(ERROR_ID, ERROR_CODE_PRODUCING_FINISHED))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::commons::ConvertPath;
    use crate::file_path_producer;
    use crate::file_path_producer::FilePathProducer;

    #[test]
    fn is_creatable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = String::from_path(&temp_dir.path());
        let _producer = FilePathProducer::new(&path);
    }

    #[test]
    fn paths_are_producable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = String::from_path(&temp_dir.path());
        let path = path.pushed("a.txt");
        let Ok(_) = fs::write(&path, "ABCDE") else {
            panic!();
        };
        let path = String::from_path(&temp_dir.path());
        let path = path.pushed("b");
        let Ok(_) = fs::create_dir_all(&path) else {
            panic!();
        };
        let path = path.pushed("c.txt");
        let Ok(_) = fs::write(&path, "FGHIJ") else {
            panic!();
        };
        let path = String::from_path(&temp_dir.path());
        let mut producer = FilePathProducer::new(&path);
        let Ok(path) = producer.next() else {
            panic!();
        };
        assert_eq!(path, "a.txt".to_string());
        let Ok(path) = producer.next() else {
            panic!();
        };
        assert_eq!(path, "b/c.txt".to_string());
        let Err(error) = producer.next() else {
            panic!();
        };
        assert_eq!(error.id, file_path_producer::ERROR_ID);
        assert_eq!(error.code, file_path_producer::ERROR_CODE_PRODUCING_FINISHED);
    }
}
