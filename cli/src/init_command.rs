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

use camino::Utf8PathBuf;
use std::fs;

use ntm_model::destination_initializer::DestinationInitializer;
use ntm_model::error::Error;
use ntm_model::error::ErrorCode;
use ntm_model::error::ErrorId;
use ntm_model::error::Result;
use ntm_model::task::Task;

pub const ERROR_ID: ErrorId = "init_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_CREATING_DIRECTORY_FAILED: ErrorCode = 1;

/// A command to initialize a backup destination.
pub struct InitCommand {
    destination_path: String,
}

impl Task for InitCommand {
    /// Executes the initialization command.
    ///
    /// This method creates the necessary directory structure for a new backup destination.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
    fn execute(&mut self) -> Result<()> {
        let mut initializer = DestinationInitializer::new(&self.destination_path);
        let result = initializer.execute();

        result
    }
}

impl InitCommand {
    /// Creates a new `InitCommand` instance.
    ///
    /// # Returns
    ///
    /// * `InitCommand` - A new `InitCommand` instance.
    pub fn new() -> Self {
        InitCommand {
            destination_path: ".".to_string(),
        }
    }

    /// Sets the destination path for the initialization.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the destination directory.
    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use ntm_model::commons::OperatePath;
    use ntm_model::task::Task;
    
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = InitCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let mut command = InitCommand::new();
        command.destination_path = temp_dir.path().to_string_easy();
        command.execute().unwrap();
    }
}
