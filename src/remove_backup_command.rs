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
use regex::Regex;
use std::fs;

use crate::backup_store::BackupStore;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "remove_backup_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_INVALID_REGULAR_EXPRESSION: ErrorCode = 1;

/// A command to remove backups specified by pattern.
pub struct RemoveBackupCommand {
    pattern: String,
    destination_path: String,
}

impl Task for RemoveBackupCommand {
    /// Marks backup files for removal by renaming them to append a ".removed" suffix,
    /// based on a user-defined pattern.
    ///
    /// This method first constructs the path to the backup store by appending "Backups"
    /// to the `destination_path`. It then retrieves all existing backup names from this store.
    ///
    /// The `self.pattern` string is interpreted as a wildcard pattern where:
    /// - `*` matches any sequence of characters (including an empty sequence).
    /// - `?` matches any single character.
    /// This wildcard pattern is converted into a regular expression for matching against backup names.
    ///
    /// # Returns
    ///
    /// A result may contain an error.
    fn execute(&mut self) -> Result<()> {
        let mut backup_path = Utf8PathBuf::from(&self.destination_path);
        backup_path.push("Backups");
        let store = BackupStore::new(backup_path.as_str());
        let names = store.names()?;

        let mut pattern = self.pattern.replace("*", ".*");
        pattern = pattern.replace("?", ".");
        let Ok(re) = Regex::new(&pattern) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_REGULAR_EXPRESSION));
        };
        for name in names {
            if re.is_match(&name) {
                println!("Pattern matched. name: {}", name);
                let mut from_path = Utf8PathBuf::from(&self.destination_path);
                from_path.push("Backups");
                from_path.push(&name);

                let mut to_path = Utf8PathBuf::from(&self.destination_path);
                to_path.push("Backups");
                let mut removed_name = name.clone();
                removed_name.push_str(".removed");
                to_path.push(&removed_name);
                if let Err(error) = fs::rename(&from_path, &to_path) {
                    println!("Error: Could not mark to removed {}. error: {}", from_path, error);
                };
            }
        }

        Ok(())
    }
}

impl RemoveBackupCommand {
    /// Creates a new instance of the struct with the given pattern.
    ///
    /// Initializes the `pattern` field by converting the input string slice into an owned `String`.
    /// The `destination_path` field is initialized as an empty string, implying it might be set
    /// later or is optional in the initial state.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A string slice (`&str`) representing the pattern that this instance will use.
    ///
    /// # Returns
    ///
    /// A new instance of `Self` with the provided `pattern` and an empty `destination_path`.
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            destination_path: "".to_string(),
        }
    }

    /// Sets the destination path for the operation.
    ///
    /// This method updates the `destination_path` field of the struct with the provided path.
    /// The path is converted to an owned `String`.
    ///
    /// # Arguments
    ///
    /// * `destination_path` - A string slice representing the new destination path.
    pub fn set_destination_path(&mut self, destination_path: &str) {
        self.destination_path = destination_path.to_string();
    }
}
