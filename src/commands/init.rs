use crate::{Config, CommandResult, Success};
use anyhow::Result;
use std::fs;

pub fn execute(output_path: &str) -> Result<CommandResult> {
    let config = Config::default();
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(output_path, json)?;
    Ok(Success!("Configuration file initialized at: {}", output_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_init_command() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let output_path = temp_file.path().to_str().unwrap();

        let result = execute(output_path)?;
        assert!(result.message.contains("Configuration file initialized"));

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