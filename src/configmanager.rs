pub mod config_manager {
    use crate::config::config::ConfigBody;
    use std::collections::HashMap;
    use std::sync::Mutex;

    thread_local! {
         pub static CONFIGS: Mutex<HashMap<String, ConfigBody<'static>>> = Mutex::new(HashMap::new());
    }
}
