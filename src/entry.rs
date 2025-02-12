pub struct Entry {
    pub id: String,
    pub last_modified: i64,
    pub permission: i32,
    pub uid: i32,
    pub gid: i32,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            id: "".to_string(),
            last_modified: 0,
            permission: 0,
            uid: 0,
            gid: 0,
        }
    }
}
