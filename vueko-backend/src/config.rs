use std::{path::PathBuf, sync::Arc};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct VuekoConfig {
    pub user: String,
    pub tmi_password: String,
    pub channels: Vec<String>,
    pub database: String,
}

pub fn from_path(path: PathBuf) -> eyre::Result<Config> {
    Ok(Arc::new(toml::from_str::<VuekoConfig>(
        &std::fs::read_to_string(path)?,
    )?))
}

pub type Config = Arc<VuekoConfig>;
