use std::backtrace::Backtrace;
use std::fmt;

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error. id: {}, code: {}, backtrace: {}, details: {}",
            self.id, self.code, self.backtrace, self.details
        )
    }
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
