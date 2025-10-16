use crate::config::ConfigBody;
use crate::errors::DevCycleError;
use crate::events::event::*;
use crate::generate_bucketed_config;
use crate::platform_data::PlatformData;
use crate::segmentation::client_custom_data::get_client_custom_data;
use crate::user::{PopulatedUser, User};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct EventQueueOptions {
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
    pub(crate) platform_data: Arc<PlatformData>,
    pub(crate) agg_event_queue_raw_tx: mpsc::Sender<AggEventQueueRawMessage>,
    pub(crate) agg_event_queue_raw_rx: mpsc::Receiver<AggEventQueueRawMessage>,
    pub(crate) user_event_queue_raw_tx: mpsc::Sender<UserEventData>,
    pub(crate) user_event_queue_raw_rx: mpsc::Receiver<UserEventData>,
    pub(crate) agg_event_queue: AggregateEventQueue,
    pub(crate) user_event_queue: UserEventQueue,
    pub(crate) user_event_queue_count: i32,
    pub(crate) queue_access_mutex: tokio::sync::Mutex<()>,
    pub(crate) events_flushed: i64,
    pub(crate) events_dropped: i64,
    pub(crate) events_reported: i64,
    pub(crate) options: EventQueueOptions,
}

impl EventQueue {
    pub fn new(
        sdk_key: String,
        event_queue_options: EventQueueOptions,
    ) -> Result<Self, DevCycleError> {
        let (agg_event_queue_raw_tx, agg_event_queue_raw_rx) = mpsc::channel(10000);
        let (user_event_queue_raw_tx, user_event_queue_raw_rx) = mpsc::channel(10000);
        let platform_data = crate::platform_data::get_platform_data(&sdk_key)
            .map_err(|e| DevCycleError::new(&e))?;
        Ok(Self {
            sdk_key,
            platform_data,
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
            options: event_queue_options,
        })
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
            .try_send(AggEventQueueRawMessage {
                event_type,
                variation_id: variation_id.to_string(),
                feature_id: feature_id.to_string(),
                variable_key: variable_key.to_string(),
                eval_metadata: eval,
            });

        if success.is_err() {
            self.events_dropped += 1;
            return Err(DevCycleError::new(&format!(
                "dropping event, queue is full: {}",
                success.unwrap_err()
            )));
        }
        return Ok(true);
    }

    pub async fn queue_event(&mut self, user: User, event: Event) -> Result<bool, DevCycleError> {
        let success = self
            .user_event_queue_raw_tx
            .try_send(UserEventData { user, event });

        if success.is_err() {
            self.events_dropped += 1;
            return Err(DevCycleError::new(&format!(
                "dropping event, queue is full: {}",
                success.unwrap_err()
            )));
        }
        return Ok(true);
    }

    pub(crate) async fn merge_agg_event_queue_keys(&mut self, config_body: &ConfigBody<'_>) {
        let _guard = self.queue_access_mutex.lock().await;
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
                            EvaluationReason::Disabled,
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

    async unsafe fn process_user_events(
        &mut self,
        mut event: UserEventData,
    ) -> Result<bool, DevCycleError> {
        let client_custom_data = get_client_custom_data(self.sdk_key.clone());

        let populated_user = PopulatedUser::new(
            event.user.clone(),
            self.platform_data.clone(),
            client_custom_data.clone(),
        );
        let bucketed_config =
            generate_bucketed_config(&self.sdk_key, populated_user.clone(), client_custom_data)
                .await;
        if bucketed_config.is_err() {
            return Err(bucketed_config.err().unwrap());
        }

        event.event.feature_vars = bucketed_config?.feature_variation_map;

        if event.event.event_type == EventType::CustomEvent {
            event.event.user_id = event.user.user_id.clone();
        }

        let _guard = self.queue_access_mutex.lock().await;

        // Add event to user event queue
        let user_id = event.user.user_id.clone();
        self.user_event_queue
            .entry(user_id)
            .or_insert_with(|| UserEventsBatchRecord {
                user: populated_user,
                events: Vec::new(),
            })
            .events
            .push(event.event);

        self.user_event_queue_count += 1;

        return Ok(true);
    }

    pub(crate) async fn process_aggregate_event(
        &mut self,
        agg_event_queue_raw_message: AggEventQueueRawMessage,
    ) {
        let _guard = self.queue_access_mutex.lock().await;

        let event_type = agg_event_queue_raw_message.event_type.clone();
        let variable_key = agg_event_queue_raw_message.variable_key;
        let feature_id = agg_event_queue_raw_message.feature_id;
        let variation_id = agg_event_queue_raw_message.variation_id;
        let eval_metadata = agg_event_queue_raw_message.eval_metadata;

        if event_type == EventType::AggregateVariableEvaluated {
            // Get or create the nested structure and update counts directly
            let eval_reasons = self
                .agg_event_queue
                .entry(event_type)
                .or_insert_with(HashMap::new)
                .entry(variable_key)
                .or_insert_with(HashMap::new)
                .entry(feature_id)
                .or_insert_with(HashMap::new)
                .entry(variation_id)
                .or_insert_with(HashMap::new);

            for (reason, count) in eval_metadata {
                *eval_reasons.entry(reason).or_insert(0) += count;
            }
        } else {
            // For defaulted events, use "default" as both feature_id and variation_id keys
            let default_key = "default".to_string();

            let default_reasons = self
                .agg_event_queue
                .entry(event_type)
                .or_insert_with(HashMap::new)
                .entry(variable_key)
                .or_insert_with(HashMap::new)
                .entry(default_key.clone())
                .or_insert_with(HashMap::new)
                .entry(default_key)
                .or_insert_with(HashMap::new);

            for (reason, count) in eval_metadata {
                *default_reasons.entry(reason).or_insert(0) += count;
            }
        }
    }

    pub(crate) async fn process_events(
        &mut self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    // Context is cancelled, exit the loop
                    return;
                }
                user_event = self.user_event_queue_raw_rx.recv() => {
                    match user_event {
                        Some(event) => unsafe{
                            let _ = self.process_user_events(event).await;
                        }
                        None => {
                            // Channel closed
                            return;
                        }
                    }
                }
                agg_event = self.agg_event_queue_raw_rx.recv() => {
                    match agg_event {
                        Some(event) => {
                            self.process_aggregate_event(event).await;
                        }
                        None => {
                            // Channel closed
                            return;
                        }
                    }
                }
            }
        }
    }
}
