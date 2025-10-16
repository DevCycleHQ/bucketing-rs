use crate::event::EvalDetails;
use crate::target::Target;
use crate::EvaluationReason;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct VariationVariable {
    pub _var: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReadOnlyVariable {
    #[serde(rename = "_id")]
    pub id: String,
    pub key: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub value: serde_json::Value,
    pub eval: EvalDetails,
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
    #[serde(rename = "_feature")]
    pub feature: String,
    #[serde(rename = "_variation")]
    pub variation: String,
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
    pub(crate) variations: Vec<Variation>,
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
    pub(crate) targets: Vec<Target>,
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
    #[serde(rename = "type", alias = "type")]
    pub _type: String,
    #[serde(rename = "_variation", alias = "_variation")]
    pub variation: String,
    #[serde(rename = "variationKey", alias = "variationKey")]
    pub variationkey: String,
    #[serde(rename = "variationName", alias = "variationName")]
    pub variationname: String,
    #[serde(rename = "evalReason", skip_serializing_if = "Option::is_none")]
    pub evalreason: Option<EvaluationReason>,
}
