use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: String,
    pub last_modified: i64,
    pub permission: i32,
    pub uid: i32,
    pub gid: i32,
}
