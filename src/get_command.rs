use std::path::PathBuf;

use crate::error;
use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "get_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_BACKUP_NOT_FOUND: ErrorCode = 1;

pub struct GetCommand {
    backup: String,
}

impl GetCommand {
    // TODO: Add path option.
    pub fn new(backup: &str) -> Self {
        GetCommand {
            backup: backup.to_string(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        // TODO: Implement this.

        let mut path = PathBuf::new();
        path.push("NTM/Backups/");
        path.push(&self.backup);
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND)),
        };
        if !exists {
            return Err(Error::new(ERROR_ID, ERROR_CODE_BACKUP_NOT_FOUND));
        }

        Err(Error::new(error::ERROR_ID, error::ERROR_CODE_NOT_IMPLEMENTED))
    }
}
