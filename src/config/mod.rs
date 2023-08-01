use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("configuration error: {}", .0)]
    ConfigError(String),
    #[error("serde_json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("std::io error")]
    StdIoError(#[from] std::io::Error),
}

#[derive(Deserialize)]
pub struct Config {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize)]
pub struct Entry {
    pub input: String,
    pub output: String,
    pub remove_files: bool,
}

impl Config {
    pub fn from_file(filename: &str) -> Result<Config, ConfigError> {
        Ok(serde_json::from_str(
            fs::read_to_string(filename)?.as_str(),
        )?)
    }
}
