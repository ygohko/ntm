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

use camino::Utf8PathBuf;
use std::fs;
use std::fs::DirEntry;
use std::thread;
use std::sync::Arc;
use std::sync::RwLock;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "backup_remover";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_DIRECTORY_FAILED: ErrorCode = 1;

struct Private {
    destination_path: String,
}

impl Private {
    fn new() -> Self {
        Self {
            destination_path: ".".to_string(),
        }
    }
}

pub struct BackupRemover {
    private: Arc<RwLock<Private>>,
}

impl Task for BackupRemover {
    fn execute(&mut self) -> Result<()> {
        let private = self.private.clone();
        let handle = thread::spawn(move || {
            main(&private);
        });
        handle.join();

        Ok(())
    }
}

impl BackupRemover {
    pub fn new() -> Self {
        Self {
            private: Arc::new(RwLock::new(Private::new())),
        }
    }

    pub fn set_destination_path(&mut self, destination_path: String) {
        let mut private = self.private.write().unwrap();
        private.destination_path = destination_path;
    }    
}

fn main(private :&Arc<RwLock<Private>>) -> Result<()> {
    let destination_path: String;
    {
        let private = private.read().unwrap();
        destination_path = private.destination_path.clone();
    }
    let mut path = Utf8PathBuf::from(&destination_path);
    path.push("Backups");
    
    // TODO: Iterate backups.
    let Ok(read_dir) = fs::read_dir(&path) else {
        return Err(Error::new(ERROR_ID, ERROR_CODE_READING_DIRECTORY_FAILED));
    };
    for result in read_dir {
        if let Ok(dir_entry) = result {
            process_dir_entry(private, &dir_entry)?;
        }
    }
    
    // TODO: If backup is marked, do recursive remove.

    Ok(())
}

fn process_dir_entry(private: &Arc<RwLock<Private>>, dir_entry: &DirEntry) -> Result<()> {
    // TODO: Imprement this.
    Ok(())
}
