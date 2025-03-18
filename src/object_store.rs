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
use std::path::Path;
use std::path::PathBuf;

use crate::attributes::Attributes;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "object_store";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_OBJECT_FAILED: ErrorCode = 1;
pub const ERROR_CODE_WRITING_OBJECT_FAILED: ErrorCode = 2;
pub const ERROR_CODE_READING_ATTTIBUTE_FAILED: ErrorCode = 3;
pub const ERROR_CODE_WRITING_ATTTIBUTE_FAILED: ErrorCode = 4;

pub struct ObjectStore {
    path: PathBuf,
}

impl ObjectStore {
    pub fn new(path: &dyn AsRef<Path>) -> Self {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ObjectStore {
            path: path_buf,
        }
    }

    pub fn add(&self, id: &str, bytes: &Vec<u8>, attributes: &Attributes) -> Result<()> {
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

    pub fn remove(&self) -> Result<()> {
        // kokokara--
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

    pub fn attributes(&self, id: &str) -> Result<Attributes> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
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
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

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
        let attribute = Attributes::new("", 0);
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
        let store = ObjectStore::new(&path);

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
        let store = ObjectStore::new(&path);

        let id = "0102030405060708".to_string();
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let attribute = Attributes::new("", 0);
        let Ok(_) = store.add(&id, &bytes, &attribute) else {
            panic!();
        };

        let exists = store.exists(&id).unwrap();
        assert_eq!(exists, true);
    }
}
