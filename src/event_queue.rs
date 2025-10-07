use crate::config::ConfigBody;
use crate::platform_data::PlatformData;
use crate::user::PopulatedUser;
use std::collections::HashMap;
use std::hash::Hash;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EventType {
    AggregateVariableEvaluated,
    AggregateVariableDefaulted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EvaluationReason {
    TargetingMatch,
    Split,
    Default,
    Error,
}
type EvalReasonAggMap = HashMap<EvaluationReason, i64>;
type VariationAggMap = HashMap<String, EvalReasonAggMap>; // variation_id -> (reason -> count)
type FeatureAggMap = HashMap<String, VariationAggMap>; // feature_id -> (variation_id -> (reason -> count))
type VariableAggMap = HashMap<String, FeatureAggMap>; // variable_key -> count
type AggregateEventQueue = HashMap<EventType, VariableAggMap>;
type UserEventQueue = HashMap<String, UserEventsBatchRecord>;
struct Event {}

struct UserEventsBatchRecord {
    user: PopulatedUser,
    events: Vec<Event>,
}

struct AggEventQueueRawMessage {
    event_type: String,
    variable_key: String,
    feature_id: String,
    variation_id: String,
    eval_metadata: HashMap<String, i64>,
}

struct EventQueue {
    sdk_key: String,
    agg_event_queue_raw: mpsc::Receiver<AggEventQueueRawMessage>,
    user_event_queue_raw: mpsc::Receiver<String>,
    agg_event_queue: AggregateEventQueue,
    user_event_queue: UserEventQueue,
    user_event_queue_count: i32,
    queue_access_mutex: tokio::sync::Mutex<()>,
    events_flushed: i64,
    events_dropped: i64,
    events_reported: i64,
    platform_data: *const PlatformData,
}

impl EventQueue {
    pub fn new(sdk_key: String, platform_data: *const PlatformData) -> (Self, mpsc::Sender<AggEventQueueRawMessage>, mpsc::Sender<String>) {
        let (agg_event_queue_raw_tx, agg_event_queue_raw_rx) = mpsc::channel(10000);
        let (user_event_queue_raw_tx, user_event_queue_raw_rx) = mpsc::channel(10000);
        (
            EventQueue {
                sdk_key,
                agg_event_queue_raw: agg_event_queue_raw_rx,
                user_event_queue_raw: user_event_queue_raw_rx,
                agg_event_queue: HashMap::new(),
                user_event_queue: HashMap::new(),
                user_event_queue_count: 0,
                queue_access_mutex: tokio::sync::Mutex::new(()),
                events_flushed: 0,
                events_dropped: 0,
                events_reported: 0,
                platform_data,
            },
            agg_event_queue_raw_tx,
            user_event_queue_raw_tx
        )
    }
    pub async unsafe fn merge_agg_event_queue_keys(&mut self, config_body: &ConfigBody<'_>) {
        let guard = self.queue_access_mutex.lock().await;
        for eventType in [EventType::AggregateVariableDefaulted, EventType::AggregateVariableEvaluated] {
            if !self.agg_event_queue.contains_key(&eventType) {
                self.agg_event_queue.insert(eventType.clone(), HashMap::new());
            }
            for variable in config_body.variables.iter() {
                if !self.agg_event_queue.get(&eventType).unwrap().contains_key(&variable.key) {
                    self.agg_event_queue.get_mut(&eventType).unwrap().insert(variable.key.clone(), HashMap::new());
                }
                for feature in (*config_body).features.iter() {
                    if !self.agg_event_queue.get(&eventType).unwrap().get(&variable.key).unwrap().contains_key(&feature.key) {
                        self.agg_event_queue.get_mut(&eventType).unwrap().get_mut(&variable.key).unwrap().insert(feature._id.clone(), HashMap::new());
                    }
                    for variation in feature.variations.iter() {
                        if !self.agg_event_queue.get(&eventType).unwrap().get(&variable.key).unwrap().get(&feature._id).unwrap().contains_key(&variation._id) {
                            self.agg_event_queue.get_mut(&eventType).unwrap().get_mut(&variable.key).unwrap().get_mut(&feature._id).unwrap().insert(variation._id.clone(), HashMap::new());
                        }
                        for reason in [EvaluationReason::TargetingMatch, EvaluationReason::Split, EvaluationReason::Default, EvaluationReason::Error] {
                            if !self.agg_event_queue.get(&eventType).unwrap().get(&variable.key).unwrap().get(&feature._id).unwrap().get(&variation._id).unwrap().contains_key(&reason) {
                                self.agg_event_queue.get_mut(&eventType).unwrap().get_mut(&variable.key).unwrap().get_mut(&feature._id).unwrap().get_mut(&variation._id).unwrap().insert(reason.clone(), 0);
                            }
                        }
                    }
                }
            }
        }
    }
}