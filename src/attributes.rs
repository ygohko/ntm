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

#[derive(Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub path: String,
    pub added: i64,
}

impl Attributes {
    pub fn new(path: &str, added: i64) -> Self {
        Self {
            path: path.to_string(),
            added,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::attributes::Attributes;

    #[test]
    fn is_creatable() {
        let _attributes = Attributes::new("/a/b/c/d.txt", 12345);
    }

    #[test]
    fn is_serializable() {
        let attributes = Attributes::new("/a/b/c/d.txt", 12345);
        let serialized = serde_json::to_string(&attributes).unwrap();
        assert_eq!(serialized, "{\"path\":\"/a/b/c/d.txt\",\"added\":12345}".to_string());
    }

    #[test]
    fn is_deserializable() {
        let serialized = "{ \"path\": \"/a/b/c/d.txt\", \"added\": 12345 }";
        let attributes: Attributes = serde_json::from_str(&serialized).unwrap();
        assert_eq!(attributes.path, "/a/b/c/d.txt".to_string());
        assert_eq!(attributes.added, 12345);
    }
}
