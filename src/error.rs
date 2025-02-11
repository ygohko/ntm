use std::backtrace::Backtrace;

pub const ERROR_ID: &str = "error";

pub const ERROR_CODE_GENERAL: i32 = 0;
pub const ERROR_CODE_PRODUCING_FINISHED: i32 = 1;

#[derive(Debug)]
pub struct Error {
    pub id: &'static str,
    pub code: i32,
    pub backtrace: String,
    pub details: String,
}

impl Error {
    pub fn new(id: &'static str, code: i32) -> Error {
        let backtrace = Backtrace::capture();
        let string = format!("{}", backtrace);
        Error {
            id,
            code,
            backtrace: string,
            details: "".to_string(),
        }
    }
}
