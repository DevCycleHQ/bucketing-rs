use crate::config::{
    ConfigBody, EdgeDBSettings, Environment, OptInColors, OptInSettings, Project, ProjectSettings,
    Variable, SSE,
};
use crate::errors::{self, DevCycleError};
use crate::feature::ConfigFeature;
use crate::filters::NoIdAudience;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[cfg(feature = "protobuf")]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/devcycle.config.rs"));
}

#[cfg(feature = "protobuf")]
pub fn convert_proto_to_config_body(
    proto: proto::ConfigBodyProto,
) -> Result<ConfigBody<'static>, DevCycleError> {
    // Convert Project
    let proto_project = proto
        .project
        .ok_or_else(|| errors::missing_field("project".to_string()))?;
    let proto_settings = proto_project
        .settings
        .ok_or_else(|| errors::missing_field("project.settings".to_string()))?;

    let project = Project {
        _id: proto_project.id,
        key: proto_project.key,
        a0_organization: proto_project.a0_organization,
        settings: ProjectSettings {
            edgedb: EdgeDBSettings {
                enabled: proto_settings.edgedb.map(|e| e.enabled).unwrap_or_default(),
            },
            optin: OptInSettings {
                enabled: proto_settings
                    .optin
                    .as_ref()
                    .map(|o| o.enabled)
                    .unwrap_or_default(),
                title: proto_settings
                    .optin
                    .as_ref()
                    .map(|o| o.title.clone())
                    .unwrap_or_default(),
                description: proto_settings
                    .optin
                    .as_ref()
                    .map(|o| o.description.clone())
                    .unwrap_or_default(),
                image_url: proto_settings
                    .optin
                    .as_ref()
                    .map(|o| o.image_url.clone())
                    .unwrap_or_default(),
                colors: OptInColors {
                    primary: proto_settings
                        .optin
                        .as_ref()
                        .and_then(|o| o.colors.as_ref())
                        .map(|c| c.primary.clone())
                        .unwrap_or_default(),
                    secondary: proto_settings
                        .optin
                        .as_ref()
                        .and_then(|o| o.colors.as_ref())
                        .map(|c| c.secondary.clone())
                        .unwrap_or_default(),
                },
            },
            disable_passthrough_rollouts: proto_settings.disable_passthrough_rollouts,
            obfuscation: None,
        },
    };

    // Convert Environment
    let proto_env = proto
        .environment
        .ok_or_else(|| errors::missing_field("environment".to_string()))?;
    let environment = Environment {
        _id: proto_env.id,
        key: proto_env.key,
    };

    // Convert SSE
    let proto_sse = proto
        .sse
        .ok_or_else(|| errors::missing_field("sse".to_string()))?;
    let sse = SSE {
        hostname: proto_sse.hostname,
        path: proto_sse.path,
    };

    // Convert Variables
    let variables: Vec<Variable> = proto
        .variables
        .into_iter()
        .map(|v| Variable {
            _id: v.id,
            _type: v.r#type,
            key: v.key,
        })
        .collect();

    // Convert Audiences - need to deserialize JSON
    let audiences: HashMap<String, NoIdAudience> = proto
        .audiences
        .into_iter()
        .map(|(key, audience)| {
            let no_id_audience: NoIdAudience = serde_json::from_str(&audience.data)
                .map_err(|e| errors::parse_error(format!("Failed to parse audience: {}", e)))?;
            Ok((key, no_id_audience))
        })
        .collect::<Result<HashMap<_, _>, DevCycleError>>()?;

    // Convert Features - need to deserialize JSON
    let features: Vec<ConfigFeature> = proto
        .features
        .into_iter()
        .map(|f| {
            serde_json::from_str(&f.data)
                .map_err(|e| errors::parse_error(format!("Failed to parse feature: {}", e)))
        })
        .collect::<Result<Vec<_>, DevCycleError>>()?;

    // Create maps for variables
    let variable_id_map: HashMap<String, Variable> = variables
        .iter()
        .map(|v| (v._id.clone(), v.clone()))
        .collect();
    let variable_key_map: HashMap<String, Variable> = variables
        .iter()
        .map(|v| (v.key.clone(), v.clone()))
        .collect();

    // Create variable_id_to_feature_map
    let variable_id_to_feature_map: HashMap<String, ConfigFeature> = features
        .iter()
        .flat_map(|feature| {
            feature
                .variations
                .iter()
                .flat_map(|variation| variation.variables.clone())
                .map(move |var_id| (var_id._var.clone(), feature.clone()))
        })
        .collect();

    // Convert timestamp
    let last_modified = Utc
        .timestamp_opt(proto.last_modified, 0)
        .single()
        .ok_or_else(|| errors::parse_error("Invalid timestamp".to_string()))?;

    // Leak audiences to get 'static lifetime
    let audiences_static = Box::leak(Box::new(audiences));

    Ok(ConfigBody {
        project,
        audiences: audiences_static,
        environment,
        features,
        variables,
        sse,

        variable_id_map,
        variable_key_map,
        variable_id_to_feature_map,
        etag: proto.etag,
        ray_id: proto.ray_id,
        last_modified,
    })
}
