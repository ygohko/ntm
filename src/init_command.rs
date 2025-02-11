use std::fs;

use crate::error;
use crate::error::Error;

pub struct InitCommand {
}

impl InitCommand {
    pub fn new() -> Self {
        InitCommand {
        }
    }

    pub fn execute(&self) -> Result<(), Error> {
        match fs::create_dir_all("NTM/Backups") {
            Ok(_) => (),
            Err(_) => return Err(Error::new(error::ERROR_ID, error::ERROR_CODE_GENERAL)),
        };
        match fs::create_dir_all("NTM/Objects") {
            Ok(_) => (),
            Err(_) => return Err(Error::new(error::ERROR_ID, error::ERROR_CODE_GENERAL)),
        };

        Ok(())
    }
}
