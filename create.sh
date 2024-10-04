#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Initialize a new git repository
echo "Initializing local git repository..."
git init

# Initialize a new GitHub repository named "rp" in the current directory
echo "Creating GitHub repository..."
gh repo create rp --public --source=.

# Create the project structure
echo "Creating project structure..."
mkdir -p src/commands
touch src/main.rs src/commands/mod.rs src/commands/collect.rs src/commands/delete.rs src/commands/fetch.rs

# Create Cargo.toml with proper structure and dependencies
echo "Creating Cargo.toml..."
cat << EOF > Cargo.toml
[package]
name = "rp"
version = "0.1.0"
edition = "2021"
authors = ["Joe Long <joelong@outlook.com>"]
description = "A CLI tool for managing repository configurations"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[[bin]]
name = "rp"
path = "src/main.rs"

EOF

# Add files and make initial commit
git add .
git commit -m "Initial commit: Project structure and Cargo.toml"

echo "Project structure created. You can now start implementing the CLI functionality."