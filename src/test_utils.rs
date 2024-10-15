use crate::models::{Config, ConfigItem};
use std::fs;

use serde_json;

pub fn create_test_config(test_id: &str) -> Config {
    Config {
        is_test: true,
        input_file: format!("test_input_{}.json", test_id),
        rpcfg: vec![
            ConfigItem {
                key: "stored".to_string(),
                description: "Storage type for configuration".to_string(),
                shellscript: "".to_string(),
                default: "local".to_string(),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: "local".to_string(), // Set a default value
            },
            ConfigItem {
                key: "config_version".to_string(),
                description: "Version of the configuration".to_string(),
                shellscript: "".to_string(),
                default: "1.0".to_string(),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: "1.0".to_string(), // Set a default value
            },
            ConfigItem {
                key: "project_name".to_string(),
                description: "Name of the project".to_string(),
                shellscript: "".to_string(),
                default: format!("project_{}", test_id),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: format!("project_{}", test_id), // Set a default value
            },
            ConfigItem {
                key: "config_name".to_string(),
                description: "Name of the configuration".to_string(),
                shellscript: "".to_string(),
                default: format!("config_{}", test_id),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: format!("config_{}", test_id), // Set a default value
            },
            ConfigItem {
                key: "environment".to_string(),
                description: "Environment for the configuration".to_string(),
                shellscript: "".to_string(),
                default: format!("env_{}", test_id),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: format!("env_{}", test_id), // Set a default value
            },
        ],
        app: vec![
            ConfigItem {
                key: format!("item1_{}", test_id),
                description: "Test item 1".to_string(),
                shellscript: "".to_string(),
                default: "default1".to_string(),
                temp_environment_variable_name: format!("TEST_ITEM_1_{}", test_id),
                required_as_env: true,
                value: "initial_value1".to_string(),
            },
            ConfigItem {
                key: format!("item2_{}", test_id),
                description: "Test item 2".to_string(),
                shellscript: "".to_string(),
                default: "default2".to_string(),
                temp_environment_variable_name: "".to_string(),
                required_as_env: false,
                value: "".to_string(),
            },
        ],
    }
}

#[macro_export]
macro_rules! create_test_input_file {
    ($seed:expr) => {{
        let test_id = format!("{}-{}", $seed, uuid::Uuid::new_v4());
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let input_path = temp_dir.path().join(format!("input-{}.json", test_id));
        
        // Create test config
        let mut config = crate::create_test_config(&test_id);
        config.input_file = input_path.to_str().unwrap().to_string();
        
        // Save the config to the file
        let config_json = serde_json::to_string_pretty(&config)
            .expect("Failed to serialize config");
        std::fs::write(&input_path, config_json)
            .expect("Failed to write config to file");
        
        // Return both the config and the TempDir
        (config, temp_dir)
    }};
}
