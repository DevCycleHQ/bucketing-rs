pub mod config_manager {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use crate::config::config::ConfigBody;

    thread_local! {
         pub static CONFIGS: Mutex<HashMap<String, ConfigBody<'static>>> = Mutex::new(HashMap::new());
    }
}
