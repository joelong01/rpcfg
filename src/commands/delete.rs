use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, Write};
use tracing::{debug, info};
use crate::{Config, CommandResult, Status, json_output_uri, env_output_uri};

pub fn execute<R: BufRead, W: Write>(
    config: &Config,
    no_prompt: bool,
    input: &mut R,
    output: &mut W,
) -> Result<CommandResult> {
    let json_path = json_output_uri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to get JSON output path"))?;
    let env_path = env_output_uri!(config)
        .ok_or_else(|| anyhow::anyhow!("Failed to get ENV output path"))?;

    debug!("JSON file path: {}", json_path);
    debug!("ENV file path: {}", env_path);

    if !no_prompt {
        writeln!(output, "Are you sure you want to delete {}? (y/N)", json_path)?;
        let mut response = String::new();
        input.read_line(&mut response)?;
        
        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            return Ok(CommandResult {
                status: Status::Ok,
                message: "Deletion cancelled.".to_string(),
                env_file: None,
                json_file: None,
            });
        }
    }

    let mut deleted_files = Vec::new();

    if fs::remove_file(&json_path).is_ok() {
        info!("Deleted JSON file: {}", json_path);
        deleted_files.push(json_path);
    } else {
        debug!("JSON file not found or couldn't be deleted: {}", json_path);
    }

    if fs::remove_file(&env_path).is_ok() {
        info!("Deleted ENV file: {}", env_path);
        deleted_files.push(env_path);
    } else {
        debug!("ENV file not found or couldn't be deleted: {}", env_path);
    }

    let message = if deleted_files.is_empty() {
        "No files were deleted.".to_string()
    } else {
        format!("Deleted files: {}", deleted_files.join(", "))
    };

    Ok(CommandResult {
        status: Status::Ok,
        message,
        env_file: None,
        json_file: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::{init, collect}, parse_config_file, safe_test};
    use std::io::Cursor;
    use uuid::Uuid;
    use std::fs;
    use tempfile::TempDir;

    safe_test!(test_delete_command_workflow, {
        let test_id = Uuid::new_v4().to_string();
        let temp_dir = TempDir::new()?;
        let input_path = temp_dir.path().join(format!("input-{}.json", test_id));

        // Step 1: Initialize a new configuration file
        {
            let mut output = Cursor::new(Vec::new());
            init::execute(input_path.to_str().unwrap(), &mut Cursor::new(Vec::new()), &mut output)?;
            assert!(input_path.exists(), "Input file should be created");
        }

        // Step 2: Load and parse the newly created config
        let mut config = parse_config_file(input_path.to_str().unwrap())?;

        // Step 3: Collect and save output files
        {
            let mut input = Cursor::new("s\nq\n"); // Save and quit
            let mut output = Cursor::new(Vec::new());
            collect::execute(&mut config, input_path.to_str().unwrap(), false, &mut input, &mut output)?;

            let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
            let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");

            assert!(fs::metadata(&json_path).is_ok(), "JSON file should be created");
            assert!(fs::metadata(&env_path).is_ok(), "ENV file should be created");
        }

        // Step 4: Delete output files (with confirmation)
        {
            let mut input = Cursor::new("y\n");
            let mut output = Cursor::new(Vec::new());

            let result = execute(&config, false, &mut input, &mut output)?;

            assert!(matches!(result.status, crate::Status::Ok));
            assert!(result.message.contains("Deleted files:"));

            let output_str = String::from_utf8(output.into_inner())?;
            assert!(output_str.contains("Are you sure you want to delete"));
        }

        // Step 5: Validate that output files are removed
        {
            let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
            let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");

            assert!(!fs::metadata(&json_path).is_ok(), "JSON file should be deleted");
            assert!(!fs::metadata(&env_path).is_ok(), "ENV file should be deleted");
        }

        // Step 6: Recreate the output files
        {
            let mut input = Cursor::new("s\nq\n"); // Save and quit
            let mut output = Cursor::new(Vec::new());
            collect::execute(&mut config, input_path.to_str().unwrap(), false, &mut input, &mut output)?;

            let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
            let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");

            assert!(fs::metadata(&json_path).is_ok(), "JSON file should be recreated");
            assert!(fs::metadata(&env_path).is_ok(), "ENV file should be recreated");
        }

        // Step 7: Try to delete output files but cancel
        {
            let mut input = Cursor::new("n\n");
            let mut output = Cursor::new(Vec::new());

            let result = execute(&config, false, &mut input, &mut output)?;

            assert!(matches!(result.status, crate::Status::Ok));
            assert_eq!(result.message, "Deletion cancelled.");

            let output_str = String::from_utf8(output.into_inner())?;
            assert!(output_str.contains("Are you sure you want to delete"));

            // Verify files still exist
            let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
            let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");

            assert!(fs::metadata(&json_path).is_ok(), "JSON file should still exist");
            assert!(fs::metadata(&env_path).is_ok(), "ENV file should still exist");
        }

        // Step 8: Delete output files with no confirmation
        {
            let mut input = Cursor::new("");
            let mut output = Cursor::new(Vec::new());

            let result = execute(&config, true, &mut input, &mut output)?;

            assert!(matches!(result.status, crate::Status::Ok));
            assert!(result.message.contains("Deleted files:"));

            let output_str = String::from_utf8(output.into_inner())?;
            assert!(!output_str.contains("Are you sure you want to delete"));

            // Verify files are deleted
            let json_path = json_output_uri!(&config).expect("Failed to construct JSON output path");
            let env_path = env_output_uri!(&config).expect("Failed to construct ENV output path");

            assert!(!fs::metadata(&json_path).is_ok(), "JSON file should be deleted");
            assert!(!fs::metadata(&env_path).is_ok(), "ENV file should be deleted");
        }

        Ok(())
    });
}
