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

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

#[allow(dead_code)]
pub const ERROR_ID: ErrorId = "task";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_NOT_SUPPORTED: ErrorCode = 1;
pub const ERROR_CODE_PANICKED: ErrorCode = 2;

/// Represents a task that can be executed.
pub trait Task {
    /// Executes the task.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `Error` if the execution fails.
    fn execute(&mut self) -> Result<()>;

    /// Initiates an operation or task to be executed asynchronously in the background.
    ///
    /// This method dispatches the primary work to a separate thread, an asynchronous runtime,
    /// or a similar background execution mechanism, allowing the current thread to continue
    /// without blocking. The `&mut self` indicates that initiating the background task
    /// may modify the internal state of the object, for example, to store a handle to
    /// the spawned task, update status, or manage resources.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the background operation was successfully initiated. This indicates
    ///   that the task was successfully handed off to the background execution system,
    ///   but it does *not* guarantee the successful completion of the background task itself.
    /// - `Err` if an error occurred while attempting to initiate or dispatch the
    ///   background operation. This could be due to issues like failing to spawn a thread,
    ///   the task executor being full, or other resource contention preventing the
    ///   initiation.
    fn execute_in_background(&mut self) -> Result<()> {
        Err(Error::new(ERROR_ID, ERROR_CODE_NOT_SUPPORTED))
    }

    /// Waits for the previously initiated background operation to complete.
    ///
    /// This method blocks the current thread until the background task, started by
    /// `execute_in_background`, has finished its execution. After this method
    /// returns successfully, the background task is considered completed, and
    /// resources associated with it may be deallocated.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is no background operation currently running to join.
    /// - The background operation itself encountered an unrecoverable error
    ///   and this error is propagated to the join caller.
    /// - An error occurred while waiting for the background operation to complete
    ///   (e.g., an interruption or an internal communication error).
    fn join(&mut self) -> Result<()> {
        Err(Error::new(ERROR_ID, ERROR_CODE_NOT_SUPPORTED))
    }
}
