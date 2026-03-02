use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub binary: String,
    #[serde(default = "default_files")]
    pub files: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    pub jobs: Option<usize>,
}

fn default_files() -> String {
    "tests/**/*".to_string()
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
