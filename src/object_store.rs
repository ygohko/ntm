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
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

use crate::attributes::Attributes;
use crate::commons::OperatePath;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "object_store";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_OBJECT_FAILED: ErrorCode = 1;
pub const ERROR_CODE_WRITING_OBJECT_FAILED: ErrorCode = 2;
pub const ERROR_CODE_REMOVING_OBJECT_FAILED: ErrorCode = 3;
pub const ERROR_CODE_READING_ATTTIBUTE_FAILED: ErrorCode = 4;
pub const ERROR_CODE_WRITING_ATTTIBUTE_FAILED: ErrorCode = 5;
pub const ERROR_CODE_REMOVING_ATTTIBUTE_FAILED: ErrorCode = 6;
pub const ERROR_CODE_OBJECT_ALREADY_EXISTS: ErrorCode = 7;
pub const ERROR_CODE_INVALID_OBJECT_ID: ErrorCode = 8;
pub const ERROR_CODE_READING_CACHED_FAILED: ErrorCode = 9;
pub const ERROR_CODE_WRITING_CACHED_FAILED: ErrorCode = 10;

const EXISTING_IDS_TABLE_COUNT: usize = 0x100 * 0x100;

#[derive(Serialize, Deserialize, Clone)]
struct SerializableExistingIds {
    ids: Vec<String>,
}

pub struct ObjectStore {
    path: String,
    adding_file: Option<File>,
    existing_ids: Vec<Vec<String>>,
}

impl ObjectStore {
    pub fn new(path: &str) -> Self {
        let mut existing_ids: Vec<Vec<String>> = Vec::new();
        for _ in 0..EXISTING_IDS_TABLE_COUNT {
            existing_ids.push(Vec::new());
        }

        ObjectStore {
            path: path.to_string(),
            adding_file: None,
            existing_ids: existing_ids,
        }
    }

    pub fn add(&mut self, id: &str, bytes: &Vec<u8>, attributes: &Attributes) -> Result<()> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let Ok(index1) = u32::from_str_radix(path1, 16) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
        };
        let Ok(index2) = u32::from_str_radix(path2, 16) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
        };
        let index = (index1 * 0x100 + index2) as usize;
        let ids = &mut self.existing_ids[index];
        // TODO: Use iter()?
        if ids.into_iter().position(|id1| id1 == id).is_some() {
            return Ok(());
        }
        ids.push(id.to_string());

        let mut path = Utf8PathBuf::from(&self.path);
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

        let serialized = match serde_json::to_string(&attributes) {
            Ok(serialized) => serialized,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED)),
        };
        let mut attributes_path = path.clone();
        let attributes_name = id.to_string() + ".attributes";
        attributes_path.push(attributes_name);
        if let Err(_) = fs::write(&attributes_path, &serialized) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED));
        }

        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = Utf8PathBuf::from(&self.path);
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        path.push(id);
        if let Err(_) = fs::remove_file(&path) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_REMOVING_OBJECT_FAILED));
        }

        let mut attributes_path = path.to_string_easy();
        attributes_path.push_str(".attributes");
        if let Err(_) = fs::remove_file(&attributes_path) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_REMOVING_ATTTIBUTE_FAILED));
        };

        Ok(())
    }

    pub fn bytes(&self, id: &str) -> Result<Vec<u8>> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = Utf8PathBuf::from(&self.path);
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        path.push(id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_OBJECT_FAILED)),
        };

        Ok(bytes)
    }

    pub fn attributes(&self, id: &str) -> Result<Attributes> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = Utf8PathBuf::from(&self.path);
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        let file_name = id.to_string() + ".attributes";
        path.push(file_name);
        let Ok(serialized) = fs::read_to_string(&path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ATTTIBUTE_FAILED));
        };
        let attributes: Attributes = match serde_json::from_str(&serialized) {
            Ok(attributes) => attributes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_ATTTIBUTE_FAILED)),
        };

        Ok(attributes)
    }

    pub fn exists(&mut self, id: &str) -> Result<bool> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let Ok(index1) = u32::from_str_radix(path1, 16) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
        };
        let Ok(index2) = u32::from_str_radix(path2, 16) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
        };
        let index = (index1 * 0x100 + index2) as usize;
        // TODO: Use iter()?
        let ids = &mut self.existing_ids[index];
        if ids.into_iter().position(|id1| id1 == id).is_some() {
            return Ok(true);
        }

        let mut path = Utf8PathBuf::from(&self.path);
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        path.push(id);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_OBJECT_FAILED)),
        };
        if exists {
            ids.push(id.to_string());
        }

        Ok(exists)
    }

    pub fn begin_adding(&mut self, id: &str, attributes: &Attributes) -> Result<()> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = Utf8PathBuf::from(&self.path);
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
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
            return Err(Error::new(ERROR_ID, ERROR_CODE_OBJECT_ALREADY_EXISTS));
        }

        self.adding_file = match OpenOptions::new()
            .write(true)
            .create(true)
            .open(&object_path)
        {
            Ok(file) => Some(file),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        };

        let serialized = match serde_json::to_string(&attributes) {
            Ok(serialized) => serialized,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED)),
        };
        let mut attributes_path = path.clone();
        let attributes_name = id.to_string() + ".attributes";
        attributes_path.push(attributes_name);
        if let Err(_) = fs::write(&attributes_path, &serialized) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_ATTTIBUTE_FAILED));
        }

        Ok(())
    }

    pub fn write_adding(&self, bytes: &Vec<u8>) -> Result<()> {
        if self.adding_file.is_none() {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED));
        }

        if let Err(_) = self.adding_file.as_ref().unwrap().write(bytes) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED));
        }

        Ok(())
    }

    pub fn end_adding(&mut self) {
        if self.adding_file.is_none() {
            return;
        }
        self.adding_file = None;
    }

    pub fn load_existing_ids(&mut self) -> Result<()> {
        let path = Utf8PathBuf::from(&self.path).parent_or_empty();
        let mut path = Utf8PathBuf::from(path);
        path.push("existing_ids.json");
        let Ok(serialized) = fs::read_to_string(&path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CACHED_FAILED));
        };

        let serializable: SerializableExistingIds = match serde_json::from_str(&serialized) {
            Ok(serializable) => serializable,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CACHED_FAILED)),
        };
        for id in serializable.ids {
            let path1 = &id[0..2];
            let path2 = &id[2..4];
            let Ok(index1) = u32::from_str_radix(path1, 16) else {
                return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
            };
            let Ok(index2) = u32::from_str_radix(path2, 16) else {
                return Err(Error::new(ERROR_ID, ERROR_CODE_INVALID_OBJECT_ID));
            };
            let index = (index1 * 0x100 + index2) as usize;
            self.existing_ids[index].push(id);
        }

        Ok(())
    }

    // TODO: Rename to save_cached()?
    pub fn save_existing_ids(&self) -> Result<()> {
        let mut serializable = SerializableExistingIds { ids: Vec::new() };
        for i in 0..EXISTING_IDS_TABLE_COUNT {
            let ids = &self.existing_ids[i];
            for id in ids {
                serializable.ids.push(id.clone());
            }
        }
        let Ok(serialized) = serde_json::to_string(&serializable) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_CACHED_FAILED));
        };

        let path = Utf8PathBuf::from(&self.path).parent_or_empty();
        let mut path = Utf8PathBuf::from(path);
        path.push("existing_ids.json");
        if let Err(_) = fs::write(&path, &serialized) {
            return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_CACHED_FAILED));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::object_store::Attributes;
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
        let _store = ObjectStore::new(&path.to_string_easy());
    }

    #[test]
    fn object_is_addable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        store.add(&id, &bytes, &attribute).unwrap();
    }

    #[test]
    fn object_is_removable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        store.add(&id, &bytes, &attribute).unwrap();
        store.remove(&id).unwrap();
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
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        let Ok(_) = store.add(&id, &bytes, &attribute) else {
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
    fn attributes_are_gettable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attributes = Attributes::new("a/b/c/d.txt", 123456);
        let Ok(_) = store.add(&id, &bytes, &attributes) else {
            panic!();
        };

        let Ok(attributes1) = store.attributes(&id) else {
            panic!();
        };
        assert_eq!(attributes.path, attributes1.path);
        assert_eq!(attributes.added, attributes1.added);
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
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        let Ok(_) = store.add(&id, &bytes, &attribute) else {
            panic!();
        };

        let exists = store.exists(&id).unwrap();
        assert_eq!(exists, true);
    }

    #[test]
    fn object_is_divided_addable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        store.begin_adding(&id, &attribute).unwrap();
        store.write_adding(&bytes).unwrap();
        store.end_adding();
    }

    #[test]
    fn existing_id_is_loadable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        store.add(&id, &bytes, &attribute).unwrap();
        store.save_existing_ids().unwrap();
        store.load_existing_ids().unwrap();
    }

    #[test]
    fn existing_id_is_savable() {
        let Ok(temp_dir) = TempDir::new("test") else {
            panic!();
        };
        let path = temp_dir.path().join("Objects");
        if let Err(_) = fs::create_dir_all(&path) {
            panic!();
        }
        let mut store = ObjectStore::new(&path.to_string_easy());

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        store.add(&id, &bytes, &attribute).unwrap();
        store.save_existing_ids().unwrap();
    }
}
