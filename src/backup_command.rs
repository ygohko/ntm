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
use chrono::DateTime;
use chrono::Local;
use hex_string::HexString;
use sha2::Digest;
use sha2::Sha256;
use std::convert::From;
use std::fs;
use std::fs::Metadata;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::MetadataExt;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;

use crate::background_executer::BackgroundExecuter;
use crate::commons::OperatePath;
use crate::config::Config;
use crate::entry::Entry;
use crate::entry_saver::EntrySaver;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;
use crate::object_adder::ObjectAdder;
use crate::object_store::ObjectStore;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "backup_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_CONFIG_FAILED: ErrorCode = 1;
pub const ERROR_CODE_READING_SOURCE_FAILED: ErrorCode = 2;

pub struct BackupCommand {
    // TODO: Add getter method.
    pub name: String,
    executer: BackgroundExecuter,
    executing: DateTime<Local>,
    destination_path: String,
    excluded_directories: Vec<String>,
    processed_count: i64,
    added_count: Arc<AtomicI64>,
    count: i32,
}

impl Task for BackupCommand {
    fn execute(&mut self) -> Result<()> {
        self.executer.execute()?;
        let mut path = Utf8PathBuf::from(&self.destination_path);
        path.push("Objects");
        let store = Arc::new(RwLock::new(ObjectStore::new(&path.to_string_easy())));
        self.name = self.executing.format("%Y%m%d-%H%M").to_string();
        let mut path = Utf8PathBuf::from(&self.destination_path);
        path.push("ntm.toml");
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let string = match String::from_utf8(bytes) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let result = toml::from_str(&string);
        let config: Config;
        if result.is_ok() {
            config = result.unwrap();
            // println!("config.source_path: {}", config.source_path);
        } else {
            config = Config::new();
        }
        self.excluded_directories = match config.excluded_directories {
            Some(directories) => directories,
            None => vec![],
        };

        let mut producer = FilePathProducer::new(&config.source_path);
        if self.excluded_directories.len() > 0 {
            producer.set_excluded_directories(&self.excluded_directories);
        }
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
                if let Err(error) = self.process_file(&path, &store, &config.source_path) {
                    println!("process_file() failed: error: {}", error);
                }
            }
        }
        println!("Waiting for background tasks...");
        self.executer.terminate()?;

        println!(
            "{} object(s) added.",
            self.added_count.load(Ordering::Relaxed)
        );

        let store1 = store.read().unwrap();
        store1.test();
        
        Ok(())
    }
}

impl BackupCommand {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            executer: BackgroundExecuter::new(),
            executing: Local::now(),
            destination_path: ".".to_string(),
            excluded_directories: vec![],
            processed_count: 0,
            added_count: Arc::new(AtomicI64::new(0)),
            count: 0,
        }
    }

    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    fn process_file(
        &mut self,
        path: &String,
        store: &Arc<RwLock<ObjectStore>>,
        source_path: &String,
    ) -> Result<()> {
        if self.count == 0 {
            println!(
                "Processing ({}, {}): {}",
                self.processed_count,
                self.added_count.load(Ordering::Relaxed),
                path
            );
        }
        let path_buf = Utf8PathBuf::from(&path);
        let path_file_name = path_buf.file_name_or_empty();
        let path_directries = path_buf.parent_or_empty();
        self.count += 1;
        self.count %= 100;
        let mut path_buf = Utf8PathBuf::new();
        path_buf.push(&source_path);
        path_buf.push(path.clone());
        let metadata = match path_buf.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
        };
        let file_size = metadata.len();
        let mut modified: u64 = 0;
        if let Ok(system_time) = metadata.modified() {
            if let Ok(duration) = system_time.duration_since(SystemTime::UNIX_EPOCH) {
                modified = duration.as_secs();
            }
        }
        let permission = permission(&metadata);
        let uid = uid(&metadata);
        let gid = gid(&metadata);
        let id_path = path_buf.to_string_easy();
        let string = format!("p,{},{},{}", id_path, modified, file_size);
        let id = object_id(&string.as_bytes().to_vec());

        let adder = Box::new(ObjectAdder::new(
            &store.clone(),
            &id,
            &path,
            &source_path,
            file_size,
            self.executing.timestamp(),
            &self.added_count,
        ));
        if let Some(sender) = &self.executer.sender {
            if let Err(error) = sender.send(adder) {
                println!("Sending a task failed. error: {}", error);
            }
        }
        self.processed_count += 1;

        let mut entry_path = Utf8PathBuf::from(&self.destination_path);
        entry_path.push("Backups");
        entry_path.push(self.name.clone());
        entry_path.push(&path_directries);
        entry_path.push(&path_file_name);
        let entry = Entry {
            id: id,
            last_modified: modified,
            permission: permission,
            uid: uid,
            gid: gid,
        };
        let saver = Box::new(EntrySaver::new(&entry, &entry_path.to_string_easy()));
        if let Some(sender) = &self.executer.sender {
            if let Err(error) = sender.send(saver) {
                println!("Sending a task failed. error: {}", error);
            }
        }

        Ok(())
    }
}

fn object_id(bytes: &Vec<u8>) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(bytes.clone());
    let hash = sha256.finalize();
    let hash_values = hash.to_vec();
    let hex = HexString::from_bytes(&hash_values);

    hex.as_string()
}

#[cfg(not(target_os = "windows"))]
fn permission(metadata: &Metadata) -> u32 {
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    mode & 0o777
}

#[cfg(target_os = "windows")]
fn permission(metadata: &Metadata) -> u32 {
    let permissions = metadata.permissions();
    if permissions.readonly() {
        return 0o444;
    }

    0o644
}

#[cfg(not(target_os = "windows"))]
fn uid(metadata: &Metadata) -> u32 {
    let uid = metadata.uid();

    uid
}

#[cfg(target_os = "windows")]
fn uid(_metadata: &Metadata) -> u32 {
    0
}

#[cfg(not(target_os = "windows"))]
fn gid(metadata: &Metadata) -> u32 {
    let gid = metadata.gid();

    gid
}

#[cfg(target_os = "windows")]
fn gid(_metadata: &Metadata) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::OperatePath;
    use crate::init_command::InitCommand;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _command = BackupCommand::new();
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
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();
    }
}
