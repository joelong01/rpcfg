// src/models.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct ConfigItem {
    pub description: String,
    #[serde(default)]
    pub shellscript: String,
    pub default: String,
    #[serde(default)]
    pub temp_environment_variable_name: String,
    #[serde(default)]
    pub required_as_env: bool,
    #[serde(skip)]
    pub value: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Config {
    pub stored: String,
    pub config_version: String,
    #[serde(flatten)]
    pub items: HashMap<String, ConfigItem>,
}