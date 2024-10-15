use anyhow::Context;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite; // Add this import at the top of the file
use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;

use tabwriter::TabWriter;
use tracing::debug;

use crate::models::{CommandResult, Config, ConfigItem};
use crate::{env_output_uri, json_output_uri, Success};

/// Executes the collect command, gathering configuration input from the user.
///
/// This function serves as the entry point for the collect command. It checks if the
/// configuration needs updating based on file timestamps (unless ignore_timestamps is true),
/// and if so, it calls `collect_user_input` to handle the actual collection of configuration data.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `input_file` - The path to the input file.
/// * `ignore_timestamps` - Whether to ignore timestamp checks and always collect.
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
/// * There's an I/O error when checking file timestamps or accessing the file system.
/// * The `collect_user_input` function encounters an error.
pub fn execute(
    config: &mut crate::Config,
    input_file: &str,
    ignore_timestamps: bool,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<crate::CommandResult> {
    let input_path = Path::new(input_file);

    // Get the output file path using the JsonOutputUri! macro
    let output_path = json_output_uri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to get JSON output path"))?;
    let output_path = Path::new(&output_path);

    debug!("Input file: {:?}", input_path);
    debug!("Output file: {:?}", output_path);
    debug!("Ignore timestamps: {}", ignore_timestamps);

    // Check if the output file exists and is newer than the input, unless ignore_timestamps is true
    if !ignore_timestamps && output_path.exists() {
        // Get the modification times
        let input_modified = input_path.metadata()?.modified()?;
        let output_modified = output_path.metadata()?.modified()?;

        debug!("Input modified: {:?}", input_modified);
        debug!("Output modified: {:?}", output_modified);

        // If the output is newer than or equal to the input, return silently
        if output_modified >= input_modified {
            debug!("Output file is up to date. Skipping collection.");
            return Ok(crate::CommandResult {
                status: crate::Status::Ok,
                message: "Configuration is up to date.".to_string(),
                env_file: env_output_uri!(config),
                json_file: Some(output_path.to_string_lossy().into_owned()),
            });
        }
    }

    let result = collect_user_input(config, input, output)?;

    Ok(result)
}
/// Collects user input to configure items in the provided Config object.
///
/// This function initializes config values with defaults if
/// they are empty and then handles the interactive configuration loop.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
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
///     let mut input = Cursor::new("6\nnew_value\ns\nq\n");
///     let mut output = Vec::new();
///     let result = collect_user_input(&mut config, &mut input, &mut output)?;
///     assert!(matches!(result.status, rpcfg::Status::Ok));
///     Ok(())
/// }
/// ```
pub fn collect_user_input<R: BufRead, W: Write>(
    config: &mut Config,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<CommandResult> {
    debug!("collect_user_input: config: {:?}", config);

    // Initialize empty values with defaults
    initialize_config_values(config);

    interactive_config_loop(config, input, output)?;

    // Set environment variables for required items
    set_environment_variables(config);

    let mut result = Success!("Configuration collected successfully.");
    result.env_file = env_output_uri!(config);
    result.json_file = json_output_uri!(config);
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
/// update, add new settings, and save configuration items. It continues to prompt the user
/// for actions until they choose to quit or save.
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
    let mut new_setting_added = false;

    loop {
        config.validate_rpcfg_config()?;
        show_current_config(config, output)?;

        write!(
            output,
            "\nEnter item number to update, 'S' to save, 'N' to add a new setting, or 'Q' to quit: "
        )?;
        output.flush()?;

        let user_input = read_user_input(input)?;

        match user_input.as_str() {
            "s" | "S" => {
                save_configuration(config, new_setting_added)?;
                writeln!(output, "Configuration saved.")?;
                break;
            }
            "q" | "Q" => break,
            "n" | "N" => {
                add_new_setting(config, input, output)?;
                new_setting_added = true;
            }
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
    Ok(user_input.trim().to_string()) // Remove .to_lowercase()
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
/// * `index` - The index of the item to be updated (combined across rpcfg and app items).
/// * `new_value` - The new value to set for the item.
///
/// # Returns
///
/// Returns a Result<()>. The function succeeds if the item is successfully updated.
///
/// # Errors
///
/// This function will return an error if:
/// * The specified index is out of bounds for the combined rpcfg and app items.
///
/// # Examples
///
/// ```
/// use rpcfg::{Config, ConfigItem};
/// use anyhow::Result;
/// use std::io::Cursor;
/// use rpcfg::commands::collect::update_item;
///
/// fn main() -> Result<()> {
///     let mut config = Config::default();
///     config.app.push(ConfigItem {
///                 key: "app_item1".to_string(),
///                 description: "App Test item 1".to_string(),
///                 shellscript: "".to_string(),
///                 default: "default1".to_string(),
///                 temp_environment_variable_name: "APP_TEST_ITEM_1".to_string(),
///                 required_as_env: true,
///                 value: "old_value".to_string(),
///             });
///     
///     // Update the first app item (index 5, assuming 5 rpcfg items)
///     update_item(&mut config, 5, &mut Cursor::new("new_value\n"), &mut Vec::new())?;
///     assert_eq!(config.app[0].value, "new_value");
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
pub fn save_configuration(config: &Config, save_input: bool) -> anyhow::Result<()> {
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

    // Save input file if save_input is true and input_file is specified
    if save_input {
        let input_file_path = &config.input_file;
        if !input_file_path.is_empty() {
            debug!("Updating input file: {}", input_file_path);
            let input_content = serde_json::to_string_pretty(&config)?;
            fs::write(input_file_path, input_content)
                .with_context(|| format!("Failed to update input file: {}", input_file_path))?;
            debug!("Input file updated successfully");
        } else {
            debug!("No input file path specified, skipping input file update");
        }
    } else {
        debug!("Skipping input file update (save_input is false)");
    }

    Ok(())
}

/// Adds a new setting to the configuration interactively.
///
/// This function prompts the user to enter details for a new configuration item,
/// including the key, description, default value, environment variable name,
/// and whether it's required as an environment variable.
///
/// # Arguments
///
/// * `config` - A mutable reference to the Config object to be updated.
/// * `input` - A mutable reference to a BufRead trait object for reading user input.
/// * `output` - A mutable reference to a Write trait object for writing prompts and messages.
///
/// # Returns
///
/// Returns a Result, which is Ok if the new setting is successfully added,
/// or an error if any I/O operations fail.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an I/O error when reading input or writing output.
/// * Any of the read_user_input calls fail.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use rpcfg::{Config, commands::collect::add_new_setting};
///
/// let mut config = Config::default();
/// let mut input = Cursor::new("new_key\nNew description\ndefault_value\nNEW_ENV_VAR\ny\n");
/// let mut output = Vec::new();
///
/// add_new_setting(&mut config, &mut input, &mut output).unwrap();
///
/// assert_eq!(config.app.last().unwrap().key, "new_key");
/// assert_eq!(config.app.last().unwrap().description, "New description");
/// ```
pub fn add_new_setting<R: BufRead, W: Write>(
    config: &mut Config,
    input: &mut R,
    output: &mut W,
) -> anyhow::Result<()> {
    writeln!(output, "Adding a new setting:")?;

    write!(output, "Enter key: ")?;
    output.flush()?;
    let key = read_user_input(input)?;

    write!(output, "Enter description: ")?;
    output.flush()?;
    let description = read_user_input(input)?;

    write!(output, "Enter default value: ")?;
    output.flush()?;
    let default = read_user_input(input)?;

    write!(output, "Enter environment variable name (or leave empty): ")?;
    output.flush()?;
    let temp_environment_variable_name = read_user_input(input)?;

    write!(
        output,
        "Is this required as an environment variable? (y/n): "
    )?;
    output.flush()?;
    let required_as_env = read_user_input(input)?.to_lowercase() == "y";

    let new_item = ConfigItem {
        key,
        description,
        shellscript: String::new(),
        default: default.clone(),
        temp_environment_variable_name,
        required_as_env,
        value: default,
    };

    config.app.push(new_item);
    writeln!(output, "New setting added successfully.")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init;
    use crate::{create_test_input_file, parse_config_file};
    use crate::{models::ConfigItem, safe_test, test_utils::create_test_config};
    use std::fs;
    use std::io::Cursor;
    use tempfile::TempDir;
    use uuid::Uuid;

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

        let mut input = Cursor::new("9\nq\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, &mut input, &mut output)?;

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

        // Provide enough input to complete the process:
        // - "invalid" (invalid input)
        // - "99" (invalid item number)
        // - "6" (select first item)
        // - "newvalue" (new value for the item)
        // - "s" (save)
        // - "q" (quit)
        let mut input = Cursor::new("invalid\n99\n6\nnewvalue\ns\nq\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, &mut input, &mut output)?;

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
        let result = collect_user_input(&mut config, &mut input, &mut output)?;
        assert!(matches!(result.status, crate::models::Status::Ok));

        // Check JSON file content
        let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
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
        let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");
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

    safe_test!(test_add_new_setting, {
        // Create a test input file and get the config
        let (mut config, _temp_dir) = create_test_input_file!("add_new_setting");

        // Simulate user input to add a new setting
        let mut input = Cursor::new("n\nnew_key\nNew description\ndefault_value\nNEW_ENV_VAR\ny\ns\nq\n");
        let mut output = Cursor::new(Vec::new());

        // Run collect_user_input
        let result = collect_user_input(&mut config, &mut input, &mut output)?;
        assert!(matches!(result.status, crate::models::Status::Ok));

        // Verify the new setting is in the config object
        let new_item = config.app.iter().find(|item| item.key == "new_key");
        assert!(new_item.is_some(), "New setting should be present in the config object");
        let new_item = new_item.unwrap();
        assert_eq!(new_item.description, "New description");
        assert_eq!(new_item.default, "default_value");
        assert_eq!(new_item.temp_environment_variable_name, "NEW_ENV_VAR");
        assert!(new_item.required_as_env);

        // Verify the new setting is saved to the input file
        let updated_config = parse_config_file(&config.input_file)?;
        let new_item = updated_config.app.iter().find(|item| item.key == "new_key");
        assert!(new_item.is_some(), "New setting should be present in the input file");
        let new_item = new_item.unwrap();
        assert_eq!(new_item.description, "New description");
        assert_eq!(new_item.default, "default_value");
        assert_eq!(new_item.temp_environment_variable_name, "NEW_ENV_VAR");
        assert!(new_item.required_as_env);

        Ok(())
    });

    safe_test!(test_storage_type_update, {
        let test_id = Uuid::new_v4().to_string();

        // Test invalid storage type
        {
            let mut config = create_test_config(&test_id);
            let mut input = Cursor::new("1\ninvalid_storage\ns\nq\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect_user_input(&mut config, &mut input, &mut output)?;
            assert!(matches!(result.status, crate::models::Status::Ok));

            let stored_item = config
                .rpcfg
                .iter()
                .find(|item| item.key == "stored")
                .unwrap();
            assert_eq!(
                stored_item.value, "local",
                "Storage type should be reset to 'local' after invalid input"
            );
        }

        // Test empty storage type
        {
            let mut config = create_test_config(&test_id);
            let mut input = Cursor::new("1\n\ns\nq\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect_user_input(&mut config, &mut input, &mut output)?;
            assert!(matches!(result.status, crate::models::Status::Ok));

            let stored_item = config
                .rpcfg
                .iter()
                .find(|item| item.key == "stored")
                .unwrap();
            assert_eq!(
                stored_item.value, "local",
                "Storage type should be set to 'local' when empty input is provided"
            );
        }

        // Test setting to "local"
        {
            let mut config = create_test_config(&test_id);
            let mut input = Cursor::new("1\nlocal\ns\nq\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect_user_input(&mut config, &mut input, &mut output)?;
            assert!(matches!(result.status, crate::models::Status::Ok));

            let stored_item = config
                .rpcfg
                .iter()
                .find(|item| item.key == "stored")
                .unwrap();
            assert_eq!(
                stored_item.value, "local",
                "Storage type should be set to 'local'"
            );
        }

        // Test setting to "keyvault"
        // todo: add keyvault support
        // {
        //     let mut config = create_test_config(&test_id);
        //     let mut input = Cursor::new("1\nkeyvault\ns\nq\n");
        //     let mut output = Cursor::new(Vec::new());

        //     let result = collect_user_input(&mut config, &mut input, &mut output)?;
        //     assert!(matches!(result.status, crate::models::Status::Ok));

        //     let stored_item = config.rpcfg.iter().find(|item| item.key == "stored").unwrap();
        //     assert_eq!(stored_item.value, "keyvault", "Storage type should be set to 'keyvault'");
        // }

        Ok(())
    });

    safe_test!(test_ignore_timestamps_flag, {
        let test_id = Uuid::new_v4().to_string();
        let temp_dir = tempfile::TempDir::new()?;
        let input_path = temp_dir.path().join("input.json");
        let mut config = create_test_config(&test_id);

        // Create initial input file
        let mut input_file = fs::File::create(&input_path)?;
        serde_json::to_writer_pretty(&mut input_file, &config)?;
        input_file.flush()?;

        // First collection
        let mut input = Cursor::new("6\nnew_value\ns\nq\n");
        let mut output = Cursor::new(Vec::new());
        let result = execute(
            &mut config,
            input_path.to_str().unwrap(),
            false,
            &mut input,
            &mut output,
        )?;
        let first_output_size = output.get_ref().len();
        debug!("First execution result: {:?}", result);
        debug!(
            "First output content: {}",
            String::from_utf8_lossy(output.get_ref())
        );
        debug!("First output size: {}", first_output_size);
        assert!(
            first_output_size > 0,
            "First output buffer should not be empty"
        );

        // Second collection without ignore_timestamps
        let mut input = Cursor::new("");
        let mut output = Cursor::new(Vec::new());
        execute(
            &mut config,
            input_path.to_str().unwrap(),
            false,
            &mut input,
            &mut output,
        )?;
        let second_output_size = output.get_ref().len();
        assert_eq!(
            second_output_size, 0,
            "Second output buffer should be empty"
        );

        // Third collection with ignore_timestamps
        let mut input = Cursor::new("s\nq\n");
        let mut output = Cursor::new(Vec::new());
        execute(
            &mut config,
            input_path.to_str().unwrap(),
            true,
            &mut input,
            &mut output,
        )?;
        let third_output_size = output.get_ref().len();
        assert!(
            third_output_size > 0,
            "Third output buffer should not be empty"
        );

        Ok(())
    });
}
