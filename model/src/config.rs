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

use std::fs;
use std::path::Path;
use serde::Deserialize;

use crate::error::Error;
use crate::error::ErrorCode;
use crate::error::ErrorId;
use crate::error::Result;

pub const ERROR_ID: ErrorId = "config";

#[allow(dead_code)]
pub const ERROR_CODE_GENERAL: ErrorCode = 0;
pub const ERROR_CODE_READING_CONFIG_FAILED: ErrorCode = 1;

/// Represents the application configuration.
#[derive(Deserialize)]
pub struct Config {
    pub source_path: String,
    pub excluded_directories: Option<Vec<String>>,
    pub cache_size: Option<u32>,
}

impl Config {
    /// Creates a new `Config` instance with default values.
    ///
    /// # Returns
    ///
    /// * `Config` - A new `Config` instance.
    pub fn new() -> Self {
        Self {
            source_path: "".to_string(),
            excluded_directories: Some(vec![]),
            cache_size: Some(8),
        }
    }

    /// Loads a `Config` instance from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Config`, or an `Error` with
    /// `ERROR_CODE_READING_CONFIG_FAILED` if the file cannot be read, is not
    /// valid UTF-8, or cannot be parsed as a `Config`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let string = match String::from_utf8(bytes) {
            Ok(string) => string,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };
        let config = match toml::from_str(&string) {
            Ok(config) => config,
            Err(_) => return Err(Error::new(ERROR_ID, ERROR_CODE_READING_CONFIG_FAILED)),
        };

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempdir::TempDir;

    use crate::config;
    use crate::config::Config;

    #[test]
    fn is_creatable() {
        let config = Config::new();
        assert_eq!(config.source_path, "".to_string());
        assert_eq!(config.cache_size, Some(8));
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

        let serialized = "{ \"source_path\": \"/a/b/c\", \"excluded_directories\": [ \"d\" ], \"cache_size\": 16 }";
        let config: Config = match serde_json::from_str(&serialized) {
            Ok(config) => config,
            Err(_) => {
                assert!(false);

                Config::new()
            }
        };
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(config.excluded_directories, Some(vec!["d".to_string()]));
        assert_eq!(config.cache_size, Some(16));
    }

    #[test]
    fn is_loadable() {
        let temp_dir = TempDir::new("test").unwrap();

        let path = temp_dir.path().join("config1.toml");
        fs::write(&path, "source_path = \"/a/b/c\"\n").unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(config.excluded_directories, None);
        assert_eq!(config.cache_size, None);

        let path = temp_dir.path().join("config2.toml");
        fs::write(
            &path,
            "source_path = \"/a/b/c\"\nexcluded_directories = [ \"d\", \"e\" ]\ncache_size = 16\n",
        )
        .unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.source_path, "/a/b/c".to_string());
        assert_eq!(
            config.excluded_directories,
            Some(vec!["d".to_string(), "e".to_string()])
        );
        assert_eq!(config.cache_size, Some(16));
    }

    #[test]
    fn fails_to_load_nonexistent_file() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("nonexistent.toml");
        match Config::load(&path) {
            Ok(_) => assert!(false),
            Err(error) => {
                assert_eq!(error.id, config::ERROR_ID);
                assert_eq!(error.code, config::ERROR_CODE_READING_CONFIG_FAILED);
            }
        }
    }

    #[test]
    fn fails_to_load_non_utf8_file() {
        let temp_dir = TempDir::new("test").unwrap();
        let path = temp_dir.path().join("config.toml");
        fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
        match Config::load(&path) {
            Ok(_) => assert!(false),
            Err(error) => {
                assert_eq!(error.id, config::ERROR_ID);
                assert_eq!(error.code, config::ERROR_CODE_READING_CONFIG_FAILED);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_toml() {
        let temp_dir = TempDir::new("test").unwrap();

        let path = temp_dir.path().join("config1.toml");
        fs::write(&path, "source_path = \n").unwrap();
        match Config::load(&path) {
            Ok(_) => assert!(false),
            Err(error) => {
                assert_eq!(error.id, config::ERROR_ID);
                assert_eq!(error.code, config::ERROR_CODE_READING_CONFIG_FAILED);
            }
        }

        // Missing the required source_path field.
        let path = temp_dir.path().join("config2.toml");
        fs::write(&path, "cache_size = 16\n").unwrap();
        match Config::load(&path) {
            Ok(_) => assert!(false),
            Err(error) => {
                assert_eq!(error.id, config::ERROR_ID);
                assert_eq!(error.code, config::ERROR_CODE_READING_CONFIG_FAILED);
            }
        }
    }
}
