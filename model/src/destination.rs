/*
 * Copyright (c) 2026 Yasuaki Gohko
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

use crate::backup_store::BackupStore;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "destination";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 1;
pub const ERROR_CODE_CREATING_FILE_FAILED: ErrorCode = 1;

/// A task for initializing backup destination
pub struct Destination {
    path: String,
}

impl Destination {
    /// Creates a new `Destination` instance for an existing destination.
    ///
    /// This does not touch the file system. Use `Destination::init()` to create
    /// the directories of a new destination.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the backup destination.
    ///
    /// # Returns
    ///
    /// A new `Destination` instance.
    pub fn new(path: &str) -> Self {
        Destination {
            path: path.to_string(),
        }
    }

    /// Initializes a backup destination and returns a `Destination` instance for it.
    ///
    /// Creates the `Backups` and `Objects` directories under `path`. Existing
    /// directories are left as they are.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the backup destination.
    /// * `source_path` - The path to the backup source.
    ///
    /// # Errors
    ///
    /// * `Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)`: If creating
    ///   a directory fails.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `Destination` instance, or an `Error` if the
    /// initialization fails.
    pub fn init(path: &str, source_path: Option<String>) -> Result<Self> {
        let config1: &str = r#"
source_path = ""#.trim_start();
        let config2: &str = r#"
"
# excluded_directories = ["/path/to/excluded/directories",]
# cache_size = 10
"#.trim_start();
        let source_path = match source_path {
            Some(source_path) => source_path,
            None => "/path/to/source".to_string(),
        };
        let config = config1.to_string() + &source_path + config2;

        let mut backup_path = Utf8PathBuf::from(&path);
        backup_path.push("Backups");
        match fs::create_dir_all(&backup_path) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        let mut object_path = Utf8PathBuf::from(&path);
        object_path.push("Objects");
        match fs::create_dir_all(&object_path) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        let mut config_path = Utf8PathBuf::from(&path);
        config_path.push("ntm.toml");
        match fs::write(&config_path, config) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_FILE_FAILED))
        };

        let result = Self {
            path: path.to_string(),
        };

        Ok(result)
    }

    /// Returns a `BackupStore` for the `Backups` directory of the destination.
    ///
    /// # Returns
    ///
    /// A new `BackupStore` instance.
    pub fn backup_store(&self) -> BackupStore {
        let mut path = Utf8PathBuf::from(&self.path);
        path.push("Backups");
        let store = BackupStore::new(path.as_str());

        store
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::destination::Destination;

    #[test]
    fn is_creatable() {
        let _destination = Destination::new("NTM");
    }

    #[test]
    fn is_initializable() {
        let temp_dir = TempDir::new("test").unwrap();
        Destination::init(&temp_dir.path().to_string_easy()).unwrap();
    }
}
