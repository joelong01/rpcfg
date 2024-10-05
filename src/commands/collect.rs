use crate::{Config, CommandResult, Success, Fail, EnvOutputUri, JsonOutputUri};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufRead, Write};
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
/// * `input_file` - The name of the input file, used for generating output file paths.
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
/// let result = execute(&mut config, true, "config.json");
/// ```
pub fn execute(config: &mut Config, interactive: bool, input_file: &str) -> Result<CommandResult> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    collect_user_input(config, interactive, input_file, &mut stdin, &mut stdout)
}
/// Collects user input to configure items in the provided Config object.
///
/// This function handles both interactive and non-interactive modes for collecting
/// configuration data. In interactive mode, it prompts the user for input and allows
/// updating individual items, toggling storage type, and saving the configuration.
/// In non-interactive mode, it uses default values for all items.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `interactive` - A boolean flag indicating whether to run in interactive mode.
/// * `input_file` - The name of the input file, used for generating output file paths.
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
/// * The configuration fails to save when requested.
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
/// let result = collect_user_input(&mut config, true, "config.json", &mut input, &mut stdout());
/// ```

fn collect_user_input<R: BufRead, W: Write>(
    config: &mut Config,
    interactive: bool,
    input_file: &str,
    input: &mut R,
    output: &mut W,
) -> Result<CommandResult> {
    let mut storage_type = config.stored.clone();

    // Initialize empty values with defaults
    for item in config.items.values_mut() {
        if item.value.is_empty() {
            item.value = item.default.clone();
        }
    }

    loop {
        // Display current configuration
        show_current_config(config, &storage_type, output)?;

        if !interactive {
            // If not interactive, just use default values and exit
            for item in config.items.values_mut() {
                item.value = item.default.clone();
            }
            break;
        }

        // Prompt for action
        write!(output, "\nEnter item number to update, 'T' to toggle storage type, 'S' to save, or 'C' to continue: ")?;
        output.flush()?;

        let mut user_input = String::new();
        input.read_line(&mut user_input)?;
        let user_input = user_input.trim().to_lowercase();

        match user_input.as_str() {
            "t" => {
                storage_type = if storage_type == "local" {
                    "keyvault".to_string()
                } else {
                    "local".to_string()
                };
            }
            "s" => match save_configuration(config, &storage_type, input_file) {
                Ok(()) => {
                    writeln!(output, "Configuration saved.")?;
                    for (key, item) in &config.items {
                        debug!("Saved setting: {} = {}", key, item.value);
                    }
                }
                Err(e) => {
                    writeln!(output, "Failed to save configuration: {}", e)?;
                    return Ok(Fail!("Failed to save configuration: {}", e));
                }
            },
            "c" => break,
            _ => {
                if let Ok(index) = user_input.parse::<usize>() {
                    if index > 0 && index <= config.items.len() {
                        update_item(config, index - 1, input, output)?;
                    } else {
                        writeln!(output, "Invalid item number. Please try again.")?;
                    }
                } else {
                    writeln!(output, "Invalid input. Please try again.")?;
                }
            }
        }
    }

    // Set environment variables for required items
    for item in config.items.values() {
        if item.required_as_env {
            std::env::set_var(&item.temp_environment_variable_name, &item.value);
        }
    }

    let mut result = Success!("Configuration collected successfully.");
    result.env_file = EnvOutputUri!(&storage_type, input_file);
    result.json_file = JsonOutputUri!(&storage_type, input_file);
    Ok(result)
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
fn show_current_config<W: Write>(config: &Config, storage_type: &str, out: &mut W) -> Result<()> {
    writeln!(
        out,
        "\nCurrent configuration (Storage type: {}):",
        storage_type
    )?;
    for (index, (_key, item)) in config.items.iter().enumerate() {
        let display_value = if item.value.is_empty() {
            &item.default
        } else {
            &item.value
        };
        writeln!(
            out,
            "{}. {} = {}",
            index + 1,
            item.description,
            display_value
        )?;
    }
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
fn update_item<R: BufRead, W: Write>(
    config: &mut Config,
    index: usize,
    input: &mut R,
    output: &mut W,
) -> Result<()> {
    let key = config
        .items
        .keys()
        .nth(index)
        .context("Item not found")?
        .clone();
    let item = config.items.get_mut(&key).context("Item not found")?;

    debug!("Updating setting: {}", item.description);

    write!(
        output,
        "Enter new value for {} [{}]: ",
        item.description, item.default
    )?;
    output.flush()?;

    let mut user_input = String::new();
    input.read_line(&mut user_input)?;
    let user_input = user_input.trim();

    let new_value = if user_input.is_empty() {
        item.default.clone()
    } else {
        user_input.to_string()
    };

    debug!("New value for {}: {}", item.description, new_value);

    item.value = new_value;
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
/// * `input_file` - The name of the input file, used for generating output file paths.
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
/// let input_file = "config.json";
/// save_configuration(&config, storage_type, input_file).expect("Failed to save configuration");
/// ```
///
/// # Note
///
/// This function will overwrite existing files if they already exist at the target paths.
fn save_configuration(config: &Config, storage_type: &str, input_file: &str) -> Result<()> {
    debug!("save_configuration: storage_type = {}, input_file = {}", storage_type, input_file);
    
    let json_file_path = JsonOutputUri!(storage_type, input_file)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct JSON output path"))?;
    debug!("JSON file path: {:?}", json_file_path);
    
    let env_file_path = EnvOutputUri!(storage_type, input_file)
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
            .map(|(k, v)| (k.to_lowercase(), serde_json::Value::String(v.value.clone())))
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
    use crate::ConfigItem;
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::{collections::HashMap, io::Cursor};
    use tempfile::{NamedTempFile, TempDir};
    use uuid::Uuid;
    use std::sync::atomic::Ordering;

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    fn setup_test_config(test_id: &str) -> Result<(Config, NamedTempFile)> {
        let mut config = Config {
            config_version: String::from("1.0"),
            stored: String::from("local"),
            items: HashMap::new(),
        };

        config.items.insert(
            format!("item1_{}", test_id),
            ConfigItem {
                description: "Item 1".to_string(),
                default: "default1".to_string(),
                shellscript: String::new(),
                temp_environment_variable_name: format!("TEST_ITEM_1_{}", test_id),
                required_as_env: true,
                value: String::new(),
            },
        );

        config.items.insert(
            format!("item2_{}", test_id),
            ConfigItem {
                description: "Item 2".to_string(),
                default: "default2".to_string(),
                shellscript: String::new(),
                temp_environment_variable_name: String::new(),
                required_as_env: false,
                value: String::new(),
            },
        );

        // Create a temporary file
        let mut temp_file = NamedTempFile::new()?;

        // Write the config to the temporary file
        let config_json = serde_json::to_string_pretty(&config)?;
        writeln!(temp_file, "{}", config_json)?;

        Ok((config, temp_file))
    }
    // Helper function to set up a test environment
    fn setup_test_environment() -> (TempDir, PathBuf, String) {
        std::env::set_var("RP_TEST_MODE", "true");
        let test_id = Uuid::new_v4().to_string();
        let test_number = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        debug!("Test number: {}", test_number);
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();
        let home_var = format!("HOME_{}", test_id);
        std::env::set_var(&home_var, temp_home.to_str().unwrap());
        debug!("Test environment: HOME = {:?}", temp_home);
        (temp_dir, temp_home, home_var)
    }

    // Helper function to clean up the test environment
    fn cleanup_test_environment(temp_dir: TempDir, home_var: &str) {
        std::env::remove_var(home_var);
        std::env::remove_var("RP_TEST_MODE");
        temp_dir.close().unwrap();
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
        let (temp_dir, _, home_var) = setup_test_environment();
        let test_id = Uuid::new_v4().to_string();
        crate::rp_macros::VERBOSE.store(true, std::sync::atomic::Ordering::SeqCst);

        let (mut config, temp_input_file) = setup_test_config(&test_id)?;
        let input_file = temp_input_file.path().to_str().unwrap().to_string();

        let mut input = Cursor::new("");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, false, &input_file, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        for (key, item) in &config.items {
            debug!(
                "Item {}: value = {}, default = {}",
                key, item.value, item.default
            );
            assert_eq!(item.value, item.default);
        }

        cleanup_test_environment(temp_dir, &home_var);
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
    #[serial]
    fn test_toggle_storage_type() -> Result<()> {
        let (temp_dir, _, home_var) = setup_test_environment();
        let test_id = Uuid::new_v4().to_string();
        crate::rp_macros::VERBOSE.store(true, std::sync::atomic::Ordering::SeqCst);

        let (mut config, temp_input_file) = setup_test_config(&test_id)?;
        let input_file = temp_input_file.path().to_str().unwrap().to_string();

        let mut input = Cursor::new("t\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &input_file, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        debug!("Output: {}", output_str);

        assert!(output_str.contains("Storage type: keyvault"));

        cleanup_test_environment(temp_dir, &home_var);
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
    #[serial]
    fn test_invalid_input() -> Result<()> {
        let (temp_dir, _, home_var) = setup_test_environment();
        let test_id = Uuid::new_v4().to_string();
        crate::rp_macros::VERBOSE.store(true, std::sync::atomic::Ordering::SeqCst);

        let (mut config, temp_input_file) = setup_test_config(&test_id)?;
        let input_file = temp_input_file.path().to_str().unwrap().to_string();

        let mut input = Cursor::new("invalid\n3\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &input_file, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        debug!("Output: {}", output_str);

        assert!(output_str.contains("Invalid input. Please try again."));
        assert!(output_str.contains("Invalid item number. Please try again."));

        cleanup_test_environment(temp_dir, &home_var);
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
    fn test_save_configuration() -> Result<()> {
        let (temp_dir, _, home_var) = setup_test_environment();
        let test_id = Uuid::new_v4().to_string();
        crate::rp_macros::VERBOSE.store(true, std::sync::atomic::Ordering::SeqCst);

        let (mut config, temp_input_file) = setup_test_config(&test_id)?;
        let input_file = temp_input_file.path().to_str().unwrap().to_string();

        let mut input = Cursor::new(format!("1\nnew_value1\ns\nc\n"));
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &input_file, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let json_path =
            JsonOutputUri!("local", &input_file).expect("Failed to construct JSON output path");
        let env_path =
            EnvOutputUri!("local", &input_file).expect("Failed to construct ENV output path");

        debug!("JSON path: {:?}", json_path);
        debug!("ENV path: {:?}", env_path);

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
        let json_content = fs::read_to_string(&json_path)?;
        let json_map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&json_content)?;

        debug!("JSON content: {}", json_content);
        debug!("Config items: {:?}", config.items);

        assert_eq!(json_map[&format!("item1_{}", test_id)], "new_value1");
        assert_eq!(json_map[&format!("item2_{}", test_id)], "default2");

        assert_eq!(
            config.items[&format!("item1_{}", test_id)].value,
            "new_value1"
        );
        assert_eq!(
            config.items[&format!("item2_{}", test_id)].value,
            "default2"
        );

        // The temp_input_file will be automatically deleted when it goes out of scope
        cleanup_test_environment(temp_dir, &home_var);
        Ok(())
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
    #[serial]
    fn test_environment_variable_setting() -> Result<()> {
        let (temp_dir, _, home_var) = setup_test_environment();
        let test_id = Uuid::new_v4().to_string();
        crate::rp_macros::VERBOSE.store(true, std::sync::atomic::Ordering::SeqCst);

        let (mut config, temp_input_file) = setup_test_config(&test_id)?;
        let input_file = temp_input_file.path().to_str().unwrap().to_string();

        let mut input = Cursor::new(format!("1\nnew_env_value\ns\nc\n"));
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &input_file, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        debug!("Test output: {}", output_str);

        let env_var_name = format!("TEST_ITEM_1_{}", test_id);
        let env_var_value = std::env::var(&env_var_name).unwrap();
        debug!("Environment variable {}: {}", env_var_name, env_var_value);
        assert_eq!(env_var_value, "new_env_value");

        let expected_env_path =
            EnvOutputUri!("local", &input_file).expect("Failed to construct ENV output path");
        debug!("Expected .env path: {:?}", expected_env_path);
        assert_eq!(
            result.env_file,
            Some(expected_env_path.clone()),
            "The env_file in the result doesn't match the expected path"
        );

        assert!(
            std::path::Path::new(&expected_env_path).exists(),
            "The .env file was not created at {:?}",
            expected_env_path
        );

        let env_content = fs::read_to_string(&expected_env_path)?;
        debug!("Env file content: {}", env_content);
        assert!(
            env_content.contains(&format!("TEST_ITEM_1_{}=new_env_value", test_id)),
            "The .env file does not contain the expected content"
        );

        std::env::remove_var(&env_var_name);
        cleanup_test_environment(temp_dir, &home_var);
        Ok(())
    }
}
