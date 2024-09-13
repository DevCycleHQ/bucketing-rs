pub(crate) mod config {
    use std::collections::HashMap;
    use crate::feature::feature::ConfigFeature;
    use crate::filters::filters::NoIdAudience;

    pub(crate) struct Project {
        pub _id: String,
        pub key: String,
        pub a0_organization: String,
        pub settings: ProjectSettings,
    }

    pub(crate) struct ProjectSettings {
        pub edgedb: EdgeDBSettings,
        pub optin: OptInSettings,
        pub disable_passthrough_rollouts: bool,
    }

    pub(crate) struct EdgeDBSettings {
        enabled: bool,
    }

    pub(crate) struct OptInSettings {
        enabled: bool,
        title: String,
        description: String,
        image_url: String,
        colors: OptInColors,
    }
    pub(crate) struct OptInColors {
        primary: String,
        secondary: String,
    }

    pub(crate) struct Environment {
        _id: String,
        key: String,
    }

    pub(crate) struct BucketingConfiguration {
        flush_events_interval: u64,
        disable_automatic_event_logging: bool,
        disable_custom_event_logging: bool,
        disable_push_state_event_logging: bool,
    }

    pub(crate) struct SSE {
        sse: SSEHost,
    }

    pub(crate) struct SSEHost {
        hostname: String,
        path: String,
    }

    pub(crate) struct Variable {
        _id: String,
        _type: String,
        key: String,
    }
    pub(crate) struct ConfigBody<'a> {
        pub(crate) project: Project,
        pub(crate) audiences: &'a HashMap<String, NoIdAudience<'a>>,
        pub(crate) environment: Environment,
        pub(crate) features: Vec<ConfigFeature>,
        pub(crate) variables: Vec<Variable>,
        pub(crate) sse: SSE,
        pub(crate) variable_id_map: HashMap<String, Variable>,
        pub(crate) variable_key_map: HashMap<String, Variable>,
        pub(crate) variable_id_to_feature_map: HashMap<String, ConfigFeature>,
    }
}