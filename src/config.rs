use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub binary: String,
    #[serde(default = "default_files", deserialize_with = "deserialize_files")]
    pub files: Vec<String>,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    pub jobs: Option<usize>,
}

fn deserialize_files<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }
    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multiple(v) => Ok(v),
    }
}

pub const DEFAULT_FILES_GLOB: &str = "tests/**/*";

fn default_files() -> Vec<String> {
    vec![DEFAULT_FILES_GLOB.to_string()]
}

fn default_prefix() -> String {
    "// ".to_string()
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

/// Search upward from `start` for `ouro.toml`, like Cargo does.
pub fn find_config_file(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("ouro.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
