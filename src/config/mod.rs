use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("configuration error: {}", .0)]
    ConfigError(String),
    #[error("serde_json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("std::io error")]
    StdIoError(#[from] std::io::Error),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize, Debug, PartialEq)]
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_test1() {
//         let text = r#"
//             {
//                 "entries": [
//                     {"input": "/path/input", "output": "/path/output", "remove_files": true}
//                 ]
//             }
//         "#;

//         assert_eq!(
//             serde_json::from_str::<Config>(text).unwrap(),
//             Config {
//                 entries: vec![Entry {
//                     input: "/path/input".into(),
//                     output: "/path/output".into(),
//                     remove_files: true
//                 }]
//             }
//         )
//     }
// }
