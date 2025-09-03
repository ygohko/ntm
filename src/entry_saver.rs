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
use crate::task;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "entry_saver";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_WRITING_ENTRY_FAILED: ErrorCode = 1;

/// A task for saving an entry to a file.
pub struct EntrySaver {
    entry: Entry,
    path: String,
}

impl Task for EntrySaver {
    /// Executes the entry saving task.
    ///
    /// This method serializes the entry and writes it to the specified file path.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
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

    /// Attempts to execute a task or operation in the background.
    ///
    /// **Note:** This method currently returns an error, explicitly indicating that
    /// background execution is not supported by the current implementation.
    /// It serves as a placeholder or an unimplemented feature.
    ///
    /// # Errors
    ///
    /// This method will always return an `Err(Error)` with `task::ERROR_ID` and
    /// `task::ERROR_CODE_NOT_SUPPORTED`, as background execution is not
    /// currently implemented or supported.
    ///
    /// # Returns
    ///
    /// A `Result<()>` which will always be `Err` in the current implementation,
    /// signalling the lack of background execution support.
    fn execute_in_background(&mut self) -> Result<()> {
        Err(Error::new(task::ERROR_ID, task::ERROR_CODE_NOT_SUPPORTED))
    }

    /// Joins the current execution context with this task, waiting for the task to complete.
    ///
    /// **Note:** This method is currently not supported and will always return an error indicating
    /// `NOT_SUPPORTED`.
    ///
    /// # Returns
    ///
    /// An `Err` containing an `Error` with `ERROR_ID` and `ERROR_CODE_NOT_SUPPORTED`, as this
    /// operation is not implemented.
    fn join(&mut self) -> Result<()> {
        Err(Error::new(task::ERROR_ID, task::ERROR_CODE_NOT_SUPPORTED))
    }
}

impl EntrySaver {
    /// Creates a new `EntrySaver` instance.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to be saved.
    /// * `path` - The file path where the entry will be saved.
    ///
    /// # Returns
    ///
    /// * `EntrySaver` - A new `EntrySaver` instance.
    pub fn new(entry: &Entry, path: &str) -> Self {
        Self {
            entry: entry.clone(),
            path: path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::entry::Entry;
    use crate::entry_saver::EntrySaver;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let temp_dir = TempDir::new("test").unwrap();
        let mut temp_path = temp_dir.path().to_path_buf();
        temp_path.push("a.txt");
        let entry = Entry {
            id: "01abcdef".to_string(),
            last_modified: 12345,
            permission: 0o777,
            uid: 1000,
            gid: 1000,
        };
        let _saver = EntrySaver::new(&entry, &temp_path.to_string_easy());
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let mut temp_path = temp_dir.path().to_path_buf();
        temp_path.push("a.txt");
        let entry = Entry {
            id: "01abcdef".to_string(),
            last_modified: 12345,
            permission: 0o777,
            uid: 1000,
            gid: 1000,
        };
        let mut saver = EntrySaver::new(&entry, &temp_path.to_string_easy());
        saver.execute().unwrap();
    }
}
