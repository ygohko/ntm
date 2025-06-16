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

use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "entry_saver";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_WRITING_ENTRY_FAILED: ErrorCode = 1;

pub struct EntrySaver {
    entry: Entry,
    path: String,
}

impl Task for EntrySaver {
    fn execute(&mut self) -> Result<()> {
        let path = Utf8PathBuf::from(&self.path);
        let parent = path.parent_or_empty();
        if let Err(_) = fs::create_dir_all(&parent) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ENTRY_FAILED));
        }
        let string = match serde_json::to_string(&self.entry) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ENTRY_FAILED)),
        };
        if let Err(_) = fs::write(&path, string.as_bytes()) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ENTRY_FAILED));
        }

        Ok(())
    }
}

impl EntrySaver {
    pub fn new(entry: &Entry, path: &str) -> Self {
        Self {
            entry: entry.clone(),
            path: path.to_string(),
        }
    }
}
