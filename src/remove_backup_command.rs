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

use crate::backup_store::BackupStore;
use crate::error::Result;
use crate::task::Task;

/// A command to remove backups specified by pattern.
pub struct RemoveBackupCommand {
    pattern: String,
    destination_path: String,
}

impl Task for RemoveBackupCommand {
    fn execute(&mut self) -> Result<()> {
        let mut backup_path = Utf8PathBuf::from(&self.destination_path);
        backup_path.push("Backups");
        let store = BackupStore::new(backup_path.as_str());
        let names = store.names()?;

        for name in names {
            // TODO: Check wheather this backup matches pattern.
        }

        // TODO: Mark removed this backup.

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
