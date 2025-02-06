pub struct ObjectStore {
    path: String,
}

impl ObjectStore {
    pub fn new(path :&dyn AsRef<Path>) -> Self {
        ObjectStore {
            path,
        }
    }
}
