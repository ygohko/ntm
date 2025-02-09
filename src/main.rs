mod backup_command;
mod error;
mod file_path_producer;
mod object_store;

use serde::Deserialize;
use std::fs;

use crate::backup_command::BackupCommand;
use crate::error::Error;

#[derive(Deserialize)]
struct Config {
    source_path: String,
}

fn main() -> Result<(), Error> {

    let bytes = match fs::read("NTM/config.toml") {
        Ok(bytes) => bytes,
        Err(_) => return Err(Error::new(error::CODE_GENERAL)),
    };
    let string = match String::from_utf8(bytes) {
        Ok(string) => string,
        Err(_) => return Err(Error::new(error::CODE_GENERAL)),
    };
    let result = toml::from_str(&string);
    if result.is_ok() {
        let config: Config = result.unwrap();
        println!("config.source_path: {}", config.source_path);
    }

    let command = BackupCommand::new();
    command.execute()?;

    Ok(())
}
