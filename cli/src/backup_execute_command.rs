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

use ntm_model::backup_executer::BackupExecuter;
use ntm_model::error::Result;
use ntm_model::task::Task;

/// A command to backup files and directories.
pub struct BackupExecuteCommand {
    name: String,
    destination_path: String,
}

impl Task for BackupExecuteCommand {
    fn execute(&mut self) -> Result<()> {
        let mut executer = BackupExecuter::new(&self.destination_path);
        let result = executer.execute();
        self.name = executer.name();

        result
    }
}

impl BackupExecuteCommand {
    /// Creates a new `BackupExecuteCommand` instance.
    ///
    /// # Returns
    ///
    /// A new `BackupExecuteCommand` instance.
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            destination_path: ".".to_string(),
        }
    }

    /// Sets the destination path for the backup.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the backup destination.
    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    #[cfg(test)]
    /// Returns the name of the backup.
    ///
    /// # Returns
    ///
    /// A `String` representing the name of the backup.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use ntm_model::commons::OperatePath;
    use ntm_model::task::Task;

    use crate::backup_execute_command::BackupExecuteCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = BackupExecuteCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut source_path = temp_path.clone();
        source_path.push("source");
        fs::create_dir_all(&source_path).unwrap();
        let mut file_path = source_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "ABCDE").unwrap();

        let mut ntm_path = temp_path.clone();
        ntm_path.push("ntm");
        fs::create_dir_all(&ntm_path).unwrap();
        let mut command = InitCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut config_path = ntm_path.clone();
        config_path.push("ntm.toml");
        let config = format!("source_path = \"{}\"", source_path.display());
        fs::write(config_path, config).unwrap();

        let mut command = BackupExecuteCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();
    }
}
