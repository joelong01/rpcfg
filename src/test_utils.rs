use crate::models::{Config, ConfigItem};

pub fn create_test_config(test_id: &str) -> Config {
    Config {
        is_test: true,
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
