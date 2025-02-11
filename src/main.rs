mod backup_command;
mod config;
mod error;
mod file_path_producer;
mod init_command;
mod object_store;

use std::env;
use std::process::ExitCode;

use crate::backup_command::BackupCommand;
use crate::init_command::InitCommand;

fn main() -> ExitCode {
    // TODO: Embed clap.
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() < 2 {
        println!("USAGE: ntm COMMAND");

        return ExitCode::SUCCESS;
    }
    let command_name = arguments[1].clone();

    if command_name == "init".to_string() {
        let command = InitCommand::new();
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            },
        };
    }
    else if command_name == "backup".to_string() {
        let command = BackupCommand::new();
        // command.execute()?;
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);
                return ExitCode::FAILURE;
            },
        };
    }

    ExitCode::SUCCESS
}
