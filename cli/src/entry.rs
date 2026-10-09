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

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Represents an entry in the backup, containing metadata about a file.
#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: String,
    pub last_modified: u64,
    pub permission: u32,
    pub uid: u32,
    pub gid: u32,
}

#[cfg(test)]
mod tests {
    use crate::entry::Entry;

    #[test]
    fn is_serializable() {
        let entry = Entry {
            id: "abc123".to_string(),
            last_modified: 123,
            permission: 456,
            uid: 789,
            gid: 123,
        };
        let Ok(_serialized) = serde_json::to_string(&entry) else {
            panic!();
        };
    }

    #[test]
    fn is_deserializable() {
        let serialized = "{ \"id\": \"abc123\", \"last_modified\": 123, \"permission\": 456, \"uid\": 789, \"gid\": 123 }".to_string();
        let _entry: Entry = match serde_json::from_str(&serialized) {
            Ok(entry) => entry,
            Err(_) => panic!(),
        };
    }
}
