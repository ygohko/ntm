mod backup_command;
mod config;
mod error;
mod file_path_producer;
mod object_store;

use crate::backup_command::BackupCommand;
use crate::error::Error;

fn main() -> Result<(), Error> {
    let command = BackupCommand::new();
    command.execute()?;

    Ok(())
}
