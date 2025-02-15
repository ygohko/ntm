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
