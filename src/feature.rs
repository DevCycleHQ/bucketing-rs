use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::target::Target;

#[derive(Clone, Serialize, Deserialize)]
pub struct VariationVariable {
    pub _var: String,             // `json:"_var"`
    pub value: serde_json::Value, // {} `json:"value"`
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReadOnlyVariable {
    pub id: String,
    pub key: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Variation {
    pub(crate) _id: String,
    pub(crate) name: String,
    pub(crate) key: String,
    pub(crate) variables: Vec<VariationVariable>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FeatureVariation {
    pub feature: String,   // `json:"_feature"`
    pub variation: String, // `json:"_variation"`
}

impl Variation {
    pub fn get_variable_by_id(&self, id: &str) -> Option<&VariationVariable> {
        for variable in &self.variables {
            if variable._var == id {
                return Some(variable);
            }
        }
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigFeature {
    pub _id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub key: String,
    pub variations: Vec<Variation>,
    pub configuration: FeatureConfiguration,
    #[serde(default)]
    pub settings: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FeatureConfiguration {
    pub _id: String,
    #[serde(default)]
    pub prerequisites: Vec<FeaturePrerequisites>,
    #[serde(default)]
    pub winning_variation: Option<FeatureVariation>,
    #[serde(default)]
    pub forced_users: HashMap<String, String>,
    pub targets: Vec<Target>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FeaturePrerequisites {
    _feature: String,   // `json:"_feature"`
    comparator: String, // `json:"comparator"`
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Feature {
    pub _id: String,
    pub key: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub variation: String,
    pub variationkey: String,
    pub variationname: String,
    pub evalreason: Option<String>,
}
