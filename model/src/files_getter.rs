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
#[cfg(not(target_os = "windows"))]
use std::fs::File;
#[cfg(not(target_os = "windows"))]
use std::ops::Add;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs as unix_fs;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
#[cfg(not(target_os = "windows"))]
use std::time::Duration;
#[cfg(not(target_os = "windows"))]
use std::time::SystemTime;

use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorId;
use crate::error::ErrorCode;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "files_getter";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;
pub const ERROR_CODE_READING_ENTRY_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_BYTES_FAILED: ErrorCode = 3;
pub const ERROR_CODE_WRITING_METADATA_FAILED: ErrorCode = 4;

/// The task to get files from backups.
pub struct FilesGetter {
    name: String,
    extraction_path: String,
    destination_path: String,
    gotten_path: String,
    processed_count: i64,
    count: i32,
}

impl Task for FilesGetter {
    /// Gets files from the backup and writes them under the gotten path.
    ///
    /// The files are written to `<gotten_path>/<name>/<extraction_path>`, keeping
    /// their relative paths in the backup. If the extraction path is empty, all
    /// files in the backup are gotten. If it points to a directory, the files under
    /// the directory are gotten. If it points to a file, only the file is gotten.
    ///
    /// A file whose object cannot be read from the object store is skipped.
    /// Failing to apply the metadata (modification time, permission and owner) to
    /// a gotten file does not stop the processing either.
    ///
    /// # Errors
    ///
    /// * `Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)`: If the backup does
    ///   not exist.
    /// * `Error::new(ERROR_ID, ERROR_CODE_GENERAL)`: If the extraction path does
    ///   not exist in the backup.
    /// * `Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)`: If reading or
    ///   parsing an entry fails.
    /// * `Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)`: If creating a
    ///   directory or writing a file under the gotten path fails.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the execution fails.
    fn execute(&mut self) -> Result<()> {
        // TODO: Return error if path is invalid.
        let mut path = Utf8PathBuf::from(&self.destination_path);
        path.push("Objects");
        let store = ObjectStore::new(&path.to_string_easy());
        let mut backup_path = Utf8PathBuf::from(&self.destination_path);
        backup_path.push("Backups");
        backup_path.push(&self.name);
        let mut path = Utf8PathBuf::new();
        path.push(&backup_path);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)),
        };
        if !exists {
            return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND));
        }

        let mut is_file = false;
        if self.extraction_path != "".to_string() {
            let mut path = backup_path.clone();
            path.push(&self.extraction_path);
            is_file = Self::is_file(&path)?;
        }
        if is_file {
            self.process_file(&store, &backup_path.to_string_easy())?;
        } else {
            self.process_directory(&store, &backup_path.to_string_easy())?;
        }

        Ok(())
    }
}

impl FilesGetter {
    /// Creates a new `FilesGetter` instance.
    ///
    /// This does not touch the file system. Call `execute()` to get the files.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the backup to get files from (e.g. `20261102-0900`).
    /// * `extraction_path` - The path of a file or a directory in the backup to be
    ///   gotten, relative to the backup root. An empty string means the whole
    ///   backup.
    /// * `destination_path` - The path to the backup destination.
    /// * `gotten_path` - The path of the directory to write the gotten files to.
    ///
    /// # Returns
    ///
    /// A new `FilesGetter` instance.
    pub fn new(name: &str, extraction_path: &str, destination_path: &str, gotten_path: &str) -> Self {
        Self {
            name: name.to_string(),
            extraction_path: extraction_path.to_string(),
            destination_path: destination_path.to_string(),
            gotten_path: gotten_path.to_string(),
            processed_count: 0,
            count: 0,
        }
    }

    fn process_directory(&mut self, store: &ObjectStore, backup_path: &str) -> Result<()> {
        let mut path = Utf8PathBuf::from(&backup_path);
        if self.extraction_path != "".to_string() {
            path.push(&self.extraction_path);
        }
        let mut producer = FilePathProducer::new(&path.to_string_easy());
        let mut done = false;
        while !done {
            let path = match producer.next() {
                Ok(path) => path,
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID
                        && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED
                    {
                        done = true;
                    } else {
                        return Err(error);
                    }

                    "".to_string()
                }
            };

            if !done {
                let mut entry_path = Utf8PathBuf::new();
                entry_path.push(&backup_path);
                if self.extraction_path != "".to_string() {
                    entry_path.push(&self.extraction_path);
                }
                entry_path.push(&path);
                if self.count == 0 {
                    println!("Processing ({}): {}", self.processed_count, entry_path);
                }
                self.count += 1;
                self.count %= 100;
                let string = match fs::read_to_string(entry_path.clone()) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
                };

                let entry: Entry = match serde_json::from_str(&string) {
                    Ok(entry) => entry,
                    Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
                };

                let result = store.bytes(&entry.id);
                if let Err(ref error) = result {
                    println!("store.bytes() failed. error: {}", error);
                }
                if let Ok(bytes) = result {
                    let mut gotten_path = Utf8PathBuf::from(&self.gotten_path);
                    gotten_path.push(&self.name);
                    if self.extraction_path != "".to_string() {
                        gotten_path.push(&self.extraction_path);
                    }
                    gotten_path.push(&path);
                    let directories = gotten_path.parent_or_empty();
                    match fs::create_dir_all(&directories) {
                        Ok(_) => (),
                        // TODO: Skipping file that writing is failed may be needed.
                        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                    }
                    match fs::write(&gotten_path, bytes) {
                        Ok(_) => (),
                        // TODO: Skipping file that writing is failed may be needed.
                        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
                    };
                    if let Err(error) = Self::apply_metadata(&gotten_path.to_string_easy(), &entry) {
                        println!("apply_metadata() failed: error: {}", error);
                    }

                    self.processed_count += 1;
                }
            }
        }
        println!("{} file(s) gotten.", self.processed_count);

        Ok(())
    }
    
    fn process_file(&mut self, store: &ObjectStore, backup_path: &str) -> Result<()> {
        let mut entry_path = Utf8PathBuf::from(&backup_path);
        entry_path.push(&self.extraction_path);
        let string = match fs::read_to_string(entry_path.clone()) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
        };
        let entry: Entry = match serde_json::from_str(&string) {
            Ok(entry) => entry,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ENTRY_FAILED)),
        };

        let result = store.bytes(&entry.id);
        if let Err(ref error) = result {
            println!("store.bytes() failed. error: {}", error);
        }
        if let Ok(bytes) = result {
            let mut gotten_path = Utf8PathBuf::from(&self.gotten_path);
            gotten_path.push(&self.name);
            gotten_path.push(&self.extraction_path);
            let directories = gotten_path.parent_or_empty();
            match fs::create_dir_all(&directories) {
                Ok(_) => (),
                // TODO: Skipping file that writing is failed may be needed.
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
            }
            match fs::write(&gotten_path, bytes) {
                Ok(_) => (),
                // TODO: Skipping file that writing is failed may be needed.
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_BYTES_FAILED)),
            };
            if let Err(error) = Self::apply_metadata(&gotten_path.to_string_easy(), &entry) {
                println!("apply_metadata() failed: error: {}", error);
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn apply_metadata(path: &str, entry: &Entry) -> Result<()> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
        };
        let mut modified = SystemTime::UNIX_EPOCH;
        modified = modified.add(Duration::from_secs(entry.last_modified));
        if let Err(_) = file.set_modified(modified) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
        }

        if entry.permission != 0 {
            let metadata = match fs::metadata(path) {
                Ok(metadata) => metadata,
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
            };
            let mut permissions = metadata.permissions();
            permissions.set_mode(entry.permission);
            if let Err(_) = fs::set_permissions(path, permissions) {
                return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
            }
        }

        if entry.uid != 0 && entry.gid != 0 {
            if let Err(_) = unix_fs::chown(path, Some(entry.uid), Some(entry.gid)) {
                return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn apply_metadata(path: &str, entry: &Entry) -> Result<()> {
        if entry.permission != 0 {
            let metadata = match fs::metadata(path) {
                Ok(metadata) => metadata,
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
            };
            let mut permissions = metadata.permissions();
            let readonly: bool;
            if (entry.permission & 0o200) != 0 {
                readonly = false;
            } else {
                readonly = true;
            }
            permissions.set_readonly(readonly);
        }

        Ok(())
    }

    fn is_file(path: impl AsRef<Path>) -> Result<bool> {
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL)),
        };
        if metadata.is_file() {
            return Ok(true);
        }
        if metadata.is_dir() {
            return Ok(false);
        }

        Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::attributes::Attributes;
    use crate::commons::OperatePath;
    use crate::entry::Entry;
    use crate::files_getter;
    use crate::files_getter::FilesGetter;
    use crate::object_store::ObjectStore;
    use crate::task::Task;

    const BACKUP_NAME: &str = "11111111-1111";

    /// Creates a destination with a backup containing `files`. Each file is a
    /// tuple of the relative path, the object ID and the content.
    fn create_destination(path: &Path, files: &[(&str, &str, &[u8])]) {
        let mut store = ObjectStore::new(&path.join("Objects").to_string_easy());
        let backup_path = path.join("Backups").join(BACKUP_NAME);
        for (file_path, id, bytes) in files {
            store.add(id, &bytes.to_vec(), &Attributes::new(file_path, 0)).unwrap();
            write_entry(&backup_path, file_path, id);
        }
    }

    fn write_entry(backup_path: &Path, file_path: &str, id: &str) {
        let entry = Entry {
            id: id.to_string(),
            last_modified: 1_000_000_000,
            permission: 0,
            uid: 0,
            gid: 0,
        };
        let entry_path = backup_path.join(file_path);
        fs::create_dir_all(entry_path.parent().unwrap()).unwrap();
        fs::write(&entry_path, serde_json::to_string(&entry).unwrap()).unwrap();
    }

    fn get(destination_path: &Path, extraction_path: &str, gotten_path: &Path) -> crate::error::Result<()> {
        let mut getter = FilesGetter::new(
            BACKUP_NAME,
            extraction_path,
            &destination_path.to_string_easy(),
            &gotten_path.to_string_easy(),
        );
        getter.execute()
    }

    fn gotten_file(gotten_path: &Path, file_path: &str) -> PathBuf {
        gotten_path.join(BACKUP_NAME).join(file_path)
    }

    const FILES: [(&str, &str, &[u8]); 3] = [
        ("a.txt", "0101010101010101", b"a"),
        ("dir/b.txt", "0202020202020202", b"b"),
        ("dir/sub/c.txt", "0303030303030303", b"c"),
    ];

    #[test]
    fn whole_backup_is_gettable() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);

        get(&destination_path, "", &gotten_path).unwrap();

        for (file_path, _, bytes) in FILES {
            assert_eq!(fs::read(gotten_file(&gotten_path, file_path)).unwrap(), bytes);
        }
    }

    #[test]
    fn directory_is_gettable() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);

        get(&destination_path, "dir", &gotten_path).unwrap();

        assert!(!gotten_file(&gotten_path, "a.txt").exists());
        assert_eq!(fs::read(gotten_file(&gotten_path, "dir/b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(gotten_file(&gotten_path, "dir/sub/c.txt")).unwrap(), b"c");
    }

    #[test]
    fn file_is_gettable() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);

        get(&destination_path, "dir/b.txt", &gotten_path).unwrap();

        assert!(!gotten_file(&gotten_path, "a.txt").exists());
        assert_eq!(fs::read(gotten_file(&gotten_path, "dir/b.txt")).unwrap(), b"b");
        assert!(!gotten_file(&gotten_path, "dir/sub/c.txt").exists());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn modification_time_is_applied() {
        use std::time::Duration;
        use std::time::SystemTime;

        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);

        get(&destination_path, "", &gotten_path).unwrap();

        let modified = fs::metadata(gotten_file(&gotten_path, "a.txt")).unwrap().modified().unwrap();
        assert_eq!(modified, SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000));
    }

    #[test]
    fn file_whose_object_is_missing_is_skipped() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);
        let backup_path = destination_path.join("Backups").join(BACKUP_NAME);
        write_entry(&backup_path, "missing.txt", "0404040404040404");

        get(&destination_path, "", &gotten_path).unwrap();

        assert!(!gotten_file(&gotten_path, "missing.txt").exists());
        for (file_path, _, bytes) in FILES {
            assert_eq!(fs::read(gotten_file(&gotten_path, file_path)).unwrap(), bytes);
        }
    }

    #[test]
    fn getting_from_nonexistent_backup_fails() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        fs::create_dir_all(destination_path.join("Backups")).unwrap();
        fs::create_dir_all(destination_path.join("Objects")).unwrap();

        let error = get(&destination_path, "", &gotten_path).unwrap_err();

        assert_eq!(error.id, files_getter::ERROR_ID);
        assert_eq!(error.code, files_getter::ERROR_CODE_BACKUP_NOT_FOUND);
        assert!(!gotten_path.exists());
    }

    #[test]
    fn getting_nonexistent_extraction_path_fails() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);

        let error = get(&destination_path, "nonexistent", &gotten_path).unwrap_err();

        assert_eq!(error.id, files_getter::ERROR_ID);
        assert_eq!(error.code, files_getter::ERROR_CODE_GENERAL);
        assert!(!gotten_path.exists());
    }

    #[test]
    fn getting_invalid_entry_fails() {
        let temp_dir = TempDir::new("test").unwrap();
        let destination_path = temp_dir.path().join("Destination");
        let gotten_path = temp_dir.path().join("Gotten");
        create_destination(&destination_path, &FILES);
        let backup_path = destination_path.join("Backups").join(BACKUP_NAME);
        fs::write(backup_path.join("invalid.txt"), "invalid").unwrap();

        let error = get(&destination_path, "invalid.txt", &gotten_path).unwrap_err();

        assert_eq!(error.id, files_getter::ERROR_ID);
        assert_eq!(error.code, files_getter::ERROR_CODE_READING_ENTRY_FAILED);
    }
}
