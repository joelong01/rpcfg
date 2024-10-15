use serde::{Deserialize, Serialize};
use anyhow::Error;

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
    //  this is here because we need a convinient place to collect the values from the user. we never store
    //  the values in the input file, but rather in the output files.
    #[serde(skip)]
    pub value: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Config {
    pub rpcfg: Vec<ConfigItem>,
    pub app: Vec<ConfigItem>,
    #[serde(skip)]
    pub is_test: bool,
}

impl Config {
    /// Get all settings (ConfigItems) with a given key
    ///
    /// This method searches for ConfigItems with the given key in both the rpcfg and app arrays.
    /// It returns a vector of references to all matching ConfigItems.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key of the settings to retrieve
    ///
    /// # Returns
    ///
    /// * `Vec<&ConfigItem>` - A vector of references to all matching ConfigItems
    ///

    pub fn get_settings(&self, key: &str) -> Vec<&ConfigItem> {
        self.rpcfg
            .iter()
            .chain(self.app.iter())
            .filter(|item| item.key == key)
            .collect()
    }

    /// Get mutable references to all settings (ConfigItems) with a given key
    ///
    /// This method searches for ConfigItems with the given key in both the rpcfg and app arrays.
    /// It returns a vector of mutable references to all matching ConfigItems.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key of the settings to retrieve
    ///
    /// # Returns
    ///
    /// * `Vec<&mut ConfigItem>` - A vector of mutable references to all matching ConfigItems
    pub fn get_settings_mut(&mut self, key: &str) -> Vec<&mut ConfigItem> {
        let mut results = Vec::new();
        for item in self.rpcfg.iter_mut().chain(self.app.iter_mut()) {
            if item.key == key {
                results.push(item);
            }
        }
        results
    }

    pub fn validate_rpcfg_config(&mut self) -> Result<(), Error> {
        for item in self.rpcfg.iter_mut() {
            if item.value.is_empty() {
                tracing::debug!(
                    "RPCFG item {} has no value, setting to default {}",
                    item.key,
                    item.default
                );
                item.value = item.default.clone();
            }
        }

        // Check if "stored" setting exists, if not, add it
        if let Some(stored_setting) = self.rpcfg.iter_mut().find(|item| item.key == "stored") {
            if stored_setting.value != "local" && stored_setting.value != "keyvault" {
                tracing::warn!(
                    "Invalid 'stored' value: '{}', setting to 'local'",
                    stored_setting.value
                );
                stored_setting.value = "local".to_string();
            }
        } else {
            tracing::info!("'stored' setting not found, adding with value 'local'");
            self.rpcfg.push(ConfigItem {
                key: "stored".to_string(),
                description: "Storage type for configuration".to_string(),
                shellscript: "".to_string(),
                default: "local".to_string(),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: "local".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpcfg: vec![
                ConfigItem {
                    key: "stored".to_string(),
                    description: "Storage type for configuration".to_string(),
                    shellscript: "".to_string(),
                    default: "local".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "local".to_string(),
                },
                ConfigItem {
                    key: "config_version".to_string(),
                    description: "Version of the configuration".to_string(),
                    shellscript: "".to_string(),
                    default: "1.0".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "1.0".to_string(),
                },
                ConfigItem {
                    key: "project_name".to_string(),
                    description: "Name of the project".to_string(),
                    shellscript: "".to_string(),
                    default: "rpcfg".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "default_project_name".to_string(),
                },
                ConfigItem {
                    key: "config_name".to_string(),
                    description: "Name of the configuration".to_string(),
                    shellscript: "".to_string(),
                    default: "rpcfg_config".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "default_config_name".to_string(),
                },
                ConfigItem {
                    key: "environment".to_string(),
                    description: "Environment for the configuration".to_string(),
                    shellscript: "".to_string(),
                    default: "development".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "default_env".to_string(),
                },
            ],
            app: Vec::new(),
            is_test: false,
        }
    }
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
