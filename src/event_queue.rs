use crate::client_custom_data::get_client_custom_data;
use crate::config::ConfigBody;
use crate::errors::DevCycleError;
use crate::generate_bucketed_config;
use crate::platform_data::PlatformData;
use crate::user::{PopulatedUser, User};
use crate::event::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::time::Duration;
use tokio::sync::mpsc;



pub(crate) struct EventQueueOptions {
    pub flush_events_interval: Duration,
    pub disable_automatic_event_logging: bool,
    pub disable_custom_event_logging: bool,
    pub max_event_queue_size: i32,
    pub max_user_event_queue_size: i32,
    pub flush_events_batch_size: i32,
    pub flush_events_queue_size: i32,
    pub events_api_base_uri: String,
}

impl EventQueueOptions {
    pub fn is_event_logging_disabled(&self, event_type: *const EventType) -> bool {
        if event_type == &EventType::CustomEvent {
            return self.disable_custom_event_logging;
        }
        return self.disable_automatic_event_logging;
    }
}

impl Default for EventQueueOptions {
    fn default() -> Self {
        EventQueueOptions {
            flush_events_interval: Duration::from_secs(60),
            disable_automatic_event_logging: false,
            disable_custom_event_logging: false,
            max_event_queue_size: 10000,
            max_user_event_queue_size: 1000,
            flush_events_batch_size: 100,
            flush_events_queue_size: 1000,
            events_api_base_uri: "https://events.devcycle.com".to_string(),
        }
    }
}
pub(crate) struct EventQueue {
    pub(crate) sdk_key: String,
    pub(crate) agg_event_queue_raw_tx: mpsc::Sender<crate::event::AggEventQueueRawMessage>,
    pub(crate) agg_event_queue_raw_rx: mpsc::Receiver<crate::event::AggEventQueueRawMessage>,
    pub(crate) user_event_queue_raw_tx: mpsc::Sender<String>,
    pub(crate) user_event_queue_raw_rx: mpsc::Receiver<String>,
    pub(crate) agg_event_queue: crate::event::AggregateEventQueue,
    pub(crate) user_event_queue: crate::event::UserEventQueue,
    pub(crate) user_event_queue_count: i32,
    pub(crate) queue_access_mutex: tokio::sync::Mutex<()>,
    pub(crate) events_flushed: i64,
    pub(crate) events_dropped: i64,
    pub(crate) events_reported: i64,
    pub(crate) platform_data: *const PlatformData,
    pub(crate) options: crate::event_queue::EventQueueOptions,
}

impl EventQueue {
    pub fn new(
        sdk_key: String,
        event_queue_options: EventQueueOptions,
        platform_data: *const PlatformData,
    ) -> Self {
        let (agg_event_queue_raw_tx, agg_event_queue_raw_rx) = mpsc::channel(10000);
        let (user_event_queue_raw_tx, user_event_queue_raw_rx) = mpsc::channel(10000);
        Self {
            sdk_key,
            agg_event_queue_raw_tx,
            user_event_queue_raw_tx,
            agg_event_queue_raw_rx,
            user_event_queue_raw_rx,
            agg_event_queue: HashMap::new(),
            user_event_queue: HashMap::new(),
            user_event_queue_count: 0,
            queue_access_mutex: tokio::sync::Mutex::new(()),
            events_flushed: 0,
            events_dropped: 0,
            events_reported: 0,
            platform_data,
            options: event_queue_options,
        }
    }

    pub async fn queue_variable_evaluated_event(
        &mut self,
        variable_key: &str,
        feature_id: &str,
        variation_id: &str,
        eval_reason: EvaluationReason,
    ) -> Result<bool, DevCycleError> {
        return self
            .queue_aggregate_event_internal(
                variable_key,
                feature_id,
                variation_id,
                EventType::AggregateVariableEvaluated,
                eval_reason,
            )
            .await;
    }
    pub async fn queue_variable_defaulted_event(
        &mut self,
        variable_key: &str,
        feature_id: &str,
        variation_id: &str,
    ) -> Result<bool, DevCycleError> {
        return self
            .queue_aggregate_event_internal(
                variable_key,
                feature_id,
                variation_id,
                EventType::AggregateVariableDefaulted,
                EvaluationReason::Default,
            )
            .await;
    }

    async fn queue_aggregate_event_internal(
        &mut self,
        variable_key: &str,
        feature_id: &str,
        variation_id: &str,
        event_type: EventType,
        eval_reason: EvaluationReason,
    ) -> Result<bool, DevCycleError> {
        if self.options.is_event_logging_disabled(&event_type) {
            return Ok(false);
        }
        if variable_key.is_empty() {
            return Err(DevCycleError::new(
                "a variable key is required for aggregate events",
            ));
        }
        let mut eval: EvalReasonAggMap = HashMap::new();
        if event_type == EventType::AggregateVariableDefaulted {
            eval.insert(EvaluationReason::Default, 1);
        } else {
            eval.insert(eval_reason, 1);
        }

        let success = self
            .agg_event_queue_raw_tx
            .send(AggEventQueueRawMessage {
                event_type,
                variation_id: variation_id.to_string(),
                feature_id: feature_id.to_string(),
                variable_key: variable_key.to_string(),
                eval_metadata: eval,
            })
            .await;

        if success.is_err() {
            self.events_dropped +=1 ;
        }
        return Ok(true);
    }

    pub async fn queue_event(&mut self, user: User, event: Event) -> Result<bool, DevCycleError> {
        let success = self
            .user_event_queue_raw_tx
            .send(user.user_id.clone())
            .await;
        if success.is_err() {
            self.events_dropped += 1;
        }
        return Ok(true);
    }
    pub async fn merge_agg_event_queue_keys(&mut self, config_body: &ConfigBody<'_>) {
        let guard = self.queue_access_mutex.lock().await;
        for event_type in [
            EventType::AggregateVariableDefaulted,
            EventType::AggregateVariableEvaluated,
        ] {
            if !self.agg_event_queue.contains_key(&event_type) {
                self.agg_event_queue
                    .insert(event_type.clone(), HashMap::new());
            }
            for variable in config_body.variables.iter() {
                if !self
                    .agg_event_queue
                    .get(&event_type)
                    .unwrap()
                    .contains_key(&variable.key)
                {
                    self.agg_event_queue
                        .get_mut(&event_type)
                        .unwrap()
                        .insert(variable.key.clone(), HashMap::new());
                }
                for feature in config_body.features.iter() {
                    if !self
                        .agg_event_queue
                        .get(&event_type)
                        .unwrap()
                        .get(&variable.key)
                        .unwrap()
                        .contains_key(&feature.key)
                    {
                        self.agg_event_queue
                            .get_mut(&event_type)
                            .unwrap()
                            .get_mut(&variable.key)
                            .unwrap()
                            .insert(feature._id.clone(), HashMap::new());
                    }
                    for variation in feature.variations.iter() {
                        if !self
                            .agg_event_queue
                            .get(&event_type)
                            .unwrap()
                            .get(&variable.key)
                            .unwrap()
                            .get(&feature._id)
                            .unwrap()
                            .contains_key(&variation._id)
                        {
                            self.agg_event_queue
                                .get_mut(&event_type)
                                .unwrap()
                                .get_mut(&variable.key)
                                .unwrap()
                                .get_mut(&feature._id)
                                .unwrap()
                                .insert(variation._id.clone(), HashMap::new());
                        }
                        for reason in [
                            EvaluationReason::TargetingMatch,
                            EvaluationReason::Split,
                            EvaluationReason::Default,
                            EvaluationReason::Error,
                        ] {
                            if !self
                                .agg_event_queue
                                .get(&event_type)
                                .unwrap()
                                .get(&variable.key)
                                .unwrap()
                                .get(&feature._id)
                                .unwrap()
                                .get(&variation._id)
                                .unwrap()
                                .contains_key(&reason)
                            {
                                self.agg_event_queue
                                    .get_mut(&event_type)
                                    .unwrap()
                                    .get_mut(&variable.key)
                                    .unwrap()
                                    .get_mut(&feature._id)
                                    .unwrap()
                                    .get_mut(&variation._id)
                                    .unwrap()
                                    .insert(reason.clone(), 0);
                            }
                        }
                    }
                }
            }
        }
    }

    async unsafe fn process_user_events(&mut self, mut event: UserEventData) -> Result<bool, DevCycleError> {
        let client_custom_data = get_client_custom_data(self.sdk_key.clone());

        let populated_user =
            PopulatedUser::new(event.user.clone(), (*self.platform_data).clone(), client_custom_data.clone());
        let bucketed_config = generate_bucketed_config(
            self.sdk_key.clone(),
            populated_user,
            client_custom_data,
        )
        .await;
        if bucketed_config.is_err() {
            return Err(bucketed_config.err().unwrap());
        }

        event.event.feature_vars = bucketed_config?.feature_variation_map;

        if event.event.event_type == EventType::CustomEvent {
            event.event.user_id = event.user.user_id
        }

        self.queue_access_mutex.lock();

        return Ok(true);
    }
}
