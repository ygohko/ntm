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
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "gc_command";

pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_FINDING_BACKUP_FAILED: ErrorCode = 1;

pub struct GcCommand {
}

impl GcCommand {
    pub fn new() ->Self {
        Self {
        }
    }

    pub fn execute(&self) -> Result<()> {
        // TODO: Iterate backup entries.
        /*
        let path = PathBuf::from("Backups");
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
        */
        let backup_paths = match backup_paths() {
            Ok(backup_paths) => backup_paths,
            Err(error) => return Err(error),
        };
        for path in backup_paths {
            println!("path: {}", path);
        }

        // TODO: Mark object files.
        // TODO: Remove not marked objects.
        // TODO: Remove mark files.

        Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL))
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
