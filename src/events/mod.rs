pub mod event;
pub(crate) mod event_queue;
pub(crate) mod event_queue_manager;

#[cfg(test)]
mod event_queue_tests;
#[cfg(test)]
mod event_tests;

// Re-export event types that are used externally
pub use event::*;
// EventQueue and EventQueueOptions are used in lib.rs public API
pub use event_queue::EventQueueOptions;
