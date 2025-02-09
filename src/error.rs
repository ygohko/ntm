pub const CODE_GENERAL: i32 = 0;
pub const CODE_PRODUCING_FINISHED: i32 = 1;

pub struct Error {
    pub code: i32,
}

impl Error {
    pub fn new(code: i32) -> Error {
        Error {
            code,
        }
    }
}
