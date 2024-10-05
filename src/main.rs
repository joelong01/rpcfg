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
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the JSON file
///
/// # Returns
///
/// * `Result<Config>` - The parsed Config struct or an error
fn parse_config_file(file_path: &str) -> Result<Config> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;
    Ok(config)
}

/// Displays the configuration table to the specified output.
///
/// # Arguments
///
/// * `config` - A reference to the Config struct containing the configuration data
/// * `out` - A mutable reference to a Write trait object for output
///
/// # Returns
///
/// * `Result<()>` - Ok if the table was successfully displayed, Err otherwise
fn show_config_table<W: Write>(config: &Config, out: &mut W) -> Result<()> {
    let mut tw = TabWriter::new(vec![]);

    writeln!(tw, "Index\tDescription\tValue")?;
    writeln!(tw, "-----\t-----------\t-------------")?;
    
    // Collect items into a vector and sort by index
    let mut items: Vec<_> = config.items.iter().collect();
    items.sort_by(|a, b| a.0.cmp(b.0));

    for (index, (_, item)) in items.iter().enumerate() {
        let display_value = if item.value.is_empty() { &item.default } else { &item.value };
        writeln!(tw, "{}\t{}\t{}", index + 1, item.description, display_value)?;
    }
    tw.flush()?;

    out.write_all(&tw.into_inner()?)?;
    write!(out, "\nValues stored {}.", config.stored)?;
    Ok(())
}

mod commands;

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

    match cli.command {
        Commands::Collect => {
            info!("Executing Collect command");
            commands::collect::execute(&mut config, cli.interactive, cli.input_file.as_ref().unwrap())?;
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
            show_config_table(&config, &mut stderr())?;
        }
    }

    info!("Application finished");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn test_show_config_table() -> Result<()> {
        // Create a sample config
        let config = Config {
            stored: "locally".to_string(),
            config_version: "1.0".to_string(),
            items: [
                ("azureLocation".to_string(), ConfigItem {
                    description: "the location for your Azure Datacenter".to_string(),
                    shellscript: "".to_string(),
                    default: "uswest3".to_string(),
                    temp_environment_variable_name: "AZURE_LOCATION".to_string(),
                    required_as_env: true,
                    value: "".to_string(), 
                }),
                ("username".to_string(), ConfigItem {
                    description: "the username for the test".to_string(),
                    shellscript: "".to_string(),
                    default: "test_user".to_string(),
                    temp_environment_variable_name: "".to_string(),
                    required_as_env: false,
                    value: "".to_string(), 
                }),
            ].into_iter().collect(),
        };

        // Create a buffer to capture the output
        let mut output = Vec::new();

        // Call the function
        show_config_table(&config, &mut output)?;

        // Convert the output to a string
        let output_str = from_utf8(&output)?;

        // Define the expected output
        let expected_output = "\
Index  Description                             Value
-----  -----------                             -------------
1      the location for your Azure Datacenter  uswest3
2      the username for the test               test_user

Values stored locally.";

        // Compare the actual output with the expected output
        assert_eq!(output_str, expected_output);

        Ok(())
    }
}