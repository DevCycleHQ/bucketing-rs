pub mod config_manager {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use crate::config::config::ConfigBody;

    pub(crate) static CONFIGS: Mutex<HashMap<String, ConfigBody>> = Mutex::new(HashMap::new());
}
