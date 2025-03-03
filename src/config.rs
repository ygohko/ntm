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

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub source_path: String,
    pub bytes_id_threshold_min: Option<i64>,
    pub bytes_id_threshold_max: Option<i64>,
    pub excluded_directories: Option<Vec<String>>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            source_path: "".to_string(),
            bytes_id_threshold_min: Some(0),
            bytes_id_threshold_max: Some(100 * 1024 * 1024),
            excluded_directories: Some(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn is_creatable() {
        let config = Config::new();
        assert_eq!(config.source_path, "".to_string());
    }

    #[test]
    fn is_deserializable() {
        let serialized = "{ \"source_path\": \"/a/b/c\" }";
        let config: Config = match serde_json::from_str(&serialized) {
            Ok(config) => config,
            Err(_) => {
                assert!(false);

                Config::new()
            }
        };
        assert_eq!(config.source_path, "/a/b/c".to_string());

        let serialized = "{ \"source_path\": \"/a/b/c\", \"bytes_id_threshold_min\": 123 }";
        let config: Config = match serde_json::from_str(&serialized) {
            Ok(config) => config,
            Err(_) => {
                assert!(false);

                Config::new()
            }
        };
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(config.bytes_id_threshold_min, Some(123));

        let serialized = "{ \"source_path\": \"/a/b/c\", \"bytes_id_threshold_max\": 456 }";
        let config: Config = match serde_json::from_str(&serialized) {
            Ok(config) => config,
            Err(_) => {
                assert!(false);

                Config::new()
            }
        };
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(config.bytes_id_threshold_max, Some(456));

        let serialized = "{ \"source_path\": \"/a/b/c\", \"excluded_directories\": [ \"d\" ] }";
        let config: Config = match serde_json::from_str(&serialized) {
            Ok(config) => config,
            Err(_) => {
                assert!(false);

                Config::new()
            }
        };
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(config.excluded_directories, Some(vec!["d".to_string()]));
    }
}
