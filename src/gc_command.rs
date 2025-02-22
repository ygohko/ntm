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
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

pub const ERROR_ID: ErrorId = "gc_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_FINDING_BACKUP_FAILED: ErrorCode = 1;
pub const ERROR_CODE_PROCESSING_ENTRY_FAILED: ErrorCode = 2;

pub struct GcCommand {
    store: ObjectStore,
}

impl GcCommand {
    pub fn new() ->Self {
        Self {
            store: ObjectStore::new(&"Objects"),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let backup_paths = match backup_paths() {
            Ok(backup_paths) => backup_paths,
            Err(error) => return Err(error),
        };
        for path in backup_paths {
            println!("path: {}", path);
            if let Err(error) = self.process_backup(&path) {
                println!("Processing backup {} failed. error: {}", path, error);
            }
        }
        self.store.sweep()?;

        Ok(())
    }

    fn process_backup(&self, path: &str) -> Result<()> {
        let mut producer = FilePathProducer::new(&path);
        let mut done = false;
        while !done {
            let option = match producer.next() {
                Ok(path) => Some(path),
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED {
                        done = true;
                    }
                    // TODO: Displaying errors would be needed.

                    None
                },
            };
            if let Some(path) = option {
                if let Err(error) = self.process_entry(&path) {
                    println!("Processing entry {} failed. error: {}", path, error);
                }
            }
        }
        
        Ok(())
    }
    
    fn process_entry(&self, path: &str) -> Result<()> {
        let string = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_ENTRY_FAILED)),
        };
        let entry: Entry = match serde_json::from_str(&string) {
            Ok(entry) => entry,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_PROCESSING_ENTRY_FAILED)),
        };
        if let Err(error) = self.store.mark(&entry.id) {
            return Err(error);
        }
   
        Ok(())
    }
}

fn backup_paths() -> Result<Vec<String>> {
    let read_dir = match fs::read_dir("Backups") {
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
