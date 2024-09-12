pub mod config {
    struct BucketingConfiguration {
        flush_events_interval: u64,
        disable_automatic_event_logging: bool,
        disable_custom_event_logging: bool,
        disable_push_state_event_logging: bool,
    }
}