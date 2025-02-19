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
        let count = arguments.len();
        if count < 3 {
            println!("Missing BACKUP argument.");

            return ExitCode::FAILURE;
        }
        let backup = arguments[2].clone();
        let mut command = GetCommand::new(&backup);
        if count >= 4 {
            let path = arguments[3].clone();
            command.set_path(&path);
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
