# Repository Configuration CLI Tool

This CLI tool manages repository configurations, allowing users to collect, store, and retrieve configuration settings for various projects and environments.

## Features

- Interactive and non-interactive configuration collection
- Storage of configurations in both JSON and ENV formats
- Support for local storage (future support for remote storage planned)
- Separation of core configuration (rpcfg) and application-specific (app) settings
- Environment variable generation for required settings
- Flexible configuration schema supporting various data types

## Installation

To install the tool, clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/repo-config-cli.git
cd repo-config-cli
cargo build --release
cargo test
```

The binary will be available in `target/release/rpcfg`.

## Usage

The general syntax for using the tool is:

```bash
rpcfg [OPTIONS] <COMMAND>
```

## Commands

- `init`: Initialize a new configuration file
- `collect`: Collect repository configurations and generate output files
- `delete`: Delete generated output files
- `fetch`: Return the JSON config with the values
- `show`: Show the configuration table

## Options

- `-i, --input <FILE>`: Path to the input JSON file (global option)
- `-s, --silent`: Use silent (non-interactive) mode (global option)
- `--trace-level <LEVEL>`: Set the tracing level (off, error, warn, info, debug, trace) (global option, default: error)
- `-h, --help`: Print help information
- `-v, --version`: Print version information

## Command-specific Options

### Init

- `-o, --output <FILE>`: Path to the output JSON file

### Collect

- `-i, --input <FILE>`: Path to the input JSON file
- `--ignore-timestamps`: Ignore timestamp checks and always collect

## Input

The primary input for this tool is a JSON configuration file. The file should have the following structure:

```json
{
  "rpcfg": [
    {
      "key": "config_version",
      "description": "Version of the configuration",
      "shellscript": "",
      "default": "1.0",
      "temp_environment_variable_name": "",
      "required_as_env": false
    },
    {
      "key": "project_name",
      "description": "Name of the project",
      "shellscript": "",
      "default": "rpcfg",
      "temp_environment_variable_name": "",
      "required_as_env": false
    },
    {
      "key": "config_name",
      "description": "Name of the configuration",
      "shellscript": "",
      "default": "rpcfg_config",
      "temp_environment_variable_name": "",
      "required_as_env": false
    },
    {
      "key": "environment",
      "description": "Environment for the configuration",
      "shellscript": "",
      "default": "development",
      "temp_environment_variable_name": "",
      "required_as_env": false
    }
  ],
  "app": []
}
```

## Output

The tool generates two types of output files:

1. JSON file: Contains the updated configuration data
2. ENV file: Contains environment variables based on the configuration

The exact paths of these files are returned in the command result.

## Examples

1. Initialize a new configuration file:

   ```bash
   rpcfg init -o mysettings.json
   ```

2. Collect configuration in interactive mode:

   ```bash
   rpcfg collect -i repo_config.json
   ```

3. Collect configuration, ignoring timestamps:

   ```bash
   rpcfg collect -i repo_config.json --ignore-timestamps
   ```

4. Show configuration:

   ```bash
   rpcfg show -i repo_config.json
   ```

5. Fetch configuration:

   ```bash
   rpcfg fetch -i repo_config.json
   ```

6. Delete generated files:

   ```bash
   rpcfg delete -i repo_config.json
   ```

## Implementation

The following table lists the main crates used in this project, along with their usage:

| Crate                | Modules                             | Usage                                                   |
| -------------------- | ----------------------------------- | ------------------------------------------------------- |
| `clap`               | `main.rs`                           | Command-line argument parsing                           |
| `anyhow`             | Throughout                          | Error handling and propagation                          |
| `serde`              | `models.rs`, `main.rs`              | JSON serialization and deserialization                  |
| `tracing`            | Throughout                          | Logging and debugging                                   |
| `tracing-subscriber` | `main.rs`                           | Setting up the tracing subscriber                       |
| `uuid`               | `commands/collect.rs`, test modules | Generating unique identifiers for tests                 |
| `tabwriter`          | `commands/collect.rs`, `main.rs`    | Formatting tabular output                               |
| `tempfile`           | Test modules                        | Creating temporary files for testing                    |
| `std::fs`            | Throughout                          | File system operations                                  |
| `std::io`            | Throughout                          | Input/output operations                                 |
| `std::collections`   | `main.rs`, `commands/fetch.rs`      | Using `HashMap` for data storage                        |

The project is structured into several modules:

- `main.rs`: Entry point and CLI setup
- `models.rs`: Data structures for configuration
- `commands/`: Submodules for each command (init, collect, delete, fetch, show)

Each module uses a combination of these crates to implement its functionality, with error handling, logging, and serialization being common themes throughout the project.
