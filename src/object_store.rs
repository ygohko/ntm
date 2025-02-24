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

use crate::commons::OperatePath;
use crate::commons::ConvertPath;
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

const MARKED_OBJECTS_MAX:usize = 40000000;

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

    pub fn add(&self, id: &str, bytes: &Vec<u8>) -> Result<()> {
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

        path.push(id);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        };
        if exists {
            return Ok(());
        }
        match fs::write(path, bytes) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
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

    pub fn mark(&mut self, id: &str) -> Result<()> {
        if self.marked_objects.contains_key(id) {
            *self.marked_objects.get_mut(id).unwrap() += 1;

            return Ok(());
        }

        // TODO: Check cached IDs.
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        let file_name = id.to_string() + ".marked";
        path.push(&file_name);        

        if path.exists() {
            return Ok(());
        }
        if let Err(_) = fs::write(path, "") {
            return Err(Error::new(ERROR_ID, ERROR_CODE_MARKING_OBJECT_FAILED));
        }

        if self.marked_objects.len() > MARKED_OBJECTS_MAX {
            self.shrink_marked_objects();
        }
        self.marked_objects.insert(id.to_string(), 1);

        Ok(())
    }

    pub fn sweep(&self) -> Result<()> {
        let mut count: i32 = 0;
        let mut producer = FilePathProducer::new(&String::from_path(&self.path));
        let mut done = false;
        while !done {
            let option = match producer.next() {
                Ok(path) => Some(path),
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED {
                        done = true;
                    }

                    None
                },
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
                    }
                }
            }
        }

        // TODO: Remove mark files when sweeping.
        /*
        let mut producer = FilePathProducer::new(&String::from_path(&self.path));
        let mut done = false;
        while !done {
            let option = match producer.next() {
                Ok(path) => Some(path),
                Err(error) => {
                    if error.id == file_path_producer::ERROR_ID && error.code == file_path_producer::ERROR_CODE_PRODUCING_FINISHED {
                        done = true;
                    }
                    
                    None
                },
            };
            if let Some(path) = option {
                if path.rfind(".marked").is_some() {
                    let mark_path = String::from_path(&self.path).pushed(&path);
                    count += 1;
                    count %= 100;
                    if count == 0 {
                        println!("Cleaning: {}", mark_path);
                    }
                    if let Err(_) = fs::remove_file(&mark_path) {
                        println!("Warning: removing mark file {} failed.", path);
                    }
                }
            }
        }
        */

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

        println!("marked_objects shrinked. len: {}", self.marked_objects.len());
    }
}
