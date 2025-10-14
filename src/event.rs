use crate::user::{PopulatedUser, User};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EventType {
    AggregateVariableEvaluated,
    AggregateVariableDefaulted,
    VariableEvaluated,
    VariableDefaulted,
    SDKConfig,
    CustomEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvaluationReason {
    #[serde(rename = "TARGETING_MATCH")]
    TargetingMatch,
    #[serde(rename = "SPLIT")]
    Split,
    #[serde(rename = "DEFAULT")]
    Default,
    #[serde(rename = "DISABLED")]
    Disabled,
    #[serde(rename = "ERROR")]
    Error,
}

impl fmt::Display for EvaluationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EvaluationReason::TargetingMatch => "TARGETING_MATCH",
            EvaluationReason::Split => "SPLIT",
            EvaluationReason::Default => "DEFAULT",
            EvaluationReason::Disabled => "DISABLED",
            EvaluationReason::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefaultReason {
    #[serde(rename = "Missing Config")]
    MissingConfig,
    #[serde(rename = "Missing Variable")]
    MissingVariable,
    #[serde(rename = "Missing Feature")]
    MissingFeature,
    #[serde(rename = "Missing Variation")]
    MissingVariation,
    #[serde(rename = "Missing Variable for Variation")]
    MissingVariableForVariation,
    #[serde(rename = "User Not in Rollout")]
    UserNotInRollout,
    #[serde(rename = "User Not Targeted")]
    UserNotTargeted,
    #[serde(rename = "Invalid Variable Type")]
    InvalidVariableType,
    #[serde(rename = "Variable Type Mismatch")]
    VariableTypeMismatch,
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "Error")]
    Error,
    #[serde(rename = "")]
    NotDefaulted,
}

impl fmt::Display for DefaultReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DefaultReason::MissingConfig => "Missing Config",
            DefaultReason::MissingVariable => "Missing Variable",
            DefaultReason::MissingFeature => "Missing Feature",
            DefaultReason::MissingVariation => "Missing Variation",
            DefaultReason::MissingVariableForVariation => "Missing Variable for Variation",
            DefaultReason::UserNotInRollout => "User Not in Rollout",
            DefaultReason::UserNotTargeted => "User Not Targeted",
            DefaultReason::InvalidVariableType => "Invalid Variable Type",
            DefaultReason::VariableTypeMismatch => "Variable Type Mismatch",
            DefaultReason::Unknown => "Unknown",
            DefaultReason::Error => "Error",
            DefaultReason::NotDefaulted => "",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalDetails {
    pub reason: EvaluationReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

pub(crate) type EvalReasonAggMap = HashMap<EvaluationReason, i64>;
pub(crate) type VariationAggMap = HashMap<String, EvalReasonAggMap>;
pub(crate) type FeatureAggMap = HashMap<String, VariationAggMap>;
pub(crate) type VariableAggMap = HashMap<String, FeatureAggMap>;
pub(crate) type AggregateEventQueue = HashMap<EventType, VariableAggMap>;
pub(crate) type UserEventQueue = HashMap<String, UserEventsBatchRecord>;

pub(crate) struct Event {
    pub(crate) event_type: EventType,
    pub(crate) target: String,
    pub(crate) custom_type: String,
    pub(crate) user_id: String,
    pub(crate) client_date: std::time::Instant,
    pub(crate) value: f64,
    pub(crate) feature_vars: HashMap<String, String>,
    pub(crate) meta_data: HashMap<String, serde_json::Value>,
}

pub(crate) struct UserEventData {
    pub(crate) event: Event,
    pub(crate) user: User,
}

pub(crate) struct UserEventsBatchRecord {
    pub(crate) user: PopulatedUser,
    pub(crate) events: Vec<Event>,
}

pub(crate) struct AggEventQueueRawMessage {
    pub(crate) event_type: EventType,
    pub(crate) variable_key: String,
    pub(crate) feature_id: String,
    pub(crate) variation_id: String,
    pub(crate) eval_metadata: EvalReasonAggMap,
}
