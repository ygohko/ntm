use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub source_path: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            source_path: "".to_string(),
        }
    }
}
