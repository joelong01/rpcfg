use crate::{Config, CommandResult, Status};
use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, Write};
use tracing::info;

/// Initializes a new configuration file with default settings.
///
/// This function creates a new Config object with default values,
/// serializes it to JSON, and writes it to the specified output path.
/// It also writes a confirmation message to the provided output stream.
///
/// # Arguments
///
/// * `output_path` - A string slice that holds the path where the configuration file will be written.
/// * `_input` - A mutable reference to a BufRead trait object. Not used in this function but included for consistency.
/// * `output` - A mutable reference to a Write trait object for writing the confirmation message.
///
/// # Returns
///
/// Returns a Result containing a CommandResult on success, or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * Writing the configuration file fails.
/// * Writing to the output stream fails.
pub fn execute<R: BufRead, W: Write>(
    output_path: &str,
    _input: &mut R,
    output: &mut W,
) -> Result<CommandResult> {
    let config = Config::default();
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(output_path, &json)
        .with_context(|| format!("Failed to write configuration file: {}", output_path))?;

    writeln!(output, "Configuration file initialized at: {}", output_path)?;

    info!("Successfully initialized configuration file");

    Ok(CommandResult {
        status: Status::Ok,
        message: format!("Configuration file initialized at: {}", output_path),
        env_file: None,
        json_file: Some(output_path.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[test]
    fn test_init_command() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let output_path = temp_file.path().to_str().unwrap();

        let mut input = Cursor::new(Vec::new());
        let mut output = Cursor::new(Vec::new());

        let result = execute(output_path, &mut input, &mut output)?;
       // assert_eq!(result.status, Status::Ok);
        assert!(result.message.contains("Configuration file initialized"));

        let output_str = String::from_utf8(output.into_inner())?;
        assert!(output_str.contains("Configuration file initialized"));

        let content = fs::read_to_string(output_path)?;
        let config: Config = serde_json::from_str(&content)?;

        // Verify the content of the config
        assert_eq!(config.rpcfg.len(), 5); // Assuming 5 default rpcfg items
        assert_eq!(config.app.len(), 0); // Assuming no default app items
        assert_eq!(config.rpcfg[0].key, "stored");
        assert_eq!(config.rpcfg[0].default, "local");

        Ok(())
    }
}
