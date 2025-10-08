use crate::user::{PopulatedUser, User};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EventType {
    AggregateVariableEvaluated,
    AggregateVariableDefaulted,
    VariableEvaluated,
    VariableDefaulted,
    SDKConfig,
    CustomEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EvaluationReason {
    TargetingMatch,
    Split,
    Default,
    Error,
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
