use anyhow::Result;
use serde::Deserialize;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub content_root: Option<PathBuf>,
    pub address: Option<String>,
    #[serde(default)]
    pub implicit_index: bool,
}

impl Config {
    pub fn parse<F: AsRef<Path>>(file: F) -> Result<Self> {
        let mut f = File::open(file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }
}
