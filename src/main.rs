#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, stderr, Write};
use serde::{Serialize, Deserialize};
use tracing::{Level, info, debug, trace};
use tracing_subscriber::FmtSubscriber;
use rp::models::{Config, ConfigItem};
use tabwriter::TabWriter;


#[macro_use]
pub mod rp_macros;
pub use rp_macros::*;

#[macro_use]
pub mod common;
pub use common::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Ok,
    Error,
}

/// CLI tool for managing repository configurations
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the input JSON file
    #[arg(short = 'f', long, global = true)]
    input_file: Option<String>,

    /// Use interactive mode to collect values
    #[arg(short, long, global = true)]
    interactive: bool,

    /// Set the tracing level (off, error, warn, info, debug, trace)
    #[arg(long, global = true, default_value = "info")]
    trace_level: Level,
}

#[derive(Subcommand)]
enum Commands {
    /// Collect repository configurations and generate output files
    Collect,
    /// Delete generated output files
    Delete,
    /// Return the JSON config with the values
    Fetch,
    /// Show the configuration table
    Show,
}

/// Parses a JSON configuration file into a Config struct.
///
/// This function reads a JSON file from the given path and deserializes it into a Config struct.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the JSON file
///
/// # Returns
///
/// * `Result<Config>` - The parsed Config struct or an error
///
/// # Errors
///
/// This function will return an error if:
/// * The file cannot be opened
/// * The JSON in the file cannot be parsed into a Config struct
fn parse_config_file(file_path: &str) -> Result<Config> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;
    
    Ok(config)
}


mod commands;

/// The main entry point for the CLI application.
///
/// This function parses command-line arguments, sets up logging, loads the configuration,
/// and executes the appropriate command based on user input.
///
/// # Returns
///
/// * `Result<()>` - Ok if the program runs successfully, Err otherwise
///
/// # Errors
///
/// This function will return an error if:
/// * Command-line argument parsing fails
/// * Config file parsing fails
/// * Executing a command fails
fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(cli.trace_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting application");

    // Parse the config file if provided
    let mut config = if let Some(file_path) = cli.input_file.as_ref() {
        debug!("Parsing config file: {}", file_path);
        parse_config_file(file_path)?
    } else {
        return Err(anyhow::anyhow!("No config file provided. Use -f or --input-file to specify a config file."));
    };

    // Execute the appropriate command
    match cli.command {
        Commands::Collect => {
            info!("Executing Collect command");
            commands::collect::execute(&mut config, cli.interactive)?;
            info!("User input collected");
        }
        Commands::Delete => {
            info!("Executing Delete command");
            // TODO: Implement Delete command
        }
        Commands::Fetch => {
            info!("Executing Fetch command");
            // TODO: Implement Fetch command
        }
        Commands::Show => {
            info!("Executing Show command");
            // to do - pull the input values and display them in a table
           
        }
    }

    info!("Application finished");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, ConfigItem};
    use commands::collect::collect_user_input;
    use std::fs;
    use std::io::Cursor;
    use uuid::Uuid;

    fn setup_test_config(test_id: &str) -> Result<Config> {
        let config = Config {
            stored: String::from("local"),
            config_version: String::from("1.0"),
            project_name: String::from("test_project"),
            config_name: format!("test_config_{}", test_id), // Salted with test_id
            is_test: true,
            items: vec![
                ConfigItem {
                    key: format!("item1_{}", test_id),
                    description: "Test item 1".to_string(),
                    shellscript: "".to_string(),
                    default: "default1".to_string(),
                    temp_environment_variable_name: format!("TEST_ITEM_1_{}", test_id),
                    required_as_env: true,
                    value: String::new(),
                },
                ConfigItem {
                    key: format!("item2_{}", test_id),
                    description: "Test item 2".to_string(),
                    shellscript: "".to_string(),
                    default: "default2".to_string(),
                    temp_environment_variable_name: String::new(),
                    required_as_env: false,
                    value: String::new(),
                },
            ],
        };

        Ok(config)
    }

    #[test]
    fn test_non_interactive_mode() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        let mut input = Cursor::new("");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, false, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        for item in &config.items {
            debug!(
                "Item {}: value = {}, default = {}",
                item.key, item.value, item.default
            );
            assert_eq!(item.value, item.default);
        }

        Ok(())
    }

    #[test]
    fn test_toggle_storage_type() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        let mut input = Cursor::new("t\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        debug!("Output: {}", output_str);

        assert!(output_str.contains("Storage type: keyvault"));

        Ok(())
    }

    #[test]
    fn test_invalid_input() -> Result<()> {
        let test_id = Uuid::new_v4().to_string();
        let mut config = setup_test_config(&test_id)?;

        let mut input = Cursor::new("invalid\n3\nc\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, true, &mut input, &mut output)?;

        assert!(matches!(result.status, crate::rp_macros::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
        debug!("Output: {}", output_str);

        assert!(output_str.contains("Invalid input. Please try again."));
        assert!(output_str.contains("Invalid item number. Please try again."));

        Ok(())
    }
 
}