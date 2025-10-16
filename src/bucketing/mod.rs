pub mod bucketing;

#[cfg(test)]
mod bucketing_tests;

// Re-export main function
pub use bucketing::generate_bucketed_config;

pub use bucketing::variable_for_user;
