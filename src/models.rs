use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct ConfigItem {
    pub key: String,
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
    pub project_name: String,
    pub config_name: String,
    #[serde(default)]
    pub is_test: bool,
    pub items: Vec<ConfigItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Ok,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandResult {
    pub status: Status,
    pub message: String,
    pub env_file: Option<String>,
    pub json_file: Option<String>,
}
