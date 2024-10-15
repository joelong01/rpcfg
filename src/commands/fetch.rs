use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use tracing::{debug, info};
use crate::models::{Config, CommandResult, Status};
use crate::JsonOutputUri;

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
    let json_path = JsonOutputUri!(config)
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
