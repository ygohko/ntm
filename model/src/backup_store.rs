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

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::commons::OperatePath;

pub const ERROR_ID: ErrorId = "backup_store";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_FINDING_BACKUP_FAILED: ErrorCode = 1;
pub const ERROR_CODE_MARKING_AS_REMOVED_FAILED: ErrorCode = 2;

/// Manages backup directories.
pub struct BackupStore {
    path: String,
}

impl BackupStore {
    /// Creates a new `BackupStore` instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path where backups are stored.
    ///
    /// # Returns
    ///
    /// * `BackupStore` - A new `BackupStore` instance.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    /// Returns a list of backup names (directory names) in the store.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `String`s representing backup names,
    /// or an `Error` if the operation fails.
    pub fn names(&self) -> Result<Vec<String>> {
        let Ok(read_dir) = fs::read_dir(&self.path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_FINDING_BACKUP_FAILED));
        };
        let mut names: Vec<String> = Vec::new();
        for result in read_dir {
            if let Ok(entry) = result {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let name = entry.path().file_name_or_empty();
                        if !name.ends_with(".removed") {
                            names.push(name);
                        }
                    }
                }
            }
        }

        Ok(names)
    }

    /// Marks a backup as removed.
    ///
    /// Renames the backup directory to `<name>.removed`. Marked backups are no
    /// longer returned by `names()`, and are deleted later by `BackupRemover`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the backup to be marked.
    ///
    /// # Errors
    ///
    /// * `Error::new(ERROR_ID, ERROR_CODE_MARKING_AS_REMOVED_FAILED)`: If renaming
    ///   the backup directory fails (e.g., the backup does not exist).
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the backup is marked successfully, or an
    /// `Error` if the operation fails.
    pub fn mark_as_removed(&self, name: &str) -> Result<()> {
        let mut from_path = Utf8PathBuf::from(&self.path);
        from_path.push(name);
        let mut to_path = Utf8PathBuf::from(&self.path);
        let mut removed_name = name.to_string();
        removed_name.push_str(".removed");
        to_path.push(removed_name);
        if let Err(_) = fs::rename(&from_path, &to_path) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_MARKING_AS_REMOVED_FAILED));
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_store;
    use crate::backup_store::BackupStore;
    use crate::commons::OperatePath;

    #[test]
    fn is_creatable() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        fs::create_dir_all(&path).unwrap();
        let _store = BackupStore::new(&path.to_string_easy());
    }

    #[test]
    fn names_are_gettable() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        let mut backup_path = path.clone();
        backup_path.push("11111111-1111");
        fs::create_dir_all(&backup_path).unwrap();
        let mut backup_path = path.clone();
        backup_path.push("22222222-2222");
        fs::create_dir_all(&backup_path).unwrap();
        let mut backup_path = path.clone();
        backup_path.push("33333333-3333");
        fs::create_dir_all(&backup_path).unwrap();
        let store = BackupStore::new(&path.to_string_easy());
        let names = store.names().unwrap();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"11111111-1111".to_string()));
        assert!(names.contains(&"22222222-2222".to_string()));
        assert!(names.contains(&"33333333-3333".to_string()));
    }

    #[test]
    fn backup_is_markable_as_removed() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        let mut backup_path = path.clone();
        backup_path.push("11111111-1111");
        fs::create_dir_all(&backup_path).unwrap();
        let mut backup_path = path.clone();
        backup_path.push("22222222-2222");
        fs::create_dir_all(&backup_path).unwrap();
        let store = BackupStore::new(&path.to_string_easy());
        store.mark_as_removed("11111111-1111").unwrap();
        assert!(!path.join("11111111-1111").exists());
        assert!(path.join("11111111-1111.removed").is_dir());
        let names = store.names().unwrap();
        assert_eq!(names, vec!["22222222-2222".to_string()]);
    }

    #[test]
    fn marking_nonexistent_backup_as_removed_fails() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        fs::create_dir_all(&path).unwrap();
        let store = BackupStore::new(&path.to_string_easy());
        let error = store.mark_as_removed("11111111-1111").unwrap_err();
        assert_eq!(error.id, backup_store::ERROR_ID);
        assert_eq!(error.code, backup_store::ERROR_CODE_MARKING_AS_REMOVED_FAILED);
        assert!(!path.join("11111111-1111.removed").exists());
    }
}
