/*
 * Copyright (c) 2025 - 2026 Yasuaki Gohko
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

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use ntm_model::commons::Notification;
use ntm_model::error::Result;
use ntm_model::files_getter::FilesGetter;
use ntm_model::task::Task;

/// A command to get backuped files and directories.
pub struct GetCommand {
    name: String,
    limited_directory: String,
    destination_path: String,
    gotten_path: String,
}

impl Task for GetCommand {
    /// Executes the get command.
    ///
    /// This method retrieves backed-up files and directories from the object store
    /// and restores them to the specified destination.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the operation fails.
    fn execute(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<Notification>();
        let handle = thread::spawn(move || {
            Self::receive_notification(receiver);
        });

        let result: Result<()>;
        {
            let mut getter = FilesGetter::new(
                &self.name,
                &self.limited_directory,
                &self.destination_path,
                &self.gotten_path,
            );
            getter.set_notification_sender(sender);
            result = getter.execute();
        }
        let _ = handle.join();

        result
    }
}

impl GetCommand {
    /// Creates a new `GetCommand` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the backup to retrieve.
    ///
    /// # Returns
    ///
    /// * `GetCommand` - A new `GetCommand` instance.
    pub fn new(name: &str) -> Self {
        GetCommand {
            name: name.to_string(),
            limited_directory: "".to_string(),
            destination_path: ".".to_string(),
            gotten_path: ".".to_string(),
        }
    }

    /// Sets the directory to limit the retrieval of backuped directories and files.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory path to limit retrieval.
    pub fn set_limited_directory(&mut self, directory: &str) -> () {
        self.limited_directory = directory.to_string();
    }

    /// Sets the destination path for the get operation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the destination directory.
    pub fn set_destination_path(&mut self, path: &str) {
        self.destination_path = path.to_string();
    }

    #[allow(dead_code)]
    /// Sets the path where the gotten files will be stored.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to store the gotten files.
    pub fn set_gotten_path(&mut self, path: &str) {
        self.gotten_path = path.to_string();
    }

    fn receive_notification(receiver: Receiver<Notification>) {
        loop {
            let notification = match receiver.recv() {
                Ok(notification) => notification,
                Err(_) => break,
            };
            match notification {
                Notification::Error(string) => println!("Error: {}", string),
                Notification::Progress(string) => println!("{}", string),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::consts;
    use std::fs;
    #[cfg(not(target_os = "windows"))]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(not(target_os = "windows"))]
    use std::time::SystemTime;
    use tempdir::TempDir;

    use ntm_model::commons::OperatePath;
    use ntm_model::task::Task;
    
    use crate::backup_execute_command::BackupExecuteCommand;
    use crate::get_command::GetCommand;
    use crate::init_command::InitCommand;

    #[test]
    fn is_creatable() {
        let _command = GetCommand::new("");
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
        let mut config = format!("source_path = \"{}\"", source_path.display());
        if consts::OS == "windows" {
            config = config.replace("\\", "\\\\");
        }
        fs::write(config_path, config).unwrap();

        let mut command = BackupExecuteCommand::new();
        command.set_destination_path(&ntm_path.to_string_easy());
        command.execute().unwrap();

        let mut gotten_path = temp_path.clone();
        gotten_path.push("gotten");
        fs::create_dir_all(&gotten_path).unwrap();
        let mut command = GetCommand::new(&command.name());
        command.set_destination_path(&ntm_path.to_string_easy());
        command.set_gotten_path(&gotten_path.to_string_easy());
        command.execute().unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn metadata_is_restorable() {
        let temp_dir = TempDir::new("test").unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let mut source_path = temp_path.clone();
        source_path.push("source");
        fs::create_dir_all(&source_path).unwrap();
        let mut file_path = source_path.clone();
        file_path.push("a.txt");
        fs::write(&file_path, "ABCDE").unwrap();
        let metadata = fs::metadata(&file_path).unwrap();
        let system_time = metadata.modified().unwrap();
        let duration = system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let modified = duration.as_secs();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o777);
        fs::set_permissions(&file_path, permissions).unwrap();

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
        let backup_name = command.name();

        let mut gotten_path = temp_path.clone();
        gotten_path.push("gotten");
        fs::create_dir_all(&gotten_path).unwrap();
        let mut command = GetCommand::new(&command.name());
        command.set_destination_path(&ntm_path.to_string_easy());
        command.set_gotten_path(&gotten_path.to_string_easy());
        command.execute().unwrap();

        let mut file_path = gotten_path.clone();
        file_path.push(&backup_name);
        file_path.push("a.txt");
        let metadata = fs::metadata(&file_path).unwrap();
        let system_time = metadata.modified().unwrap();
        let duration = system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let gotten_modified = duration.as_secs();
        assert_eq!(modified, gotten_modified);
        let permissions = metadata.permissions();
        let permission = permissions.mode() & 0o777;
        assert_eq!(permission, 0o777);
    }
}
