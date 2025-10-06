use crate::feature::ConfigFeature;
use crate::filters::NoIdAudience;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::DateTime;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    pub _id: String,
    pub key: String,
    pub a0_organization: String,
    pub settings: ProjectSettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProjectSettings {
    #[serde(alias = "edgeDB", rename="edgeDB",)]
    pub edgedb: EdgeDBSettings,
    #[serde(alias = "optIn", rename="optIn", default)]
    pub optin: OptInSettings,
    #[serde(alias = "disablePassthroughRollouts", rename="disablePassthroughRollouts", default)]
    pub disable_passthrough_rollouts: bool,
    #[serde(skip_serializing)]
    pub obfuscation: Option<ObfuscationSettings>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct EdgeDBSettings {
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct OptInSettings {
    pub enabled: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "imageURL", rename="imageURL")]
    pub image_url: String,
    #[serde(default)]
    pub colors: OptInColors,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct OptInColors {
    #[serde(default)]
    pub primary: String,
    #[serde(default)]
    pub secondary: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ObfuscationSettings {
    pub required: bool,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Environment {
    pub _id: String,
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BucketingConfiguration {
    pub flush_events_interval: u64,
    pub disable_automatic_event_logging: bool,
    pub disable_custom_event_logging: bool,
    pub disable_push_state_event_logging: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SSE {
    pub hostname: String,
    pub path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SSEHost {
    pub hostname: String,
    pub path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Variable {
    pub _id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub key: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct FullConfig {
    pub project: Project,
    pub environment: Environment,
    pub features: Vec<ConfigFeature>,
    pub variables: Vec<Variable>,
    #[serde(rename = "variableHashes")]
    pub variable_hashes: HashMap<String, u64>,
    #[serde(default)]
    pub audiences: HashMap<String, serde_json::Value>,
    #[serde(rename = "debugUsers", default)]
    pub debug_users: Vec<serde_json::Value>,
    #[serde(default)]
    pub sse: Option<SSE>,
}

pub(crate) struct ConfigBody<'a> {
    pub(crate) project: Project,
    pub(crate) audiences: &'a HashMap<String, NoIdAudience>,
    pub(crate) environment: Environment,
    pub(crate) features: Vec<ConfigFeature>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) sse: SSE,
    pub(crate) variable_id_map: HashMap<String, Variable>,
    pub(crate) variable_key_map: HashMap<String, Variable>,
    pub(crate) variable_id_to_feature_map: HashMap<String, ConfigFeature>,
    pub(crate) etag: String,
    pub(crate) ray_id: String,
    pub(crate) last_modified: DateTime<chrono::Utc>,
}

impl<'a> ConfigBody<'a> {
    pub(crate) fn get_variable_for_key(&self, key: &str) -> Option<&Variable> {
        if let Some(variable) = self.variable_key_map.get(key) {
            return Some(variable);
        }
        None
    }

    pub(crate) fn get_feature_for_key(&self, key: &str) -> Option<&ConfigFeature> {
        if let Some(feature) = self.features.iter().find(|f| f.key == key) {
            return Some(feature);
        }
        None
    }

    pub(crate) fn get_variable_for_id(&self, id: &str) -> Option<&Variable> {
        if let Some(variable) = self.variable_id_map.get(id) {
            return Some(variable);
        }
        None
    }

    pub(crate) fn get_feature_for_variable_id(&self, variable_id: &str) -> Option<&ConfigFeature> {
        if let Some(feature) = self.variable_id_to_feature_map.get(variable_id) {
            return Some(feature);
        }
        None
    }

    pub(crate) fn compile(&mut self) {
        // Build mapping of variable IDs to features
        for feature in &self.features {
            for variation in &feature.variations {
                for variable in &variation.variables {
                    if !self.variable_id_to_feature_map.contains_key(&variable._var) {
                        self.variable_id_to_feature_map.insert(variable._var.clone(), feature.clone());
                    }
                }
            }
        }

        // Build mappings for variables by key and ID
        for variable in &self.variables {
            self.variable_key_map.insert(variable.key.clone(), variable.clone());
            self.variable_id_map.insert(variable._id.clone(), variable.clone());
        }

        // Sort the feature distributions by "_variation" attribute in descending alphabetical order
        for feature in &mut self.features {
            for target in &mut feature.configuration.targets {
                target.distribution.sort_by(|a, b| b.variation.cmp(&a.variation));
            }
        }
    }
}
