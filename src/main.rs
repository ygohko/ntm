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

mod attributes;
mod background_executer;
mod backup_command;
mod backup_remover;
mod backup_store;
mod clean_command;
mod commons;
mod config;
mod entry;
mod entry_saver;
mod error;
mod file_path_producer;
mod garbage_collector;
mod get_command;
mod init_command;
mod object_adder;
mod object_store;
mod remove_backup_command;
mod task;

use clap::Parser;
use clap::Subcommand;
use std::process::ExitCode;
use std::time::SystemTime;

use crate::backup_command::BackupCommand;
use crate::clean_command::CleanCommand;
use crate::get_command::GetCommand;
use crate::init_command::InitCommand;
use crate::remove_backup_command::RemoveBackupCommand;
use crate::task::Task;

#[derive(Parser, PartialEq)]
struct InitArguments {
    /// Backup destnation that is initialized
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Parser, PartialEq)]
struct BackupExecuteArguments {
    /// Backup destination that is used for backup
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Subcommand, PartialEq)]
enum BackupCommandKind {
    /// Execute backup
    Execute(BackupExecuteArguments),
    /// Remove specified backups
    Remove,
}

#[derive(Parser, PartialEq)]
struct BackupArguments {
    /// Sub command for backup command
    #[command(subcommand)]
    command: Option<BackupCommandKind>,
}

#[derive(Parser, PartialEq)]
struct RemoveBackupArguments {
    /// Pattern to specify removing backups
    pattern: String,
    /// Backup destination that is used for backup
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Parser, PartialEq)]
struct GetArguments {
    /// Backup to get from this backup destination
    backup: String,
    /// Directory to limit getting backuped directories and files
    limited_directory: Option<String>,
    /// Backup destination that directries and files are gotten from
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Parser, PartialEq)]
struct CleanArguments {
    /// Directory to limit getting backuped directories and files
    limited_count: Option<i32>,
    /// Backup destination that cleaning is executed on
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Subcommand, PartialEq)]
enum CommandKind {
    /// Initialize a backup destination into this directory
    Init(InitArguments),
    /// Backup directories and files into this directory's backup destination
    Backup(BackupArguments),
    /// Remove backups specified by given pattern
    RemoveBackup(RemoveBackupArguments),
    /// Get backuped directories and files that is specified
    Get(GetArguments),
    /// Execute cleaning for this backup destination
    Clean(CleanArguments),
}

#[derive(Parser)]
#[command(version)]
struct Arguments {
    /// Command you want to do
    #[command(subcommand)]
    command: Option<CommandKind>,
}

fn main() -> ExitCode {
    let arguments = Arguments::parse();
    let Some(command) = arguments.command else {
        println!("USAGE: ntm COMMAND");

        return ExitCode::SUCCESS;
    };

    let started = SystemTime::now();
    if let CommandKind::Init(arguments) = command {
        let mut command = InitCommand::new();
        if let Some(destination) = arguments.destination {
            command.set_destination_path(&destination);
        }
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        };
    } else if let CommandKind::Backup(arguments) = command {
        if let Some(command) = arguments.command {
            if let BackupCommandKind::Execute(arguments) = command {
                let mut command = BackupCommand::new();
                if let Some(destination) = arguments.destination {
                    command.set_destination_path(&destination);
                }
                if let Err(error) = command.execute() {
                    println!("Error caused.\n\n{}", error);

                    return ExitCode::FAILURE;
                }
            }
        }
    } else if let CommandKind::RemoveBackup(arguments) = command {
        let pattern = arguments.pattern;
        let mut command = RemoveBackupCommand::new(&pattern);
        if let Some(destination) = arguments.destination {
            command.set_destination_path(&destination);
        }
        if let Err(error) = command.execute() {
            println!("Error caused.\n\n{}", error);

            return ExitCode::FAILURE;
        }
    } else if let CommandKind::Get(arguments) = command {
        let backup = arguments.backup;
        let mut command = GetCommand::new(&backup);
        if let Some(directory) = arguments.limited_directory {
            command.set_limited_directory(&directory);
        }
        if let Some(destination) = arguments.destination {
            command.set_destination_path(&destination);
        }
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        }
    } else if let CommandKind::Clean(arguments) = command {
        let mut command = CleanCommand::new();
        if let Some(limited_count) = arguments.limited_count {
            command.set_limited_count(limited_count as i64);
        }
        if let Some(destination) = arguments.destination {
            command.set_destination_path(&destination);
        }
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        }
    }
    let ended = SystemTime::now();
    if let Ok(duration) = ended.duration_since(started) {
        let mut seconds = duration.as_secs();
        let hours = seconds / (60 * 60);
        seconds -= hours * (60 * 60);
        let minutes = seconds / 60;
        seconds -= minutes * 60;
        println!(
            "Process completed in {}:{:02}:{:02}.",
            hours, minutes, seconds
        );
    }

    ExitCode::SUCCESS
}
