use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
  pub api_key: String,
  pub owner_email: String,
  pub owner_id: String,
}

impl Config {
  pub fn path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap();
    let config_file = config_dir.join("rendrok").join("config.json");

    config_file
  }

  pub fn load() -> Self {
    let config_file = Self::path();

    if config_file.exists() {
      serde_json::from_slice(std::fs::read(config_file).unwrap().as_slice()).unwrap()
    } else {
      Self::default()
    }
  }

  pub fn save(&self) {
    let config_file = Self::path();

    if !config_file.exists() {
      std::fs::create_dir_all(config_file.parent().unwrap()).unwrap();
    }

    std::fs::write(config_file, serde_json::to_vec_pretty(&self).unwrap()).unwrap();
  }
}
