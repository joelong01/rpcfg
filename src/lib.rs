// src/lib.rs
pub mod models;
pub mod commands;
pub mod rp_macros;

// Re-export important structs and macros
pub use models::{Config, ConfigItem};
pub use rp_macros::*;  // This will re-export all macros and other items from rp_macros

#[cfg(test)]
#[path = "../tests/common.rs"]
pub mod tests;