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

use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::thread::JoinHandle;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;
use crate::task::Task;

pub const ERROR_ID: ErrorId = "backup_command";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;

pub struct BackgroundExecuter {
    // TODO: Add send() method.
    pub sender: Option<SyncSender<Box<dyn Task + Send>>>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl Task for BackgroundExecuter {
    fn execute(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::sync_channel(5000);
        self.sender = Some(sender);

        let handle = thread::spawn(move || {
            let mut result: Result<()> = Ok(());
            let mut done = false;
            while !done {
                if let Ok(mut task) = receiver.recv() {
                    if let Err(error) = task.execute() {
                        println!("error: {}", error);
                    }
                } else {
                    // TODO: Add a error code.
                    result = Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL));
                    done = true;
                }
            }

            result
        });
        self.handle = Some(handle);

        Ok(())
    }
}

impl BackgroundExecuter {
    pub fn new() -> Self {
        Self {
            sender: None,
            handle: None,
        }
    }

    pub fn terminate(&mut self) -> Result<()> {
        if self.handle.is_none() {
            return Ok(());
        }
        self.sender = None;
        let handle = self.handle.take();
        if let Err(_) = handle.unwrap().join() {
            // TODO: Add a error code.
            return Err(Error::new(ERROR_ID, ERROR_CODE_GENERAL));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::background_executer::BackgroundExecuter;
    use crate::task::Task;

    #[test]
    fn is_creatable() {
        let _executer = BackgroundExecuter::new();
    }

    #[test]
    fn is_executable() {
        let mut executer = BackgroundExecuter::new();
        executer.execute().unwrap();
        executer.terminate().unwrap();
    }
}
