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

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::commons::ConvertPath;
use crate::commons::OperatePath;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::file_path_producer;
use crate::file_path_producer::FilePathProducer;

pub const ERROR_ID: ErrorId = "object_store";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_OBJECT_FAILED: ErrorCode = 1;
pub const ERROR_CODE_WRITING_OBJECT_FAILED: ErrorCode = 2;
pub const ERROR_CODE_MARKING_OBJECT_FAILED: ErrorCode = 3;
pub const ERROR_CODE_WRITING_ATTTIBUTE_FAILED: ErrorCode = 4;

const MARKED_OBJECTS_MAX: usize = 4000000;

#[derive(PartialEq)]
pub enum MarkingResult {
    Marked,
    AlreadyMarked,
    NotFound,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Attribute {
    path: String,
    added: i64,
}

pub struct ObjectStore {
    path: PathBuf,
    marked_objects: HashMap<String, i64>,
}

impl ObjectStore {
    pub fn new(path: &dyn AsRef<Path>) -> Self {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ObjectStore {
            path: path_buf,
            marked_objects: HashMap::new(),
        }
    }

    pub fn add(&self, id: &str, bytes: &Vec<u8>, attribute: &Attribute) -> Result<()> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        // println!("path: {}", path.display());
        match fs::create_dir_all(path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        }

        let mut object_path = path.clone();
        object_path.push(id);
        let exists = match object_path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        };
        if exists {
            return Ok(());
        }
        match fs::write(object_path, bytes) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        }

        let serialized = match serde_json::to_string(&attribute) {
            Ok(serialized) => serialized,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED)),
        };
        let mut attribute_path = path.clone();
        let attribute_name = id.to_string() + ".attribute";
        attribute_path.push(attribute_name);
        if let Err(_) = fs::write(&attribute_path, &serialized) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED));
        }

        Ok(())
    }

    pub fn bytes(&self, id: &str) -> Result<Vec<u8>> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        path.push(id);
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_OBJECT_FAILED)),
        };

        Ok(bytes)
    }

    pub fn exists(&self, id: &str) -> Result<bool> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        path.push(id);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_OBJECT_FAILED)),
        };

        Ok(exists)
    }

    pub fn mark(&mut self, id: &str) -> Result<MarkingResult> {
        if self.marked_objects.contains_key(id) {
            *self.marked_objects.get_mut(id).unwrap() += 1;

            return Ok(MarkingResult::AlreadyMarked);
        }

        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        let mut object_path = path.clone();
        object_path.push(id);
        let file_name = id.to_string() + ".marked";
        let mut mark_path = path.clone();
        mark_path.push(&file_name);

        let result: MarkingResult;
        if mark_path.exists() {
            result = MarkingResult::AlreadyMarked;
        } else {
            if let Err(_) = fs::write(mark_path, "") {
                if object_path.exists() {
                    return Err(Error::new(ERROR_ID, ERROR_CODE_MARKING_OBJECT_FAILED));
                }

                return Ok(MarkingResult::NotFound);
            }
            
            result = MarkingResult::Marked;
        }

        if self.marked_objects.len() > MARKED_OBJECTS_MAX {
            self.shrink_marked_objects();
        }
        self.marked_objects.insert(id.to_string(), 1);

        Ok(result)
    }

    pub fn sweep(&self) -> Result<()> {
        let mut count: i32 = 0;
        let mut producer = FilePathProducer::new(&String::from_path(&self.path));
        let mut removed_count:i64 = 0;
        let mut done = false;
        while !done {
            let option = match producer.next() {
                Ok(path) => Some(path),
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID
                        && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED
                    {
                        done = true;
                    }

                    None
                }
            };
            if let Some(path) = option {
                if path.rfind(".marked").is_none() {
                    let mark_path = path.clone() + ".marked";
                    let mark_path = String::from_path(&self.path).pushed(&mark_path);
                    count += 1;
                    count %= 100;
                    if count == 0 {
                        println!("Checking: {}", mark_path);
                    }
                    let exists = PathBuf::from(&mark_path).exists();
                    if exists {
                        if let Err(_) = fs::remove_file(&mark_path) {
                            println!("Warning: removing mark file {} failed.", path);
                        }
                    } else {
                        let object_path = String::from_path(&self.path).pushed(&path);
                        if let Err(_) = fs::remove_file(&object_path) {
                            println!("Warning: removing object {} failed.", object_path);
                        }
                        removed_count += 1;
                    }
                }
            }
        }
        println!("{} object(s) removed.", removed_count);

        Ok(())
    }

    fn shrink_marked_objects(&mut self) {
        let count = self.marked_objects.len();
        let mut sum: i64 = 0;
        for (_, value) in &self.marked_objects {
            sum += value;
        }
        let average = sum / (count as i64);

        let mut removing_count = MARKED_OBJECTS_MAX / 2;
        let mut keys: Vec<String> = Vec::new();
        for key in self.marked_objects.keys() {
            keys.push(key.clone());
        }
        for key in keys {
            if self.marked_objects[&key] <= average {
                self.marked_objects.remove(&key);
                removing_count -= 1;
                if removing_count <= 0 {
                    break;
                }
            }
        }

        println!(
            "marked_objects shrinked. len: {}",
            self.marked_objects.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::object_store;
    use crate::object_store::Attribute;
    use crate::object_store::ObjectStore;

    #[test]
    fn is_creatable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let _store = ObjectStore::new(&path);
    }

    #[test]
    fn objetc_is_addable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attribute {
            path: "".to_string(),
            added: 0,
        };
        store.add(&id, &bytes, &attribute).unwrap();
    }

    #[test]
    fn bytes_are_gettable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let Ok(_) = store.add(&id, &bytes) else {
            panic!();
        };

        let Ok(bytes1) = store.bytes(&id) else {
            panic!();
        };
        assert_eq!(bytes.len(), bytes1.len());
        for i in 0..bytes.len() {
            assert_eq!(bytes[i], bytes1[i]);
        }
    }

    #[test]
    fn object_existing_is_testable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let Ok(_) = store.add(&id, &bytes) else {
            panic!();
        };

        let exists = store.exists(&id).unwrap();
        assert_eq!(exists, true);
    }

    #[test]
    fn object_is_markable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let Ok(_) = store.add(&id, &bytes) else {
            panic!();
        };

        let Ok(_) = store.mark(&id) else {
            panic!();
        };
    }

    #[test]
    fn object_is_sweepable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let Ok(_) = store.add(&id, &bytes) else {
            panic!();
        };

        let Ok(_) = store.mark(&id) else {
            panic!();
        };

        let Ok(_) = store.sweep() else {
            panic!();
        };
        let Ok(_) = store.bytes(&id) else {
            panic!();
        };

        let Ok(_) = store.sweep() else {
            panic!();
        };
        let Err(_) = store.bytes(&id) else {
            panic!();
        };
    }

    #[test]
    fn marked_objects_are_shrinkable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path);

        store
            .marked_objects
            .insert("ffffffffffffffff".to_string(), 1);
        assert_eq!(store.marked_objects.len(), 1);
        store.shrink_marked_objects();
        assert_eq!(store.marked_objects.len(), 0);

        for i in 0..object_store::MARKED_OBJECTS_MAX {
            let id = format!("{:x}", i);
            store.marked_objects.insert(id, 1);
        }
        assert_eq!(store.marked_objects.len(), object_store::MARKED_OBJECTS_MAX);
        store.shrink_marked_objects();
        assert_eq!(
            store.marked_objects.len(),
            object_store::MARKED_OBJECTS_MAX / 2
        );
    }
}
