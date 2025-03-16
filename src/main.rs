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
mod backup_command;
mod commons;
mod config;
mod entry;
mod error;
mod file_path_producer;
mod gc_command;
mod get_command;
mod init_command;
mod object_store;

use clap::Parser;
use clap::Subcommand;
use std::process::ExitCode;

use crate::backup_command::BackupCommand;
use crate::gc_command::GcCommand;
use crate::get_command::GetCommand;
use crate::init_command::InitCommand;

#[derive(Parser, PartialEq)]
struct BackupArguments {
    /// Backup to get from this backup destination
    #[arg(short, long)]
    destination: Option<String>,
}

#[derive(Parser, PartialEq)]
struct GetArguments {
    /// Backup to get from this backup destination
    backup: String,
    /// Directory to limit getting backuped directories and files
    limited_directory: Option<String>,
}

#[derive(Parser, PartialEq)]
struct GcArguments {
    /// Directory to limit getting backuped directories and files
    limited_count: Option<i32>,
}

#[derive(Subcommand, PartialEq)]
enum CommandKind {
    /// Initialize a backup destination into this directory
    Init,
    /// Backup directories and files into this directory's backup destination
    Backup(BackupArguments),
    /// Get backuped directories and files that is specified
    Get(GetArguments),
    /// Execute garbage collection for this backup destination
    Gc(GcArguments),
}

#[derive(Parser)]
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

    if command == CommandKind::Init {
        let command = InitCommand::new();
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        };
    } else if let CommandKind::Backup(arguments) = command {
        let mut command = BackupCommand::new();
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        };
    } else if let CommandKind::Get(arguments) = command {
        let backup = arguments.backup;
        let mut command = GetCommand::new(&backup);
        if let Some(directory) = arguments.limited_directory {
            command.set_limited_directory(&directory);
        }
        match command.execute() {
            Ok(_) => (),
            Err(error) => {
                println!("Error caused.\n\n{}", error);

                return ExitCode::FAILURE;
            }
        }
    } else if let CommandKind::Gc(arguments) = command {
        let mut command = GcCommand::new();
        if let Some(limited_count) = arguments.limited_count {
            command.set_limited_count(limited_count as i64);
        }
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
