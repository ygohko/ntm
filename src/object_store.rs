use std::path::Path;
use std::path::PathBuf;

pub struct ObjectStore {
    path: PathBuf,
}

impl ObjectStore {
    pub fn new(path :&dyn AsRef<Path>) -> Self {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ObjectStore {
            path: path_buf,
        }
    }
}
