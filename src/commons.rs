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
use std::path::Path;

// TODO: Rename this.
// TODO: Add tests.
pub trait OperatePath3 {
    fn file_name_or_empty(&self) -> String;
    fn directories(&self) -> String;
}

impl OperatePath3 for Utf8PathBuf {
    fn file_name_or_empty(&self) -> String {
        let file_name = match self.file_name() {
            Some(file_name) => file_name,
            None => return "".to_string(),
        };

        file_name.to_string()
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

// TODO: Remove this.
pub trait OperatePath {
    fn pushed(&self, path: &str) -> String;
    fn directories(&self) -> String;
    fn file_name(&self) -> String;
    fn extension(&self) -> String;
    fn is_begun(&self, path: &str) -> bool;
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

    fn extension(&self) -> String {
        let file_name = self.file_name();
        let mut split: Vec<_> = file_name.split(".").collect();
        if split.len() < 2 {
            return "".to_string();
        }

        split.pop().unwrap().to_string()
    }

    fn is_begun(&self, path: &str) -> bool {
        // TODO: Improve implementation.
        if self.find(path) == Some(0) {
            return true;
        }

        false
    }
}

pub trait ConvertPath {
    // TODO: Rename to from_lossy()?
    fn from_path(path: &dyn AsRef<Path>) -> String;
}

impl ConvertPath for String {
    fn from_path(path: &dyn AsRef<Path>) -> String {
        path.as_ref().to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;

    use crate::commons::ConvertPath;
    use crate::commons::OperatePath;

    #[test]
    fn is_pushable() {
        let path = "a";
        let path = path.pushed("b");
        assert_eq!(path, "a/b".to_string());
    }

    #[test]
    fn directories_are_gettable() {
        let path = "a/b/c/d.txt";
        let directories = path.directories();
        assert_eq!(directories, "a/b/c".to_string());

        let path = "d.txt";
        let directories = path.directories();
        assert_eq!(directories, "".to_string());
    }

    #[test]
    fn file_name_is_gettable() {
        let path = "a/b/c/d.txt";
        let file_name = path.file_name();
        assert_eq!(file_name, "d.txt".to_string());

        let path = "a.txt";
        let file_name = path.file_name();
        assert_eq!(file_name, "a.txt".to_string());
    }

    #[test]
    fn extension_is_gettable() {
        let path = "a/b/c/d.txt";
        let extension = path.extension();
        assert_eq!(extension, "txt".to_string());

        let path = "a/b/c/d";
        let extension = path.extension();
        assert_eq!(extension, "".to_string());
    }

    #[test]
    fn head_directries_are_checkable() {
        let path = "a/b/c/d.txt";
        assert_eq!(path.is_begun("a/b"), true);
        assert_eq!(path.is_begun("a/c"), false);
    }

    #[test]
    fn string_is_gettable_from_path() {
        let path_buf = Utf8PathBuf::from("a/b/c/d.txt");
        let path: String = String::from_path(&path_buf);
        assert_eq!(path, "a/b/c/d.txt".to_string());
    }
}
