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

pub struct BackupStore {
    path: String,
}

impl BackupStore {
    pub fn new(path: &str) -> Self {
        Self {
            path,
        }
    }

    pub fn names(&mut self) -> Result<Vec<String>> {
        let Ok(read_dir) = fs::read_dir(&self.path) else {
            return Err(Error::new(ERROR_ID, ERROR_CODE_FINDING_BACKUP_FAILED));
        };
        let mut names: Vec<String> = Vec::new();
        for result in read_dir {
            if let Ok(entry) = result {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        names.push(String::from_path(&entry.path()));
                    }
                }
            }
        }

        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::backup_store::BackupStore;

    #[test]
    fn is_creatable() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        fs::create_dir_all(&path).unwrap();
        let _store = BackupStore::new(&path);
    }

    #[test]
    fn is_creatable() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("Backups");
        let mut backup_path = path.clone();
        backup_path.push("11111111-1111");
        fs::create_dir_all(&backup_path).unwrap();
        let store = BackupStore::new(&path);
        let _names = store.names().unwrap();
    }
}
