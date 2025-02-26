/*
 * Copyright (c) 2025 Yasuaki Gohko
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE ABOVE LISTED COPYRIGHT HOLDER(S) BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

use std::backtrace::Backtrace;
use std::fmt;

pub type ErrorId = &'static str;
pub type ErrorCode = i32;
pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "error";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use crate::error;
    use crate::error::Error;

    #[test]
    fn is_displayable() {
        let error = Error::new(error::ERROR_ID, error::ERROR_CODE_GENERAL);
        let _string = format!("{}", error);
    }
    
    #[test]
    fn is_creatable() {
        let _error = Error::new(error::ERROR_ID, error::ERROR_CODE_GENERAL);
    }
}
