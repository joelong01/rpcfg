
# Repository Configuration CLI Tool

This CLI tool manages repository configurations, allowing users to collect, delete, fetch, and show configuration data.

## Table of Contents

- [Repository Configuration CLI Tool](#repository-configuration-cli-tool)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Commands](#commands)
  - [Options](#options)
  - [Input](#input)
  - [Output](#output)
  - [Examples](#examples)
  - [Implementation](#implementation)

## Installation

To install the tool, clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/repo-config-cli.git
cd repo-config-cli
cargo build --release
cargo test
```

The binary will be available in `target/release/rp`.

## Usage

The general syntax for using the tool is:

```bash
rp [OPTIONS] <COMMAND>
```

## Commands

- `collect`: Collect repository configurations and generate output files
- `delete`: Delete generated output files
- `fetch`: Return the JSON config with the values
- `show`: Show the configuration table

## Options

- `-f, --input-file <FILE>`: Path to the input JSON file (global option)
- `-i, --interactive`: Use interactive mode to collect values (global option)
- `--trace-level <LEVEL>`: Set the tracing level (off, error, warn, info, debug, trace) (global option, default: info)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Input

The primary input for this tool is a JSON configuration file. The file should have the following structure:

```json
{
  "stored": "local",
  "config_version": "1.0",
  "project_name": "example_project",
  "config_name": "example_config",
  "is_test": false,
  "items": [
    {
      "key": "item1",
      "description": "Description for item 1",
      "shellscript": "",
      "default": "default_value_1",
      "temp_environment_variable_name": "TEMP_ENV_VAR_1",
      "required_as_env": true,
      "value": ""
    },
    {
      "key": "item2",
      "description": "Description for item 2",
      "shellscript": "",
      "default": "default_value_2",
      "temp_environment_variable_name": "",
      "required_as_env": false,
      "value": ""
    }
  ]
}
```

## Output

The tool generates two types of output files:

1. JSON file: Contains the updated configuration data
2. ENV file: Contains environment variables based on the configuration

The exact paths of these files are returned in the command result.

## Examples

1. Collect configuration in interactive mode:

```bash
rp -f repo_config.json -i collect
```

2. Show configuration:

```bash
rp -f repo_config.json show
```

3. Fetch configuration:

```bash
rp -f repo_config.json fetch
```

4. Delete generated files:

```bash
rp -f repo_config.json delete
```

## Implementation

The following table lists the main crates used in this project, along with their usage:

| Crate | Modules | Usage |
|-------|---------|-------|
| `clap` | `main.rs` | Command-line argument parsing |
| `anyhow` | Throughout | Error handling and propagation |
| `serde` | `models.rs`, `main.rs` | JSON serialization and deserialization |
| `tracing` | Throughout | Logging and debugging |
| `tracing-subscriber` | `common.rs`, `main.rs` | Setting up the tracing subscriber |
| `uuid` | `commands/collect.rs`, test modules | Generating unique identifiers for tests |
| `tabwriter` | `commands/collect.rs`, `main.rs` | Formatting tabular output |
| `tempfile` | Test modules | Creating temporary files for testing |
| `std::fs` | Throughout | File system operations |
| `std::io` | Throughout | Input/output operations |
| `std::collections` | `main.rs` | Using `HashMap` for data storage |
| `std::panic` | `common.rs` | Handling panics in tests |
| `std::sync` | `common.rs` | Using `Once` for one-time initialization |

The project is structured into several modules:

- `main.rs`: Entry point and CLI setup
- `models.rs`: Data structures for configuration
- `commands/`: Submodules for each command (collect, delete, fetch, show)
- `common.rs`: Common utilities and test helpers
- `rp_macros.rs`: Custom macros for the project

Each module uses a combination of these crates to implement its functionality, with error handling, logging, and serialization being common themes throughout the project.
