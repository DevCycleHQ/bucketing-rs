use crate::events::event_queue::EventQueue;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) static EVENT_QUEUES: Lazy<RwLock<HashMap<String, Arc<EventQueue>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) fn get_event_queue(sdk_key: &str) -> Option<Arc<EventQueue>> {
    let configs = EVENT_QUEUES.read().expect("EVENT_QUEUES RwLock poisoned");
    configs.get(sdk_key).cloned() // Clones the Arc, not the EventQueue
}

pub(crate) fn set_event_queue(sdk_key: &str, config: EventQueue) {
    let mut configs = EVENT_QUEUES.write().expect("EVENT_QUEUES RwLock poisoned");
    configs.insert(sdk_key.to_string(), Arc::new(config));
}

pub(crate) fn has_event_queue(sdk_key: &str) -> bool {
    let configs = EVENT_QUEUES.read().expect("EVENT_QUEUES RwLock poisoned");
    configs.contains_key(sdk_key)
}
