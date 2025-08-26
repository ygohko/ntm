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

use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::garbage_collector::GarbageCollector;
use crate::task::Task;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "gc_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;

/// A command to execute cleaning for a backup destination.
pub struct CleanCommand {
    destination_path: String,
    limited_count: Option<i64>,
}

impl Task for CleanCommand {
    /// Executes the clean command.
    ///
    /// This method identifies and removes unreferenced objects from the object store.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
    fn execute(&mut self) -> Result<()> {
        let mut collector = GarbageCollector::new();
        collector.set_destination_path(&self.destination_path);
        if let Some(limited_count) = self.limited_count {
            collector.set_limited_count(limited_count);
        }
        collector.execute()?;

        Ok(())
    }
}

impl CleanCommand {
    /// Creates a new `CleanCommand` instance.
    ///
    /// # Returns
    ///
    /// * `CleanCommand` - A new `CleanCommand` instance.
    pub fn new() -> Self {
        Self {
            destination_path: ".".to_string(),
            limited_count: None,
        }
    }

    /// Sets the destination path for the garbage collection.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the destination directory.
    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    /// Sets the limited count for the garbage collection.
    ///
    /// # Arguments
    ///
    /// * `count` - The maximum number of objects to process.
    pub fn set_limited_count(&mut self, count: i64) {
        self.limited_count = Some(count);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_command::BackupCommand;
    use crate::commons::OperatePath;
    use crate::gc_command::GcCommand;
    use crate::init_command::InitCommand;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _command = GcCommand::new();
    }

    #[test]
    fn is_executable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = &temp_dir.path().to_path_buf();
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

        let mut command = BackupCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut backup_path = ntm_path.clone();
        backup_path.push("Backups");
        backup_path.push(&command.name());
        fs::remove_dir_all(&backup_path).unwrap();
        let mut command = GcCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();
    }
}
