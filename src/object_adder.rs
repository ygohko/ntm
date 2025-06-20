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
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;

use crate::attributes::Attributes;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::object_store;
use crate::object_store::ObjectStore;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "object_adder";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_SOURCE_FAILED: ErrorCode = 1;
pub const ERROR_CODE_WRITING_OBJECT_FAILED: ErrorCode = 2;

pub struct ObjectAdder {
    store: Arc<RwLock<ObjectStore>>,
    id: String,
    path: String,
    source_path: String,
    file_size: u64,
    time_stamp: i64,
    added_count: Arc<AtomicI64>,
}

impl Task for ObjectAdder {
    fn execute(&mut self) -> Result<()> {
        const DIVIDED_WRITING_THRESHOLD: u64 = 1024 * 1024 * 1024;
        const DIVIDED_WRITING_SIZE: i64 = 100 * 1024 * 1024;

        let mut store = self.store.write().unwrap();
        let exists = match store.exists(&self.id) {
            Ok(exists) => exists,
            Err(error) => return Err(error),
        };
        if exists {
            return Ok(());
        }

        let mut joined_path = Utf8PathBuf::from(&self.source_path);
        joined_path.push(&self.path);
        if self.file_size < DIVIDED_WRITING_THRESHOLD {
            let bytes = match fs::read(&joined_path) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
            };
            let attribute = Attributes::new(&self.path, self.time_stamp);
            store.add(&self.id, &bytes, &attribute)?;
        } else {
            let mut remains: i64 = self.file_size as i64;
            let mut file = match OpenOptions::new().read(true).open(&joined_path) {
                Ok(file) => file,
                Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_SOURCE_FAILED)),
            };
            let attribute = Attributes::new(&self.path, self.time_stamp);
            let mut needs_writing = true;
            if let Err(error) = store.begin_adding(&self.id, &attribute) {
                if error.id == object_store::ERROR_ID
                    && error.code == object_store::ERROR_CODE_OBJECT_ALREADY_EXISTS
                {
                    needs_writing = false;
                } else {
                    return Err(error);
                }
            }

            if needs_writing {
                while remains > 0 {
                    let mut reading = remains;
                    if reading > DIVIDED_WRITING_SIZE {
                        reading = DIVIDED_WRITING_SIZE;
                    }
                    let mut bytes: Vec<u8> = Vec::new();
                    bytes.resize(reading as usize, 0);
                    if let Err(_) = file.read(&mut bytes) {
                        return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED));
                    }
                    store.write_adding(&bytes)?;

                    remains -= reading;
                }

                store.end_adding();
            }
        }
        self.added_count.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }
}

impl ObjectAdder {
    pub fn new(
        store: &Arc<RwLock<ObjectStore>>,
        id: &str,
        path: &str,
        source_path: &str,
        file_size: u64,
        time_stamp: i64,
        added_count: &Arc<AtomicI64>,
    ) -> Self {
        Self {
            store: store.clone(),
            id: id.to_string(),
            path: path.to_string(),
            source_path: source_path.to_string(),
            file_size: file_size,
            time_stamp: time_stamp,
            added_count: added_count.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::AtomicI64;
    use std::sync::Arc;
    use std::sync::RwLock;
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::object_adder::ObjectAdder;
    use crate::object_store::ObjectStore;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut store_path = temp_path.clone();
        store_path.push("Objects");
        fs::create_dir_all(&store_path).unwrap();
        let store = Arc::new(RwLock::new(ObjectStore::new(&store_path.to_string_easy())));
        let mut file_path = temp_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "abcdef").unwrap();
        let added_count = Arc::new(AtomicI64::new(0));
        let _adder = ObjectAdder::new(
            &store,
            "12345678",
            "a.txt",
            &temp_path.to_string_easy(),
            6,
            12345678,
            &added_count,
        );
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut store_path = temp_path.clone();
        store_path.push("Objects");
        fs::create_dir_all(&store_path).unwrap();
        let store = Arc::new(RwLock::new(ObjectStore::new(&store_path.to_string_easy())));
        let mut file_path = temp_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "abcdef").unwrap();
        let added_count = Arc::new(AtomicI64::new(0));
        let mut adder = ObjectAdder::new(
            &store,
            "12345678",
            "a.txt",
            &temp_path.to_string_easy(),
            6,
            12345678,
            &added_count,
        );
        adder.execute().unwrap();
    }
}
