/*
 * Copyright (c) 2026 Yasuaki Gohko
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

use camino::Utf8PathBuf;
use std::fs;

use crate::commons::OperatePath;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "destination";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 1;

/// A task for initializing backup destination
pub struct Destination {
    path: String,
}

impl Destination {
    /// Creates a new `Destination` instance.
    ///
    /// # Returns
    ///
    /// * `Destination` - A new `Destination` instance.
    pub fn new(path: &str) -> Self {
        Destination {
            path: path.to_string(),
        }
    }

    /// Executes the destination initializer task.
    ///
    /// This method initializes the backup destination.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
    pub fn init(path: &str) -> Result<Self> {
        let mut path = Utf8PathBuf::from(&path);
        path.push("Backups");
        match fs::create_dir_all(&path) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        let mut path = Utf8PathBuf::from(&path);
        path.push("Objects");
        match fs::create_dir_all(&path) {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        // TODO: Create ntm.toml.

        let result = Self {
            path: path.to_string_easy(),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use crate::commons::OperatePath;
    use crate::destination::Destination;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _destination = Destination::new("NTM");
    }

    #[test]
    fn is_initializable() {
        let temp_dir = TempDir::new("test").unwrap();
        let _ = Destination::init(&temp_dir.path().to_string_easy()).unwrap();
    }
}
