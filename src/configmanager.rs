use crate::config::ConfigBody;
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

pub(crate) static CONFIGS: Lazy<RwLock<HashMap<String, ConfigBody<'static>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
