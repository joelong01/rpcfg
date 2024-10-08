use anyhow::Context;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite; // Add this import at the top of the file
use std::fs;
use std::io::{self, BufRead, Write};
use tabwriter::TabWriter;
use tracing::debug;

use crate::models::{CommandResult, Config};
use crate::{EnvOutputUri, JsonOutputUri, Success};

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
/// use rpcfg::{Config, CommandResult};
/// use anyhow::Result;
///
/// fn execute(config: &mut Config, interactive: bool) -> anyhow::Result<CommandResult> {
///     // Implementation details...
///     Ok(CommandResult {
///         status: rpcfg::Status::Ok,
///         message: "Configuration collected successfully.".to_string(),
///         env_file: Some("path/to/env/file".to_string()),
///         json_file: Some("path/to/json/file".to_string()),
///     })
/// }
///
/// fn main() -> Result<()> {
///     let mut config = Config::default();
///     let result = execute(&mut config, true)?;
///     assert!(matches!(result.status, rpcfg::Status::Ok));
///     Ok(())
/// }
/// ```
pub fn execute(
    config: &mut crate::Config,
    interactive: bool,
) -> anyhow::Result<crate::CommandResult> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stderr();
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
/// use std::io::Cursor;
/// use rpcfg::{Config, ConfigItem, commands::collect::collect_user_input};
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let mut config = Config::default();
///     config.app.push(ConfigItem {
///         key: "item1".to_string(),
///         description: "Test item 1".to_string(),
///         shellscript: "".to_string(),
///         default: "".to_string(),
///         temp_environment_variable_name: "TEST_ITEM_1".to_string(),
///         required_as_env: true,
///         value: "".to_string(),
///     });
///     let mut input = Cursor::new("1\nnew_value\ns\nq\n");
///     let mut output = Vec::new();
///     let result = collect_user_input(&mut config, true, &mut input, &mut output)?;
///     assert!(matches!(result.status, rpcfg::Status::Ok));
///     Ok(())
/// }
/// ```
pub fn collect_user_input<R: BufRead, W: Write>(
    config: &mut Config,
    interactive: bool,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<CommandResult> {
    debug!(
        "collect_user_input: config: {:?} interactive: {}",
        config, interactive
    );

    // Initialize empty values with defaults
    initialize_config_values(config);

    if interactive {
        interactive_config_loop(config, input, output)?;
    }

    // Save configuration (for both interactive and non-interactive modes)
    save_configuration(config)?;

    // Set environment variables for required items
    set_environment_variables(config);

    let mut result = Success!("Configuration collected successfully.");
    result.env_file = EnvOutputUri!(config);
    result.json_file = JsonOutputUri!(config);
    Ok(result)
}

/// Initialize config values with defaults if they are empty
///
/// This function iterates through all ConfigItems in both rpcfg and app
/// and sets their value to the default if the current value is empty.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be initialized.
fn initialize_config_values(config: &mut Config) {
    for item in config.rpcfg.iter_mut().chain(config.app.iter_mut()) {
        if item.value.is_empty() {
            item.value = item.default.clone();
        }
    }
}

/// Handles the interactive configuration loop
///
/// This function manages the interactive session where the user can view,
/// update, and save configuration items. It continues to prompt the user
/// for actions until they choose to quit.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `input` - A mutable reference to a BufRead trait object for reading user input.
/// * `output` - A mutable reference to a Write trait object for writing prompts and messages.
///
/// # Returns
///
/// Returns a Result, which is Ok if the interactive session completes successfully,
/// or an error if any I/O operations fail.
fn interactive_config_loop<R: BufRead, W: Write>(
    config: &mut Config,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<()> {
    loop {
        show_current_config(config, output)?;

        write!(
            output,
            "\nEnter item number to update, 'S' to save, or 'Q' to quit: "
        )?;
        output.flush()?;

        let user_input = read_user_input(input)?;

        match user_input.as_str() {
            "s" => {
                save_configuration(config)?;
                writeln!(output, "Configuration saved.")?;
            }
            "q" => break,
            _ => handle_item_update(config, &user_input, input, output)?,
        }
    }
    Ok(())
}

/// Read and trim user input
fn read_user_input<R: BufRead>(input: &mut R) -> anyhow::Result<String> {
    let mut user_input = String::new();
    input
        .read_line(&mut user_input)
        .context("Failed to read user input")?;
    Ok(user_input.trim().to_lowercase())
}

/// Handle updating a specific item in the configuration
fn handle_item_update<R: BufRead, W: Write>(
    config: &mut Config,
    user_input: &str,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<()> {
    if let Ok(index) = user_input.parse::<usize>() {
        if index > 0 && index <= config.rpcfg.len() + config.app.len() {
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
    for item in config.rpcfg.iter().chain(config.app.iter()) {
        if item.required_as_env && !item.temp_environment_variable_name.is_empty() {
            std::env::set_var(&item.temp_environment_variable_name, &item.value);
        }
    }
}
/// Displays the current configuration
///
/// This function prints out all the configuration items in a tabular format,
/// showing their index, key, description, and current value.
///
/// # Arguments
///
/// * `config` - A reference to the Config object to be displayed.
/// * `output` - A mutable reference to a Write trait object for writing the configuration.
///
/// # Returns
///
/// Returns a Result, which is Ok if the configuration is successfully written to the output,
/// or an error if any I/O operations fail.
pub fn show_current_config<W: Write>(config: &Config, output: &mut W) -> anyhow::Result<()> {
    if config.is_test {
        writeln!(output, "(Test mode)")?;
    }
    writeln!(output)?; // Add an extra newline for spacing

    let mut tw = TabWriter::new(vec![]);

    writeln!(tw, "Index\tDescription\tValue")?;
    writeln!(tw, "-----\t-----------\t-----")?;

    for (index, item) in config.rpcfg.iter().chain(config.app.iter()).enumerate() {
        let display_value = if item.value.is_empty() {
            &item.default
        } else {
            &item.value
        };
        writeln!(tw, "{}\t{}\t{}", index + 1, item.description, display_value)?;
    }
    tw.flush()?;

    output.write_all(&tw.into_inner()?)?;
    writeln!(output)?; // Add an extra newline at the end

    Ok(())
}
/// Updates a specific item in the configuration based on user input.
///
/// This function prompts the user to enter a new value for a specific configuration item,
/// reads the input, and updates the item's value in the config.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `index` - The index of the item to be updated in the config's items vector.
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
/// * The specified index is out of bounds for the config's items vector.
/// * There's an I/O error when reading input or writing output.
///
/// # Examples
///
/// ```
/// use rpcfg::{Config, ConfigItem};
/// use std::io::{Cursor, BufRead, Write};
/// use anyhow::Result;
///
/// fn update_item<R: BufRead, W: Write>(
///     config: &mut Config,
///     index: usize,
///     input: &mut R,
///     output: &mut W,
/// ) -> Result<()> {
///     let item = config.items.get_mut(index).ok_or(anyhow::anyhow!("Item not found"))?;
///     writeln!(output, "Enter new value for {} (current: {}): ", item.description, item.value)?;
///     let mut new_value = String::new();
///     input.read_line(&mut new_value)?;
///     item.value = new_value.trim().to_string();
///     Ok(())
/// }
///
/// fn main() -> Result<()> {
///     let mut config = Config {
///         stored: "local".to_string(),
///         config_version: "1.0".to_string(),
///         project_name: "test_project".to_string(),
///         config_name: "test_config".to_string(),
///         environment: "test".to_string(),
///         is_test: true,
///         items: vec![
///             ConfigItem {
///                 key: "item1".to_string(),
///                 description: "Test item 1".to_string(),
///                 shellscript: "".to_string(),
///                 default: "default1".to_string(),
///                 temp_environment_variable_name: "TEST_ITEM_1".to_string(),
///                 required_as_env: true,
///                 value: "old_value".to_string(),
///             },
///         ],
///     };
///     let mut input = Cursor::new("new_value\n");
///     let mut output = Vec::new();
///     update_item(&mut config, 0, &mut input, &mut output)?;
///     assert_eq!(config.items[0].value, "new_value");
///     Ok(())
/// }
/// ```
pub fn update_item<R: BufRead, W: Write>(
    config: &mut Config,
    index: usize,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<()> {
    let item = config
        .rpcfg
        .iter_mut()
        .chain(config.app.iter_mut())
        .nth(index)
        .ok_or(anyhow::anyhow!("Item not found"))?;
    write!(
        output,
        "Enter new value for {} (current: {}): ",
        item.description, item.value
    )?;
    output.flush()?;
    let mut new_value = String::new();
    input.read_line(&mut new_value)?;
    item.value = new_value.trim().to_string();
    debug!("Updated item: {:?}", item);
    Ok(())
}
/// Saves the current configuration to JSON and ENV files.
///
/// This function writes the current state of the configuration to two files:
/// 1. A JSON file containing all configuration items and their values.
/// 2. An ENV file containing environment variable declarations for all items.
///
/// The function uses the `base_output_dir` function to determine the appropriate
/// base directory for the output files.
///
/// # Arguments
///
/// * `config` - A reference to the Config object containing the configuration to be saved.
///
/// # Returns
///
/// Returns a Result<()>. The function succeeds if both files are written successfully.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when creating the output directory or writing to either the JSON or ENV file.
/// * The `base_output_dir` function fails to generate a valid directory path.
/// * The configuration data cannot be serialized to JSON.
///
/// # Examples
///
/// ```
/// use rpcfg::{Config, ConfigItem, commands::collect::save_configuration};
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let mut config = Config::default();
///     config.project_name = "test_project".to_string();
///     config.config_name = "test_config".to_string();
///     config.environment = "test".to_string();
///     config.is_test = true;
///     config.rpcfg.push(ConfigItem {
///         key: "item1".to_string(),
///         description: "Test item 1".to_string(),
///         shellscript: "".to_string(),
///         default: "default1".to_string(),
///         temp_environment_variable_name: "TEST_ITEM_1".to_string(),
///         required_as_env: true,
///         value: "value1".to_string(),
///     });
///
///     save_configuration(&config)?;
///     Ok(())
/// }
/// ```
///
/// # Note
///
/// This function will create the output directory if it doesn't exist and
/// will overwrite existing files if they already exist at the target paths.
pub fn save_configuration(config: &Config) -> anyhow::Result<()> {
    let base_dir = crate::rp_macros::base_output_dir(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to get base output directory"))?;

    debug!("Base output directory: {:?}", base_dir);

    // Create the base directory if it doesn't exist
    fs::create_dir_all(&base_dir)?;

    let json_path = base_dir.with_extension("json");
    let env_path = base_dir.with_extension("env");

    debug!("JSON output path: {:?}", json_path);
    debug!("ENV output path: {:?}", env_path);

    // Create a flat HashMap for JSON, excluding the is_test property
    let mut flat_json: HashMap<String, String> = HashMap::new();
    for item in config.rpcfg.iter().chain(config.app.iter()) {
        if item.key != "is_test" {
            debug!("Adding item to JSON: {} = {}", item.key, item.value);
            flat_json.insert(item.key.clone(), item.value.clone());
        }
    }

    // Save JSON file
    let json_content = serde_json::to_string_pretty(&flat_json)?;
    fs::write(&json_path, json_content)?;

    // Save ENV file
    let mut env_content = String::new();
    for item in config.rpcfg.iter().chain(config.app.iter()) {
        if item.required_as_env {
            debug!("Saving to ENV file: {} = {}", item.key, item.value);
            writeln!(env_content, "{}={}", item.key.to_uppercase(), item.value)?;
            // Note: We're only uppercasing the key, not the value
        }
    }
    fs::write(&env_path, &env_content)?;

    debug!("Configuration saved successfully");
    debug!("ENV content: {}", env_content);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use io::Cursor;
    use uuid::Uuid;
    use super::*;
    use crate::{create_test_config, safe_test, ConfigItem};

    safe_test!(test_non_interactive_mode, {
        // Test non-interactive mode of collect_user_input
        //
        // This test verifies that in non-interactive mode:
        // 1. The function completes successfully
        // 2. All config items are set to their default values
        //
        // Failure conditions:
        // - If the function returns an error
        // - If any config item's value is not equal to its default
        let test_id = Uuid::new_v4().to_string();
        let mut config = create_test_config(&test_id);

        // Store the initial values
        let initial_values: Vec<String> = config
            .rpcfg
            .iter()
            .chain(config.app.iter())
            .map(|item| item.value.clone())
            .collect();

        let mut input = Cursor::new("");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, false, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::models::Status::Ok));

        for (index, item) in config.rpcfg.iter().chain(config.app.iter()).enumerate() {
            debug!(
                "Item {}: initial value = {}, current value = {}, default = {}",
                item.key, initial_values[index], item.value, item.default
            );

            if initial_values[index].is_empty() {
                assert_eq!(
                    item.value, item.default,
                    "Empty item should be set to default"
                );
            } else {
                assert_eq!(
                    item.value, initial_values[index],
                    "Non-empty item should remain unchanged"
                );
            }
        }

        Ok(())
    });

    safe_test!(test_invalid_input, {
        // Test invalid input handling in collect_user_input
        //
        // This test verifies that:
        // 1. The function handles invalid input correctly
        // 2. Appropriate error messages are displayed
        // 3. The function completes successfully despite invalid inputs
        //
        // Failure conditions:
        // - If the function returns an error
        // - If the output doesn't contain expected error messages
        let test_id = Uuid::new_v4().to_string();
        let mut config = create_test_config(&test_id);

        let mut input = Cursor::new("invalid\n3\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::models::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        assert!(output_str.contains("Invalid input. Please try again."));
        assert!(output_str.contains("Invalid item number. Please try again."));

        Ok(())
    });

    safe_test!(test_configuration_settings, {
        // Define the settings map
        let mut settings = HashMap::new();
        settings.insert("stored".to_string(), ("local".to_string(), false, false)); // (value, required_as_env, is_app_setting)
        settings.insert(
            "config_version".to_string(),
            ("1.0".to_string(), false, false),
        );
        settings.insert(
            "project_name".to_string(),
            ("test_project".to_string(), false, false),
        );
        settings.insert(
            "config_name".to_string(),
            ("test_config".to_string(), false, false),
        );
        settings.insert(
            "environment".to_string(),
            ("test".to_string(), false, false),
        );
        settings.insert(
            "app_setting1".to_string(),
            ("value1".to_string(), true, true),
        );
        settings.insert(
            "app_setting2".to_string(),
            ("value2".to_string(), false, true),
        );

        // Create Config and ConfigItems
        let test_id = Uuid::new_v4().to_string();
        let mut config = Config::default();

        // Update rpcfg items
        for item in config.rpcfg.iter_mut() {
            if let Some((value, required_as_env, _)) = settings.get(&item.key) {
                item.value = value.clone();
                item.required_as_env = *required_as_env;
            }
        }

        // Add app items
        for (key, (value, required_as_env, is_app_setting)) in settings.iter() {
            if *is_app_setting {
                config.app.push(ConfigItem {
                    key: format!("{}_{}", key, test_id),
                    description: format!("Description for {}", key),
                    shellscript: "".to_string(),
                    default: "default".to_string(),
                    temp_environment_variable_name: format!("{}_{}", key.to_uppercase(), test_id),
                    required_as_env: *required_as_env,
                    value: value.clone(),
                });
            }
        }

        config.is_test = true;

        // Simulate user input (no changes, just save and quit)
        let mut input = Cursor::new("s\nq\n");
        let mut output = Cursor::new(Vec::new());

        // Call collect_user_input
        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;
        assert!(matches!(result.status, crate::models::Status::Ok));

        // Check JSON file content
        let json_path = JsonOutputUri!(&config).expect("Failed to construct JSON output path");
        let json_content = fs::read_to_string(&json_path)?;
        let json_map: HashMap<String, String> = serde_json::from_str(&json_content)?;

        for (key, (value, _, is_app_setting)) in &settings {
            let config_key = if *is_app_setting {
                format!("{}_{}", key, test_id)
            } else {
                key.clone()
            };
            assert_eq!(
                &json_map[&config_key], value,
                "JSON value mismatch for key: {}",
                key
            );
        }

        // Check ENV file content
        let env_path = EnvOutputUri!(&config).expect("Failed to construct ENV output path");
        let env_content = fs::read_to_string(&env_path)?;

        for (key, (value, required_as_env, is_app_setting)) in &settings {
            let config_key = if *is_app_setting {
                format!("{}_{}", key, test_id)
            } else {
                key.clone()
            };
            let env_var = format!("{}={}", config_key.to_uppercase(), value);
            if *required_as_env {
                assert!(
                    env_content.contains(&env_var),
                    "Required env var not found: {}",
                    env_var
                );
            } else {
                assert!(
                    !env_content.contains(&env_var),
                    "Unexpected env var found: {}",
                    env_var
                );
            }
        }

        // Clean up
        std::fs::remove_file(json_path)?;
        std::fs::remove_file(env_path)?;

        Ok(())
    });
}