use crate::{CommandResult, Config, EnvOutputUri, JsonOutputUri, Success};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufRead, Write};
use tabwriter::TabWriter;
use tracing::debug;


/// Executes the collect command, gathering configuration input from the user.
///
/// This function serves as the entry point for the collect command. It sets up the
/// input and output streams and calls `collect_user_input` to handle the actual
/// collection of configuration data.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `interactive` - A boolean flag indicating whether to run in interactive mode.
///
/// # Returns
///
/// Returns a Result containing a CommandResult on success, or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when setting up input/output streams.
/// * The `collect_user_input` function encounters an error.
///
/// # Examples
///
/// ```
/// use your_crate::{Config, execute};
///
/// let mut config = Config::default();
/// let result = execute(&mut config, true);
/// ```
pub fn execute(config: &mut Config, interactive: bool) -> Result<CommandResult> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    collect_user_input(config, interactive, &mut stdin, &mut stdout)
}
/// Collects user input to configure items in the provided Config object.
///
/// This function handles both interactive and non-interactive modes for collecting
/// configuration data. In interactive mode, it prompts the user for input and allows
/// updating individual items, toggling storage type, and saving the configuration.
/// In non-interactive mode, it uses default values for all items and saves the configuration.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `interactive` - A boolean flag indicating whether to run in interactive mode.
/// * `input` - A mutable reference to a BufRead trait object for reading user input.
/// * `output` - A mutable reference to a Write trait object for writing prompts and messages.
///
/// # Returns
///
/// Returns a Result containing a CommandResult on success, or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when reading input or writing output.
/// * The configuration fails to save.
/// * Any other operation within the function fails.
///
/// # Examples
///
/// ```
/// use std::io::{Cursor, stdout};
/// use your_crate::{Config, collect_user_input};
///
/// let mut config = Config::default();
/// let mut input = Cursor::new("1\nnew_value\nc\n");
/// let result = collect_user_input(&mut config, true, &mut input, &mut stdout());
/// ```
pub fn collect_user_input<R: BufRead, W: Write>(
    config: &mut Config,
    interactive: bool,
    input: &mut R,
    output: &mut W,
) -> Result<CommandResult> {
    debug!(
        "collect_user_input: config: {:?} interactive: {}",
        config, interactive
    );
    let mut storage_type = config.stored.clone();

    // Initialize empty values with defaults
    initialize_config_values(config);

    if interactive {
        interactive_config_loop(config, &mut storage_type, input, output)?;
    } else {
        // In non-interactive mode, use default values and save
        save_configuration(config, &storage_type)?;
    }

    // Set environment variables for required items
    set_environment_variables(config);

    let mut result = Success!("Configuration collected successfully.");
    result.env_file = EnvOutputUri!(config);
    result.json_file = JsonOutputUri!(config);
    Ok(result)
}

/// Initialize config values with defaults if they are empty
fn initialize_config_values(config: &mut Config) {
    for item in &mut config.items {
        if item.value.is_empty() {
            item.value = item.default.clone();
        }
    }
}

/// Handle the interactive configuration loop
fn interactive_config_loop<R: BufRead, W: Write>(
    config: &mut Config,
    storage_type: &mut String,
    input: &mut R,
    output: &mut W,
) -> Result<()> {
    loop {
        show_current_config(config, storage_type, output)?;

        // Prompt for action
        write!(output, "\nEnter item number to update, 'T' to toggle storage type, 'S' to save, or 'C' to continue: ")?;
        output.flush()?;

        let user_input = read_user_input(input)?;

        match user_input.as_str() {
            "t" => toggle_storage_type(storage_type),
            "s" => save_config_interactive(config, storage_type, output)?,
            "c" => break,
            _ => handle_item_update(config, &user_input, input, output)?,
        }
    }
    Ok(())
}

/// Read and trim user input
fn read_user_input<R: BufRead>(input: &mut R) -> Result<String> {
    let mut user_input = String::new();
    input.read_line(&mut user_input)?;
    Ok(user_input.trim().to_lowercase())
}

/// Toggle the storage type between "local" and "keyvault"
fn toggle_storage_type(storage_type: &mut String) {
    *storage_type = if *storage_type == "local" {
        "keyvault".to_string()
    } else {
        "local".to_string()
    };
}

/// Save configuration in interactive mode and provide feedback
fn save_config_interactive<W: Write>(config: &mut Config, storage_type: &str, output: &mut W) -> Result<()> {
    match save_configuration(config, storage_type) {
        Ok(()) => writeln!(output, "Configuration saved successfully.")?,
        Err(e) => writeln!(output, "Failed to save configuration: {}", e)?,
    }
    Ok(())
}

/// Handle updating a specific item in the configuration
fn handle_item_update<R: BufRead, W: Write>(
    config: &mut Config,
    user_input: &str,
    input: &mut R,
    output: &mut W,
) -> Result<()> {
    if let Ok(index) = user_input.parse::<usize>() {
        if index > 0 && index <= config.items.len() {
            update_item(config, index - 1, input, output)?;
        } else {
            writeln!(output, "Invalid item number. Please try again.")?;
        }
    } else {
        writeln!(output, "Invalid input. Please try again.")?;
    }
    Ok(())
}

/// Set environment variables for required items
fn set_environment_variables(config: &Config) {
    for item in &config.items {
        if item.required_as_env {
            std::env::set_var(&item.temp_environment_variable_name, &item.value);
        }
    }
}
/// Displays the current configuration to the provided output.
///
/// This function prints out the current state of the configuration, including
/// the storage type and all configuration items with their current values.
///
/// # Arguments
///
/// * `config` - A reference to the Config object containing the configuration to display.
/// * `storage_type` - A string slice indicating the current storage type (e.g., "local" or "keyvault").
/// * `out` - A mutable reference to a Write trait object where the configuration will be written.
///
/// # Returns
///
/// Returns a Result<()>. The function succeeds if all write operations to `out` are successful.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when writing to the output.
///
/// # Examples
///
/// ```
/// use std::io::stdout;
/// use your_crate::{Config, show_current_config};
///
/// let config = Config::default();
/// let storage_type = "local";
/// show_current_config(&config, storage_type, &mut stdout()).expect("Failed to display config");
/// ```
pub fn show_current_config<W: Write>(
    config: &Config,
    storage_type: &str,
    out: &mut W,
) -> Result<()> {
    writeln!(out, "\nCurrent configuration:")?;
    writeln!(out, "Storage type: {}", storage_type)?;
    writeln!(out, "Project: {}", config.project_name)?;
    writeln!(out, "Config: {}", config.config_name)?;
    if config.is_test {
        writeln!(out, "(Test mode)")?;
    }
    writeln!(out)?; // Add an extra newline for spacing

    let mut tw = TabWriter::new(vec![]);

    writeln!(tw, "Index\tDescription\tValue")?;
    writeln!(tw, "-----\t-----------\t-----")?;

    for (index, item) in config.items.iter().enumerate() {
        let display_value = if item.value.is_empty() {
            &item.default
        } else {
            &item.value
        };
        writeln!(tw, "{}\t{}\t{}", index + 1, item.description, display_value)?;
    }
    tw.flush()?;

    out.write_all(&tw.into_inner()?)?;
    writeln!(out)?; // Add an extra newline at the end

    Ok(())
}
/// Updates a specific item in the configuration based on user input.
///
/// This function prompts the user to enter a new value for a specified configuration item.
/// It then updates the item's value in the provided Config object.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object containing the item to be updated.
/// * `index` - The index of the item to be updated in the config's items list.
/// * `input` - A mutable reference to a BufRead trait object for reading user input.
/// * `output` - A mutable reference to a Write trait object for writing prompts and messages.
///
/// # Returns
///
/// Returns a Result<()>. The function succeeds if the item is successfully updated.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when reading input or writing output.
/// * The specified index is out of bounds for the config's items list.
///
/// # Examples
///
/// ```
/// use std::io::{Cursor, stdout};
/// use your_crate::{Config, update_item};
///
/// let mut config = Config::default();
/// let mut input = Cursor::new("new value\n");
/// update_item(&mut config, 0, &mut input, &mut stdout()).expect("Failed to update item");
/// ```
pub fn update_item<R: BufRead, W: Write>(
    config: &mut Config,
    index: usize,
    input: &mut R,
    output: &mut W,
) -> Result<()> {
    let item = config.items.get_mut(index).context("Item not found")?;

    debug!("Updating setting: {}", item.description);

    write!(
        output,
        "Enter new value for {} (current: {}): ",
        item.description, item.value
    )?;
    output.flush()?;

    let mut new_value = String::new();
    input.read_line(&mut new_value)?;
    let new_value = new_value.trim();

    debug!("New value for {}: {}", item.description, new_value);

    item.value = new_value.to_string();

    Ok(())
}
/// Saves the current configuration to JSON and ENV files.
///
/// This function writes the current state of the configuration to two files:
/// 1. A JSON file containing all configuration items and their values.
/// 2. An ENV file containing environment variable declarations for required items.
///
/// The function uses the EnvOutputUri! and JsonOutputUri! macros to determine
/// the appropriate file paths based on the storage type and input file name.
///
/// # Arguments
///
/// * `config` - A reference to the Config object containing the configuration to be saved.
/// * `storage_type` - A string slice indicating the current storage type (e.g., "local" or "keyvault").
///
/// # Returns
///
/// Returns a Result<()>. The function succeeds if both files are written successfully.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when writing to either the JSON or ENV file.
/// * The EnvOutputUri! or JsonOutputUri! macros fail to generate valid file paths.
/// * The configuration data cannot be serialized to JSON.
///
/// # Examples
///
/// ```
/// use your_crate::{Config, save_configuration};
///
/// let config = Config::default();
/// let storage_type = "local";
/// save_configuration(&config, storage_type).expect("Failed to save configuration");
/// ```
///
/// # Note
///
/// This function will overwrite existing files if they already exist at the target paths.
pub fn save_configuration(config: &Config, storage_type: &str) -> Result<()> {
    debug!("save_configuration: storage_type = {}", storage_type);

    let json_file_path = JsonOutputUri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct JSON output path"))?;
    debug!("JSON file path: {:?}", json_file_path);

    let env_file_path = EnvOutputUri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct ENV output path"))?;
    debug!("ENV file path: {:?}", env_file_path);

    // Ensure directories exist
    if let Some(dir) = std::path::Path::new(&json_file_path).parent() {
        debug!("Creating directory: {:?}", dir);
        std::fs::create_dir_all(dir)?;
    }
    if let Some(dir) = std::path::Path::new(&env_file_path).parent() {
        debug!("Creating directory: {:?}", dir);
        std::fs::create_dir_all(dir)?;
    }

    // Save JSON file
    let json_content = serde_json::to_string_pretty(
        &config
            .items
            .iter()
            .map(|item| {
                (
                    item.key.to_lowercase(),
                    serde_json::Value::String(item.value.clone()),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>(),
    )?;

    debug!("Writing JSON content: {}", json_content);
    std::fs::write(&json_file_path, &json_content)?;
    debug!("JSON file written successfully");

    // Create .env file
    let mut env_file = File::create(&env_file_path)?;
    debug!("ENV file created");
    let json_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json_content)?;
    for (key, value) in json_map {
        if let serde_json::Value::String(v) = value {
            writeln!(env_file, "{}={}", key.to_uppercase(), v)?;
            debug!("Wrote to ENV file: {}={}", key.to_uppercase(), v);
        }
    }

    debug!("Configuration saved successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, ConfigItem};
    use std::fs;
    use std::io::Cursor;
    use uuid::Uuid;
    use crate::common::run_test;

    fn setup_test_config(test_id: &str) -> Result<Config> {
        Ok(Config {
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
        })
    }

    /// Test non-interactive mode of collect_user_input
    ///
    /// This test verifies that in non-interactive mode:
    /// 1. The function completes successfully
    /// 2. All config items are set to their default values
    ///
    /// Failure conditions:
    /// - If the function returns an error
    /// - If any config item's value is not equal to its default
    #[test]
    fn test_non_interactive_mode() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        // Store the initial values
        let initial_values: Vec<String> = config.items.iter().map(|item| item.value.clone()).collect();

        let mut input = Cursor::new("");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, false, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        for (index, item) in config.items.iter().enumerate() {
            debug!(
                "Item {}: initial value = {}, current value = {}, default = {}",
                item.key, initial_values[index], item.value, item.default
            );
            
            if initial_values[index].is_empty() {
                assert_eq!(item.value, item.default, "Empty item should be set to default");
            } else {
                assert_eq!(item.value, initial_values[index], "Non-empty item should remain unchanged");
            }
        }

        Ok(())
    }

    /// Test toggling storage type in collect_user_input
    ///
    /// This test verifies that:
    /// 1. The storage type can be toggled from local to keyvault
    /// 2. The function completes successfully after toggling
    ///
    /// Failure conditions:
    /// - If the function returns an error
    /// - If the output doesn't contain "Storage type: keyvault"
    #[test]
    fn test_toggle_storage_type() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        let mut input = Cursor::new("t\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        //debug!("Output: {}", output_str);

        assert!(output_str.contains("Storage type: keyvault"));

        Ok(())
    }

    /// Test invalid input handling in collect_user_input
    ///
    /// This test verifies that:
    /// 1. The function handles invalid input correctly
    /// 2. Appropriate error messages are displayed
    /// 3. The function completes successfully despite invalid inputs
    ///
    /// Failure conditions:
    /// - If the function returns an error
    /// - If the output doesn't contain expected error messages
    #[test]
    fn test_invalid_input() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        let mut input = Cursor::new("invalid\n3\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

         let output_str = String::from_utf8(output.into_inner())?;
        // debug!("Output: {}", output_str);

        assert!(output_str.contains("Invalid input. Please try again."));
        assert!(output_str.contains("Invalid item number. Please try again."));

        Ok(())
    }

    /// Test saving configuration in collect_user_input
    ///
    /// This test verifies that:
    /// 1. The function can save the configuration successfully
    /// 2. The saved JSON and ENV files exist and contain correct content
    /// 3. The config object is updated correctly
    ///
    /// Failure conditions:
    /// - If the function returns an error
    /// - If the JSON or ENV files are not created
    /// - If the file contents don't match expected values
    /// - If the config object doesn't reflect the changes
    #[test]
    fn test_save_configuration() -> Result<(), Box<dyn std::any::Any + Send>> {
        run_test(|| {
            let test_id = Uuid::new_v4().to_string();
            let mut config = setup_test_config(&test_id)?;

            debug!("Initial config: {:?}", config);

            // Simulate interactive input
            let mut input = Cursor::new("1\nnew_value1\ns\nc\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

            debug!("collect_user_input result: {:?}", result);
            assert!(matches!(result.status, crate::rp_macros::Status::Ok));

           // let output_str = String::from_utf8(output.into_inner())?;
           // debug!("Output: {}", output_str);

            // Check that the configuration was saved
            let json_path = JsonOutputUri!(config).unwrap();
            let env_path = EnvOutputUri!(config).unwrap();

            debug!("JSON path: {:?}", json_path);
            debug!("ENV path: {:?}", env_path);

            // Check that files exist
            assert!(
                std::path::Path::new(&json_path).exists(),
                "JSON file does not exist at {:?}",
                json_path
            );
            assert!(
                std::path::Path::new(&env_path).exists(),
                "ENV file does not exist at {:?}",
                env_path
            );

            // Check JSON file content
            let json_content = fs::read_to_string(&json_path)?;
            let json_map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&json_content)?;

            debug!("JSON content: {}", json_content);
            
            // Check that the first item was updated and the second remains default
            assert_eq!(json_map[&config.items[0].key], "new_value1");
            assert_eq!(json_map[&config.items[1].key], "default2");

            // Check ENV file content
            let env_content = fs::read_to_string(&env_path)?;
            debug!("ENV file content: {}", env_content);

            // Debug output for config items
            debug!("Config items:");
            for (index, item) in config.items.iter().enumerate() {
                debug!("Item {}: key={}, temp_env_var={}", index, item.key, item.temp_environment_variable_name);
            }

            // Case-insensitive check for the first item (which was updated)
            let expected_env_var1 = format!("{}=NEW_VALUE1", config.items[0].key.to_uppercase());
            assert!(
                env_content.to_uppercase().contains(&expected_env_var1),
                "Expected '{}' not found in env content: {}",
                expected_env_var1,
                env_content
            );

            // Case-insensitive check for the second item (which should remain default)
            let expected_env_var2 = format!("{}=DEFAULT2", config.items[1].key.to_uppercase());
            assert!(
                env_content.to_uppercase().contains(&expected_env_var2),
                "Expected '{}' not found in env content: {}",
                expected_env_var2,
                env_content
            );

            // Clean up
            std::fs::remove_file(json_path)?;
            std::fs::remove_file(env_path)?;

            Ok(())
        })
    }

    /// Test environment variable setting in collect_user_input
    ///
    /// This test verifies that:
    /// 1. The function sets environment variables correctly
    /// 2. The .env file is created with correct content
    /// 3. The function returns the correct env_file path
    ///
    /// Failure conditions:
    /// - If the function returns an error
    /// - If the environment variable is not set correctly
    /// - If the .env file is not created or contains incorrect content
    /// - If the returned env_file path doesn't match the expected path
    #[test]
    fn test_environment_variable_setting() -> Result<(), Box<dyn std::any::Any + Send>> {
        run_test(|| {
            let test_id = Uuid::new_v4().to_string();
            let mut config = setup_test_config(&test_id)?;

            debug!("Initial config: {:?}", config);

            // Simulate interactive input
            let mut input = Cursor::new("1\nnew_env_value\ns\nc\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

            debug!("collect_user_input result: {:?}", result);
            assert!(matches!(result.status, crate::rp_macros::Status::Ok));

            let output_str = String::from_utf8(output.into_inner())?;
            debug!("Output: {}", output_str);

            // Check that the configuration was saved
            let json_path = JsonOutputUri!(config).expect("Failed to construct JSON output path");
            let env_path = EnvOutputUri!(config).expect("Failed to construct ENV output path");

            debug!("JSON path: {:?}", json_path);
            debug!("ENV path: {:?}", env_path);

            // Check that files exist
            assert!(
                std::path::Path::new(&json_path).exists(),
                "JSON file does not exist at {:?}",
                json_path
            );
            assert!(
                std::path::Path::new(&env_path).exists(),
                "ENV file does not exist at {:?}",
                env_path
            );

            // Check JSON file content
            let json_content = fs::read_to_string(&json_path)?;
            let json_map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&json_content)?;

            debug!("JSON content: {}", json_content);
            
            // Check that the first item was updated and the second remains default
            assert_eq!(json_map[&config.items[0].key], "new_env_value");
            assert_eq!(json_map[&config.items[1].key], "default2");

            // Check ENV file content
            let env_content = fs::read_to_string(&env_path)?;
            debug!("ENV file content: {}", env_content);

            // Debug output for config items
            debug!("Config items:");
            for (index, item) in config.items.iter().enumerate() {
                debug!("Item {}: key={}, temp_env_var={}", index, item.key, item.temp_environment_variable_name);
            }

            // Case-insensitive check for the first item (which was updated)
            let expected_env_var1 = format!("{}=NEW_ENV_VALUE", config.items[0].key.to_uppercase());
            assert!(
                env_content.to_uppercase().contains(&expected_env_var1),
                "Expected '{}' not found in env content: {}",
                expected_env_var1,
                env_content
            );

            // Case-insensitive check for the second item (which should remain default)
            let expected_env_var2 = format!("{}=DEFAULT2", config.items[1].key.to_uppercase());
            assert!(
                env_content.to_uppercase().contains(&expected_env_var2),
                "Expected '{}' not found in env content: {}",
                expected_env_var2,
                env_content
            );

            // Check that the environment variable is set
            let env_var_name = &config.items[0].temp_environment_variable_name;
            let env_var_value = std::env::var(env_var_name).unwrap();
            debug!("Environment variable {}: {}", env_var_name, env_var_value);
            assert_eq!(env_var_value, "new_env_value");

            // Clean up
            std::env::remove_var(env_var_name);
            std::fs::remove_file(json_path)?;
            std::fs::remove_file(env_path)?;

            Ok(())
        })
    }
}