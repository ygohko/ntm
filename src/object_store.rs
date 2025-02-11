use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::error::Error;

pub const ERROR_ID: &'static str = "object_store";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: i32 = 0;
pub const ERROR_CODE_WRITING_OBJECT_FAILED: i32 = 1;

pub struct ObjectStore {
    path: PathBuf,
}

impl ObjectStore {
    pub fn new(path: &dyn AsRef<Path>) -> Self {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ObjectStore { path: path_buf }
    }

    pub fn add(&self, id: &str, bytes: &Vec<u8>) -> Result<(), Error> {
        let path1 = &id[0..2];
        let path2 = &id[2..4];
        let path3 = &id[4..6];
        let path4 = &id[6..8];
        let mut path = self.path.clone();
        path.push(path1);
        path.push(path2);
        path.push(path3);
        path.push(path4);
        println!("path: {}", path.display());
        match fs::create_dir_all(path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        }

        path.push(id);
        match fs::write(path, bytes) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_WRITING_OBJECT_FAILED)),
        }

        Ok(())
    }
}
