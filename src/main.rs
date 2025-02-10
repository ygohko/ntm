mod backup_command;
mod config;
mod error;
mod file_path_producer;
mod init_command;
mod object_store;

use std::env;

use crate::backup_command::BackupCommand;
use crate::error::Error;
use crate::init_command::InitCommand;

fn main() -> Result<(), Error> {
    // TODO: Embed clap.
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() < 2 {
        println!("USAGE: ntm COMMAND");

        return Ok(());
    }
    let command_name = arguments[1].clone();

    if command_name == "init".to_string() {
        let command = InitCommand::new();
        command.execute()?;
    }
    else if command_name == "backup".to_string() {
        let command = BackupCommand::new();
        command.execute()?;
    }

    Ok(())
}
