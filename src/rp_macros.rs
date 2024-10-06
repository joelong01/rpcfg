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
    ($config:expr) => {{
        let root_dir = $crate::get_root_dir!($config);
        root_dir.join(".rp")
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
    ($config:expr) => {{
        if $config.stored == "local" {
            Some($crate::get_rp_dir!($config)
                .join(&$config.project_name)
                .join(format!("{}_values.json", $config.config_name))
                .to_string_lossy()
                .into_owned())
        } else {
            None
        }
    }};
}

/// Macro for getting the ENV output file path
#[macro_export]
macro_rules! EnvOutputUri {
    ($config:expr) => {{
        if $config.stored == "local" {
            Some($crate::get_rp_dir!($config)
                .join(&$config.project_name)
                .join(format!("{}.env", $config.config_name))
                .to_string_lossy()
                .into_owned())
        } else {
            None
        }
    }};
}

#[macro_export]
macro_rules! get_root_dir {
    ($config:expr) => {{
        if $config.is_test {
            std::env::temp_dir()
        } else {
            $crate::get_home_dir!().expect("Failed to get home directory")
        }
    }};
}
