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

use std::fs;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "init_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 1;

pub struct InitCommand {}

impl InitCommand {
    pub fn new() -> Self {
        InitCommand {}
    }

    pub fn execute(&self) -> Result<()> {
        match fs::create_dir_all("Backups") {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        match fs::create_dir_all("Objects") {
            Ok(_) => (),
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_CREATING_DIRECTORY_FAILED)),
        };
        // TODO: Create ntm.toml.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use tempdir::TempDir;

    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = InitCommand::new();
    }

    #[test]
    fn is_executable() {
        // TODO: Implement this.
        let temp_dir = TempDir::new("test").unwrap();
        let previous_current_dir = env::current_dir().unwrap();
        let mut current_dir = previous_current_dir.clone();
        current_dir.push(&temp_dir.path());
        env::set_current_dir(&current_dir).unwrap();

        let command = InitCommand::new();
        command.execute().unwrap();

        env::set_current_dir(&previous_current_dir).unwrap();
    }
}
