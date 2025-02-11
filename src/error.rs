use std::backtrace::Backtrace;
use std::fmt;

pub type ErrorId = &'static str;
pub type ErrorCode = i32;
pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "error";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_NOT_IMPLEMENTED: ErrorCode = 1;

#[derive(Debug)]
pub struct Error {
    pub id: &'static str,
    pub code: i32,
    pub backtrace: String,
    pub details: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}\ncode: {}\nbacktrace: \n{}details: {}\n",
            self.id, self.code, self.backtrace, self.details
        )
    }
}

impl Error {
    pub fn new(id: &'static str, code: i32) -> Self {
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
