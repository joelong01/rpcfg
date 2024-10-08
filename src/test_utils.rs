use crate::models::{Config, ConfigItem};

pub fn create_test_config(test_id: &str) -> Config {
       Config {
            stored: "local".to_string(),
            config_version: "1.0".to_string(),
            project_name: "test_project".to_string(),
            config_name: format!("test_config_{}", test_id),
            is_test: true,
            items: vec![
                ConfigItem {
                    key: format!("item1_{}", test_id),
                    description: "Test item 1".to_string(),
                    shellscript: "".to_string(),
                    default: "default1".to_string(),
                    temp_environment_variable_name: format!("TEST_ITEM_1_{}", test_id),
                    required_as_env: true,
                    value: "initial_value1".to_string(), // Set a real initial value
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
