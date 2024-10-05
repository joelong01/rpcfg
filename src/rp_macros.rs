use std::sync::atomic::{AtomicUsize, AtomicBool};
use serde::{Serialize, Deserialize};


/// Global flag for verbose output
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Global counter for trace calls
pub static TRACE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Ok,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandResult {
    pub status: Status,
    pub message: String,
    pub env_file: Option<String>,
    pub json_file: Option<String>,
}

// Helper macros
#[macro_export]
macro_rules! get_home_dir {
    () => {{
        std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(std::path::PathBuf::from)
    }};
}


#[macro_export]
macro_rules! get_rp_dir {
    () => {{
        if std::env::var("RP_TEST_MODE").is_ok() {
            use std::sync::OnceLock;
            static TEMP_DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
            Some(TEMP_DIR
                .get_or_init(|| tempfile::TempDir::new().expect("Failed to create temp directory"))
                .path()
                .to_path_buf())
        } else {
            $crate::get_home_dir!().map(|home| home.join(".rp"))
        }
    }};
}

#[macro_export]
macro_rules! get_base_name {
    ($input_file:expr) => {{
        std::path::Path::new($input_file).file_stem().and_then(|s| s.to_str()).map(String::from)
    }};
}

/// Macro for creating a success CommandResult
#[macro_export]
macro_rules! Success {
    ($($arg:tt)*) => {{
        $crate::rp_macros::CommandResult {
            status: $crate::rp_macros::Status::Ok,
            message: format!($($arg)*),
            env_file: None,
            json_file: None,
        }
    }};
}

/// Macro for creating a failure CommandResult
#[macro_export]
macro_rules! Fail {
    ($($arg:tt)*) => {{
        $crate::rp_macros::CommandResult {
            status: $crate::rp_macros::Status::Error,
            message: format!($($arg)*),
            env_file: None,
            json_file: None,
        }
    }};
}

/// Macro for getting the JSON output file path
#[macro_export]
macro_rules! JsonOutputUri {
    ($storage_type:expr, $input_file:expr) => {{
        if $storage_type == "local" {
            $crate::get_rp_dir!().and_then(|rp_dir| {
                $crate::get_base_name!($input_file).map(|base_name| {
                    rp_dir.join(format!("{}-values.json", base_name)).to_string_lossy().into_owned()
                })
            })
        } else {
            None
        }
    }};
}

/// Macro for getting the ENV output file path
#[macro_export]
macro_rules! EnvOutputUri {
    ($storage_type:expr, $input_file:expr) => {{
        if $storage_type == "local" {
            $crate::get_rp_dir!().and_then(|rp_dir| {
                $crate::get_base_name!($input_file).map(|base_name| {
                    rp_dir.join(format!("{}.env", base_name)).to_string_lossy().into_owned()
                })
            })
        } else {
            None
        }
    }};
}
