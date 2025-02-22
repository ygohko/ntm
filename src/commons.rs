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

use std::path;
use std::path::Path;
use std::path::PathBuf;

pub trait OperatePath {
    fn pushed(&self, path: &str) -> String;
    fn directories(&self) -> String;
    fn file_name(&self) -> String;
    #[allow(dead_code)]
    fn to_path_buf(&self) -> PathBuf;
}

impl OperatePath for str {
    fn pushed(&self, path: &str) -> String {
        let result = self.to_string() + path::MAIN_SEPARATOR_STR + path;

        result
    }

    fn directories(&self) -> String {
        let mut split: Vec<_> = self.split(path::MAIN_SEPARATOR_STR).collect();
        if split.len() < 1 {
            return "".to_string();
        }
        split.pop();

        split.join(path::MAIN_SEPARATOR_STR)
    }

    fn file_name(&self) -> String {
        let mut split: Vec<_> = self.split(path::MAIN_SEPARATOR_STR).collect();
        if split.len() < 1 {
            return "".to_string();
        }

        split.pop().unwrap().to_string()
    }

    fn to_path_buf(&self) -> PathBuf {
        let mut result = PathBuf::new();
        result.push(self);

        result
    }
}

pub trait ConvertPath {
    fn from_path(path: &dyn AsRef<Path>) -> String;
}

impl ConvertPath for String {
    fn from_path(path: &dyn AsRef<Path>) -> String {
        path.as_ref().to_string_lossy().to_string()
    }
}
