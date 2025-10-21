pub(crate) mod client_custom_data;
pub(crate) mod config;
pub(crate) mod configmanager;
pub(crate) mod feature;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod configmanager_tests;
pub mod platform_data;
#[cfg(test)]
mod platform_data_tests;

// Re-export commonly used types only within crate
pub(crate) use config::*;
pub(crate) use feature::*;
