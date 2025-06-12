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

use camino::Utf8PathBuf;
use std::path;
use std::path::PathBuf;

// TODO: Add tests.
// TODO: Add to_string_easy().
// TODO: directories() should be migrated to parent()?
pub trait OperatePath {
    fn file_name_or_empty(&self) -> String;
    fn extension_or_empty(&self) -> String;
    fn directories(&self) -> String;
}

impl OperatePath for Utf8PathBuf {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name,
            None => return "".to_string(),
        };

        file_name.to_string()
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension,
            None => return "".to_string(),
        };

        extension.to_string()
    }

    fn directories(&self) -> String {
        let path = self.as_str().to_string();
        let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
        if split.len() < 1 {
            return "".to_string();
        }
        split.pop();

        split.join(path::MAIN_SEPARATOR_STR)
    }
}

impl OperatePath for PathBuf {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        file_name
    }

    fn extension_or_empty(&self) -> String {
        let extension = match self.extension() {
            Some(extension) => extension.to_string_lossy().to_string(),
            None => return "".to_string(),
        };

        extension
    }

    fn directories(&self) -> String {
        let path = self.to_string_lossy().to_string();
        let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
        if split.len() < 1 {
            return "".to_string();
        }
        split.pop();

        split.join(path::MAIN_SEPARATOR_STR)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests for new OperatePath trait.
}
