#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rpcfg::commands::collect::execute;
use rpcfg::test_utils::create_test_config;
use rpcfg::{Config, ConfigItem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, BufReader, Write};
use tabwriter::TabWriter;
use tracing::{debug, info, trace, Level};
use tracing_subscriber::FmtSubscriber;

/// CLI tool for managing repository configurations
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the input JSON file
    #[arg(short = 'i', long = "input", global = true)]
    input_file: Option<String>,

    /// Use silent mode (non-interactive)
    #[arg(short = 's', long = "silent", global = true)]
    silent: bool,

    /// Set the tracing level (off, error, warn, info, debug, trace)
    #[arg(long, global = true, default_value = "error")]
    trace_level: Level,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new configuration file
    Init {
        /// Path to the output JSON file
        #[arg(short = 'o', long = "output")]
        output: String,
    },
    /// Collect repository configurations and generate output files
    Collect {
        #[arg(short = 'i', long = "input")]
        input_file: String,
    },
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
///
/// # Example
///
/// ```
/// use rpcfg::models::{Config, ConfigItem};
/// use std::fs::File;
/// use std::io::Write;
/// use tempfile::NamedTempFile;
///
/// // Create a temporary JSON file
/// let mut temp_file = NamedTempFile::new().unwrap();
/// writeln!(temp_file, r#"{{
///     "stored": "local",
///     "config_version": "1.0",
///     "project_name": "test_project",
///     "config_name": "test_config",
///     "is_test": true,
///     "items": []
/// }}"#).unwrap();
///
/// // Parse the config file
/// let config = rpcfg::parse_config_file(temp_file.path().to_str().unwrap());
/// assert!(config.is_ok());
/// ```
fn parse_config_file(file_path: &str) -> Result<Config> {
    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;

    Ok(config)
}

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
///
/// # Example
///
/// ```
/// use rpcfg::Cli;
/// use clap::Parser;
/// use std::ffi::OsString;
///
/// // Simulate command-line arguments
/// let args = vec![
///     OsString::from("rp"),
///     OsString::from("collect"),
///     OsString::from("-f"),
///     OsString::from("config.json"),
/// ];
///
/// // Parse CLI arguments
/// let cli = Cli::parse_from(args);
///
/// // Check if parsing was successful
/// assert_eq!(cli.input_file, Some("config.json".to_string()));
/// ```
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(cli.trace_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting application");

    // Execute the appropriate command
    match cli.command {
        Commands::Init { output } => {
            info!("Executing Init command");
            let result = rpcfg::commands::init::execute(&output)?;
            println!("{}", result.message);
        }
        Commands::Collect { input_file } => {
            info!("Executing Collect command");
            let mut config = parse_config_file(&input_file)?;
            rpcfg::commands::collect::execute(&mut config, &input_file)?;
        }
        _ => {
            // For other commands, check if input_file is provided
            let config = if let Some(file_path) = cli.input_file.as_ref() {
                debug!("Parsing config file: {}", file_path);
                parse_config_file(file_path)?
            } else {
                return Err(anyhow::anyhow!(
                    "No config file provided. Use -f or --input-file to specify a config file."
                ));
            };

            match cli.command {
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
                _ => unreachable!(),
            }
        }
    }

    info!("Application finished");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, ConfigItem};
    use rpcfg::commands::collect::collect_user_input;
    use rpcfg::safe_test;
    use std::fs;
    use std::io::Cursor;
    use uuid::Uuid;


    //
    // we haven't implemented this feature yet, so we can't test it
    // safe_test!(test_toggle_storage_type, {
    //     let test_id = Uuid::new_v4().to_string();
    //     let mut config = create_test_config(&test_id);

    //     let mut input = Cursor::new("1\nkeyvault\ns\nq\n");
    //     let mut output = Cursor::new(Vec::new());

    //     let result = collect_user_input(&mut config, &mut input, &mut output)?;

    //     assert!(matches!(result.status, rpcfg::Status::Ok));

    //     let output_str = String::from_utf8(output.into_inner())?;
    //     debug!("Output: {}", output_str);

    //     assert!(output_str.contains("stored=keyvault"));

    //     Ok(())
    // });

    safe_test!(test_invalid_input, {
        let test_id = Uuid::new_v4().to_string();
        let mut config = create_test_config(&test_id);

        let mut input = Cursor::new("invalid\n99\nq\n");
        let mut output = Cursor::new(Vec::new());

        let result = collect_user_input(&mut config, &mut input, &mut output)?;

        assert!(matches!(result.status, rpcfg::Status::Ok));

        let output_str = String::from_utf8(output.into_inner())?;
       
        assert!(output_str.contains("Invalid input. Please try again."));
        assert!(output_str.contains("Invalid item number. Please try again."));

        Ok(())
    });
}
