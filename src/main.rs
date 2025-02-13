mod backup_command;
mod config;
mod entry;
mod error;
mod file_path_producer;
mod get_command;
mod init_command;
mod object_store;

use std::env;
use std::process::ExitCode;

use crate::backup_command::BackupCommand;
use crate::get_command::GetCommand;
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
            }
        };
    } else if command_name == "backup".to_string() {
        let command = BackupCommand::new();
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        };
    } else if command_name == "get".to_string() {
        if arguments.len() < 3 {
            println!("Missing BACKUP argument.");

            return ExitCode::FAILURE;
        }
        let backup = arguments[2].clone();
        let command = GetCommand::new(&backup);
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
