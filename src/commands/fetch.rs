use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use tracing::{debug, info};
use crate::models::{Config, CommandResult, Status};
use crate::json_output_uri;

/// Fetches and displays the current configuration from a JSON file.
///
/// This function reads the configuration from a JSON file specified in the Config object,
/// parses it into a HashMap, and writes the prettified JSON to the provided output stream.
///
/// # Arguments
///
/// * `config` - A reference to the Config object containing the path to the JSON file.
/// * `_input` - A mutable reference to a BufRead trait object. Not used in this function but included for consistency.
/// * `output` - A mutable reference to a Write trait object for writing the fetched configuration.
///
/// # Returns
///
/// Returns a Result containing a CommandResult on success, or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The JSON output path cannot be retrieved from the Config object.
/// * Opening or reading the JSON file fails.
/// * Parsing the JSON content fails.
/// * Writing to the output stream fails.
pub fn execute<R: BufRead, W: Write>(
    config: &Config,
    _input: &mut R,
    output: &mut W,
) -> Result<crate::CommandResult> {
    // Get the JSON output file path
    let json_path = json_output_uri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to get JSON output path"))?;

    // Trace the JSON file path to stderr
    debug!("JSON file path: {}", json_path);

    // Open and read the JSON file
    let file = File::open(&json_path)
        .with_context(|| format!("Failed to open JSON file: {}", json_path))?;
    let reader = BufReader::new(file);

    // Parse the JSON into a HashMap
    let config_map: HashMap<String, String> = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {}", json_path))?;

    // Write the fetched configuration to the output
    let json_output = serde_json::to_string_pretty(&config_map)?;
    writeln!(output, "{}", json_output)?;

    info!("Successfully fetched configuration from JSON file");

    Ok(CommandResult {
        status: Status::Ok,
        message: "Configuration fetched successfully.".to_string(),
        env_file: None,
        json_file: Some(json_path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::collect, create_test_input_file, parse_config_file, safe_test};
    use std::io::Cursor;
    use serde_json::Value;

    safe_test!(test_fetch_command, {
        // Create a test input file and get the config
        let (mut config, _temp_dir) = create_test_input_file!("fetch_command");

        // Step 1: Add a new setting and save it
        {
            let mut input = Cursor::new("n\nnew_key\nNew description\ndefault_value\nNEW_ENV_VAR\ny\ns\nq\n");
            let mut output = Cursor::new(Vec::new());

            let result = collect::collect_user_input(&mut config, &mut input, &mut output)?;
            assert!(matches!(result.status, crate::models::Status::Ok));

            // Verify the new setting is in the config object
            let new_item = config.app.iter().find(|item| item.key == "new_key");
            assert!(new_item.is_some(), "New setting should be present in the config object");
            let new_item = new_item.unwrap();
            assert_eq!(new_item.description, "New description");
            assert_eq!(new_item.default, "default_value");
            assert_eq!(new_item.temp_environment_variable_name, "NEW_ENV_VAR");
            assert!(new_item.required_as_env);
        }

        // Step 2: Fetch the output file
        {
            let mut input = Cursor::new(Vec::new());
            let mut output = Cursor::new(Vec::new());

            let result = execute(&config, &mut input, &mut output)?;
            assert!(matches!(result.status, crate::Status::Ok));

            // Unmarshal the JSON from the output buffer
            let output_str = String::from_utf8(output.into_inner())?;
            let json_data: Value = serde_json::from_str(&output_str)?;

            // Verify rpcfg settings
            assert_eq!(json_data["stored"], "local");
            assert_eq!(json_data["config_version"], "1.0");
            assert!(json_data["project_name"].as_str().unwrap().starts_with("project_"));
            assert!(json_data["config_name"].as_str().unwrap().starts_with("config_"));
            assert!(json_data["environment"].as_str().unwrap().starts_with("env_"));

            // Verify the new setting
            assert_eq!(json_data["new_key"], "default_value");
        }

        // Step 3: Verify the new setting is saved to the input file
        {
            let updated_config = parse_config_file(&config.input_file)?;
            let new_item = updated_config.app.iter().find(|item| item.key == "new_key");
            assert!(new_item.is_some(), "New setting should be present in the input file");
            let new_item = new_item.unwrap();
            assert_eq!(new_item.description, "New description");
            assert_eq!(new_item.default, "default_value");
            assert_eq!(new_item.temp_environment_variable_name, "NEW_ENV_VAR");
            assert!(new_item.required_as_env);
        }

        Ok(())
    });
}
