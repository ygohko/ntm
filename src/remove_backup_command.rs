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

        let mut pattern = regex::escape(&self.pattern);
        pattern = pattern.replace("*", ".*");
        pattern = pattern.replace("?", ".");
        let Ok(re) = Regex::new(&pattern) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_REGULAR_EXPRESSION));
        };
        for name in names {
            if re.is_match(&name) {
                let mut from_path = Utf8PathBuf::from(&self.destination_path);
                from_path.push("Backups");
                from_path.push(&name);
                let mut to_path = Utf8PathBuf::from(&self.destination_path);
                to_path.push("Backups");
                let mut removed_name = name.clone();
                removed_name.push_str(".removed");
                to_path.push(&removed_name);
                match fs::rename(&from_path, &to_path) {
                    Ok(()) => {
                        println!("Marked as removed: {}", name);
                    },
                    Err(error) => {
                        println!(
                            "Error: Could not mark to removed {}. error: {}",
                            from_path, error
                        );
                    },
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
            destination_path: ".".to_string(),
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

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::OperatePath;
    use crate::init_command::InitCommand;
    use crate::remove_backup_command::RemoveBackupCommand;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _command = RemoveBackupCommand::new("*");
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = &temp_dir.path().to_path_buf();
        let mut source_path = temp_path.clone();
        source_path.push("source");
        fs::create_dir_all(&source_path).unwrap();
        let mut file_path = source_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "ABCDE").unwrap();

        let mut ntm_path = temp_path.clone();
        ntm_path.push("ntm");
        fs::create_dir_all(&ntm_path).unwrap();
        let mut command = InitCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut command = RemoveBackupCommand::new("*");
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();
    }
}
