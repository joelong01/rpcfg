use std::sync::atomic::{AtomicUsize, AtomicBool};



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
   #[macro_export]
   macro_rules! create_test_config {
       ($test_id:expr) => {
           crate::test_utils::create_test_config($test_id)
       };
   }