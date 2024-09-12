pub mod config {
    use std::collections::HashMap;
    use crate::feature::feature::ConfigFeature;
    use crate::filters::filters::NoIdAudience;

    struct Project {
        _id: String,
        key: String,
        a0_organization: String,
        settings: ProjectSettings,
    }

    struct ProjectSettings {
        edgedb: EdgeDBSettings,
        optin: OptInSettings,
        disable_passthrough_rollouts: bool,
    }

    struct EdgeDBSettings {
        enabled: bool
    }

    struct OptInSettings {
        enabled: bool,
        title: String,
        description: String,
        image_url: String,
        colors: OptInColors,
    }
    struct OptInColors {
        primary: String,
        secondary: String,
    }

    struct Environment {
        _id: String,
        key: String
    }

    struct BucketingConfiguration {
        flush_events_interval: u64,
        disable_automatic_event_logging: bool,
        disable_custom_event_logging: bool,
        disable_push_state_event_logging: bool,
    }

    struct SSE {
        sse: SSEHost
    }

    struct SSEHost{
        hostname: String,
        path: String,
    }

    struct Variable {
        _id: String,
        _type: String,
        key: String,
    }
    struct ConfigBody<'a>{
        project: Project,
        audiences: HashMap<String, NoIdAudience<'a>>,
        environment: Environment,
        features: Vec<ConfigFeature<'a>>,
        variables: Vec<Variable>,
        sse: SSE,
        variable_id_map: HashMap<String, Variable>,
        variable_key_map: HashMap<String, Variable>,
        variable_id_to_feature_map: HashMap<String, ConfigFeature<'a>>
    }
}