use std::fs;
use std::path::Path;
use std::path::PathBuf;

// TODO: Add Error type.

pub struct ObjectStore {
    path: PathBuf,
}

impl ObjectStore {
    pub fn new(path: &dyn AsRef<Path>) -> Self {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ObjectStore { path: path_buf }
    }

    pub fn add(&self, id: &str, bytes: &Vec<u8>) {
        // TODO: Implement this.
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
            Err(_) => panic!(),
        }

        // TODO: Write bytes.
        path.push(id);
        match fs::write(path, bytes) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }
}
