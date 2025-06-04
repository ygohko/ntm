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
use std::fs::File;
use std::ops::Add;
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use crate::commons::ConvertPath;
use crate::commons::OperatePath;
use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "get_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;
pub const ERROR_CODE_READING_ENTRY_FAILED: ErrorCode = 2;
pub const ERROR_CODE_WRITING_BYTES_FAILED: ErrorCode = 3;
pub const ERROR_CODE_WRITING_METADATA_FAILED: ErrorCode = 4;

pub struct GetCommand {
    backup: String,
    limited_directory: String,
    destination_path: String,
    gotten_path: String,
    processed_count: i64,
    count: i32,
}

impl Task for GetCommand {
    fn execute(&mut self) -> Result<()> {
        // TODO: Return error if path is invalid.
        let path = self.destination_path.pushed("Objects");
        let store = ObjectStore::new(&path);
        let mut backup_path = PathBuf::from(&self.destination_path);
        backup_path.push("Backups");
        backup_path.push(&self.backup);
        let mut path = PathBuf::new();
        path.push(&backup_path);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)),
        };
        if !exists {
            return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND));
        }

        if self.limited_directory != "".to_string() {
            path.push(&self.limited_directory);
        }
        let mut producer = FilePathProducer::new(&String::from_path(&path));
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
                let mut entry_path = PathBuf::new();
                entry_path.push(&backup_path);
                if self.limited_directory != "".to_string() {
                    entry_path.push(&self.limited_directory);
                }
                entry_path.push(&path);
                if self.count == 0 {
                    println!(
                        "Processing ({}): {}",
                        self.processed_count,
                        entry_path.display()
                    );
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
                let bytes = match store.bytes(&entry.id) {
                    Ok(bytes) => bytes,
                    // TODO: Skipping file that object is not found may be needed.
                    Err(error) => return Err(error),
                };
                let mut gotten_path = PathBuf::from(&self.gotten_path);
                gotten_path.push(&self.backup);
                if self.limited_directory != "".to_string() {
                    gotten_path.push(&self.limited_directory);
                }
                gotten_path.push(&path);
                let directories = String::from_path(&gotten_path).directories();
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
                // TODO: Apply meta data in entry.
                apply_metadata(&gotten_path.to_string_lossy(), &entry);
                
                self.processed_count += 1;
            }
        }
        println!("{} file(s) gotten.", self.processed_count);

        Ok(())
    }
}

impl GetCommand {
    pub fn new(backup: &str) -> Self {
        GetCommand {
            backup: backup.to_string(),
            limited_directory: "".to_string(),
            destination_path: ".".to_string(),
            gotten_path: ".".to_string(),
            processed_count: 0,
            count: 0,
        }
    }

    pub fn set_limited_directory(&mut self, directory: &str) -> () {
        self.limited_directory = directory.to_string();
    }

    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    #[allow(dead_code)]
    pub fn set_gotten_path(&mut self, path: &str) {
        self.gotten_path = path.to_string();
    }
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

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED)),
    };
    let mut permissions = metadata.permissions();
    permissions.set_mode(entry.permission);
    if let Err(_) = fs::set_permissions(path, permissions) {
        return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_METADATA_FAILED));
    }

    // TODO: Change owner.
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::ConvertPath;
    use crate::get_command::GetCommand;
    use crate::init_command::InitCommand;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _command = GetCommand::new("");
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
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
        command.set_destination_path(&String::from_path(&ntm_path));
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.set_destination_path(&String::from_path(&ntm_path));
        command.execute().unwrap();

        let mut gotten_path = temp_path.clone();
        gotten_path.push("gotten");
        fs::create_dir_all(&gotten_path).unwrap();
        let mut command = GetCommand::new(&command.name);
        command.set_destination_path(&String::from_path(&ntm_path));
        command.set_gotten_path(&String::from_path(&gotten_path));
        command.execute().unwrap();
    }
}
