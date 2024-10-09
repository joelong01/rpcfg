use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::path::PathBuf;

/// Global flag for verbose output
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Global counter for trace calls
pub static TRACE_COUNT: AtomicUsize = AtomicUsize::new(0);

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
        if $config.is_test {
            std::env::temp_dir().join(".rpcfg")
        } else {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
                .join(".rpcfg")
        }
    }};
}

#[macro_export]
macro_rules! get_base_name {
    ($input_file:expr) => {{
        std::path::Path::new($input_file).file_stem().and_then(|s| s.to_str()).map(String::from)
    }};
}

pub fn base_output_dir(config: &crate::Config) -> Option<PathBuf> {
    let stored = config.get_settings("stored").first().map(|item| item.value.as_str()).unwrap_or("local");
    let project_name = config.get_settings("project_name").first().map(|item| item.value.as_str()).unwrap_or("default_project");
    let config_name = config.get_settings("config_name").first().map(|item| item.value.as_str()).unwrap_or("default_config");
    let environment = config.get_settings("environment").first().map(|item| item.value.as_str()).unwrap_or("default_env");

    if stored == "local" {
        Some(crate::get_rp_dir!(config)
            .join(project_name)
            .join(format!("{}-{}", config_name, environment)))
    } else {
        None
    }
}

/// Macro for creating a success CommandResult
#[macro_export]
macro_rules! Success {
    ($($arg:tt)*) => {{
        $crate::models::CommandResult {
            status: $crate::models::Status::Ok,
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
        $crate::models::CommandResult {
            status: $crate::models::Status::Error,
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
        $crate::rp_macros::base_output_dir($config).map(|path| path.with_extension("json").to_string_lossy().into_owned())
    }};
}

/// Macro for getting the ENV output file path
#[macro_export]
macro_rules! EnvOutputUri {
    ($config:expr) => {{
        $crate::rp_macros::base_output_dir($config).map(|path| path.with_extension("env").to_string_lossy().into_owned())
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

#[macro_export]
macro_rules! create_test_config {
    ($test_id:expr) => {
        crate::test_utils::create_test_config($test_id)
    };
}

