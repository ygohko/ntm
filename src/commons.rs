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

use camino::Utf8Path;
use camino::Utf8PathBuf;
use std::path::Path;
use std::path::PathBuf;

/// A trait for operating on file paths.
pub trait OperatePath {
    /// Returns the file name of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The file name.
    fn file_name_or_empty(&self) -> String;
    /// Returns the extension of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The extension.
    fn extension_or_empty(&self) -> String;
    /// Returns the parent directory of the path, or an empty string if not present.
    ///
    /// # Returns
    ///
    /// * `String` - The parent directory.
    fn parent_or_empty(&self) -> String;
    /// Converts the path to a `String`.
    ///
    /// # Returns
    ///
    /// * `String` - The path as a string.
    fn to_string_easy(&self) -> String;
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

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.as_str().to_string()
    }
}

impl OperatePath for Utf8Path {
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

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.as_str().to_string()
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

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl OperatePath for Path {
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

    fn parent_or_empty(&self) -> String {
        let parent = match self.parent() {
            Some(parent) => parent,
            None => return "".to_string(),
        };

        parent.to_string_easy()
    }

    fn to_string_easy(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8Path;
    use camino::Utf8PathBuf;
    use std::path::Path;
    use std::path::PathBuf;

    use crate::commons::OperatePath;

    #[test]
    fn file_name_is_gettable() {
        let path = Utf8PathBuf::from("/a/b/c.txt");
        let file_name = path.file_name_or_empty();
        assert_eq!(file_name, "c.txt");

        let path = Utf8Path::new("/a/b/c.txt");
        let file_name = path.file_name_or_empty();
        assert_eq!(file_name, "c.txt");

        let path = PathBuf::from("/a/b/c.txt");
        let file_name = path.file_name_or_empty();
        assert_eq!(file_name, "c.txt");

        let path = Path::new("/a/b/c.txt");
        let file_name = path.file_name_or_empty();
        assert_eq!(file_name, "c.txt");
    }

    #[test]
    fn extension_is_gettable() {
        let path = Utf8PathBuf::from("/a/b/c.txt");
        let extension = path.extension_or_empty();
        assert_eq!(extension, "txt");

        let path = Utf8Path::new("/a/b/c.txt");
        let extension = path.extension_or_empty();
        assert_eq!(extension, "txt");

        let path = PathBuf::from("/a/b/c.txt");
        let extension = path.extension_or_empty();
        assert_eq!(extension, "txt");

        let path = Path::new("/a/b/c.txt");
        let extension = path.extension_or_empty();
        assert_eq!(extension, "txt");
    }

    #[test]
    fn parent_directories_are_gettable() {
        let path = Utf8PathBuf::from("/a/b/c.txt");
        let string = path.to_string_easy();
        assert_eq!(string, "/a/b/c.txt");

        let path = Utf8Path::new("/a/b/c.txt");
        let string = path.to_string_easy();
        assert_eq!(string, "/a/b/c.txt");

        let path = PathBuf::from("/a/b/c.txt");
        let string = path.to_string_easy();
        assert_eq!(string, "/a/b/c.txt");

        let path = Path::new("/a/b/c.txt");
        let string = path.to_string_easy();
        assert_eq!(string, "/a/b/c.txt");
    }
}
