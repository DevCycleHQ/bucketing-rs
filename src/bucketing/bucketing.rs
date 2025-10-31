use crate::config::*;
use crate::configmanager;
use crate::constants;
use crate::errors;
use crate::errors::bucket_result_error_to_default_reason;
use crate::errors::{missing_config, missing_variation, DevCycleError};
use crate::events::event::{EvalDetails, EvaluationReason};
use crate::murmurhash::murmurhash;
use crate::target::*;
use crate::target::{Rollout, RolloutStage};
use crate::user::{BucketedUserConfig, PopulatedUser};
use std::collections::HashMap;
use std::ops::Sub;
use std::ptr::null_mut;
use std::sync::Arc;

// Helper function to validate variable types
fn is_variable_type_valid(actual_type: &str, expected_type: &str) -> bool {
    // First check if the variable type is one of the valid types
    if actual_type != constants::VARIABLE_TYPES_STRING
        && actual_type != constants::VARIABLE_TYPES_NUMBER
        && actual_type != constants::VARIABLE_TYPES_JSON
        && actual_type != constants::VARIABLE_TYPES_BOOL
    {
        return false;
    }
    // Then check if it matches the expected type
    if actual_type != expected_type {
        return false;
    }
    true
}

// Helper function to generate bucketed variable for user
async fn generate_bucketed_variable_for_user(
    sdk_key: &str,
    user: PopulatedUser,
    variable_key: &str,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<
    (String, serde_json::Value, String, String, EvaluationReason),
    (DevCycleError, EvaluationReason),
> {
    // Get config (already returns Arc<ConfigBody> from the CONFIGS map)
    let config = configmanager::get_config(sdk_key);

    if config.is_none() {
        eprintln!("Variable called before client initialized, returning default value");
        return Err((errors::missing_config(), EvaluationReason::Error));
    }

    let config = config.unwrap();
    // Get variable by key
    let variable = config.get_variable_for_key(variable_key);
    if variable.is_none() {
        return Err((errors::missing_variable(), EvaluationReason::Disabled));
    }
    let variable = variable.unwrap();

    // Get feature for variable
    let feat_for_variable = config.get_feature_for_variable_id(&variable._id);
    if feat_for_variable.is_none() {
        return Err((errors::missing_feature(), EvaluationReason::Disabled));
    }
    let feat_for_variable = feat_for_variable.unwrap();

    // Check if user qualifies for feature
    let target_and_hashes = match does_user_qualify_for_feature(
        &config,
        feat_for_variable,
        user.clone(),
        client_custom_data,
    ) {
        Ok(th) => th,
        Err(e) => return Err((e, EvaluationReason::Default)),
    };

    // Bucket user for variation
    let (variation, is_random_distrib) =
        match bucket_user_for_variation(feat_for_variable, target_and_hashes.clone()) {
            Ok(v) => v,
            Err(e) => return Err((e, EvaluationReason::Default)),
        };

    // Get variable from variation
    let variation_variable = variation.get_variable_by_id(&variable._id);
    if variation_variable.is_none() {
        return Err((
            errors::missing_variable_for_variation(),
            EvaluationReason::Disabled,
        ));
    }
    let variation_variable = variation_variable.unwrap();

    // Determine evaluation reason
    let eval_reason = if target_and_hashes.is_rollout || is_random_distrib {
        EvaluationReason::Split
    } else {
        EvaluationReason::TargetingMatch
    };

    Ok((
        variable._type.clone(),
        variation_variable.value.clone(),
        feat_for_variable._id.clone(),
        variation._id.clone(),
        eval_reason,
    ))
}

pub struct VariableForUserResult {
    pub variable_type: String,
    pub variable_value: serde_json::Value,
    pub feature_id: String,
    pub variation_id: String,
    pub eval_reason: Result<EvaluationReason, DevCycleError>,
}

pub async fn variable_for_user(
    sdk_key: &str,
    user: PopulatedUser,
    variable_key: &str,
    expected_variable_type: &str,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<VariableForUserResult, DevCycleError> {
    let result =
        generate_bucketed_variable_for_user(sdk_key, user, variable_key, client_custom_data).await;
    let event_queue = match crate::events::event_queue_manager::get_event_queue(sdk_key) {
        Some(eq) => eq,
        None => {
            eprintln!("Event queue not initialized for SDK key: {}", sdk_key);
            return Err(errors::event_queue_not_initialized());
        }
    };
    match result {
        Ok((variable_type, variable_value, feature_id, variation_id, eval_reason)) => {
            // Validate variable type
            if !is_variable_type_valid(&variable_type, expected_variable_type)
                && !expected_variable_type.is_empty()
            {
                let err = errors::invalid_variable_type();

                if let Err(event_err) = event_queue
                    .queue_variable_defaulted_event(variable_key, "", "")
                    .await
                {
                    eprintln!("Failed to queue variable defaulted event: {}", event_err);
                }

                return Err(err);
            }

            // Queue variable evaluated event
            if let Err(event_err) = event_queue
                .queue_variable_evaluated_event(
                    variable_key,
                    &feature_id,
                    &variation_id,
                    eval_reason.clone(),
                )
                .await
            {
                eprintln!("Failed to queue variable evaluated event: {}", event_err);
            }

            Ok(VariableForUserResult {
                variable_type,
                variation_id,
                variable_value,
                feature_id,
                eval_reason: Ok(eval_reason),
            })
        }
        Err((err, eval_reason)) => {
            let default_reason = bucket_result_error_to_default_reason(&err);

            if let Err(event_err) = event_queue
                .queue_variable_defaulted_event(variable_key, "", "")
                .await
            {
                eprintln!("Failed to queue variable defaulted event: {}", event_err);
            }

            // Return empty values with the evaluation reason from the error
            Ok(VariableForUserResult {
                variable_type: String::new(),
                variable_value: serde_json::Value::Null,
                variation_id: String::new(),
                eval_reason: Err(err),
                feature_id: String::new(),
            })
        }
    }
}

pub(crate) fn determine_user_bucketing_value_for_target(
    target_bucketing_key: String,
    user_id: String,
    merged_custom_data: HashMap<String, serde_json::Value>,
) -> String {
    if target_bucketing_key == "" || target_bucketing_key == "user_id" {
        return user_id;
    }
    if merged_custom_data.contains_key(&target_bucketing_key) {
        if merged_custom_data
            .get(&target_bucketing_key)
            .unwrap()
            .is_null()
        {
            return String::from(constants::DEFAULT_BUCKETING_VALUE);
        }
        return match merged_custom_data.get(&target_bucketing_key) {
            Some(value) => value.to_string(),
            None => String::from(constants::DEFAULT_BUCKETING_VALUE),
        };
    }
    String::from(constants::DEFAULT_BUCKETING_VALUE)
}

pub(crate) fn get_current_rollout_percentage(
    rollout: Rollout,
    current_date: chrono::DateTime<chrono::Utc>,
) -> f64 {
    let start = rollout.start_percentage;
    let rollout_start_date = rollout.start_date;
    let current_date_time = current_date;

    if rollout._type == constants::ROLLOUT_TYPE_SCHEDULE {
        if current_date_time.gt(&rollout_start_date) {
            return 1.0;
        }
        return 0.0;
    }

    let stages = rollout.stages;
    let mut current_stages: Vec<RolloutStage> = Vec::new();
    let mut next_stages: Vec<RolloutStage> = Vec::new();
    for stage in stages {
        if current_date_time > stage.date {
            current_stages.push(stage);
        } else {
            next_stages.push(stage);
        }
    }

    // Note: This function contains unsafe pointer operations that should be refactored
    // for better Rust idioms, but keeping the logic intact for now
    let _current_stage: *mut RolloutStage;
    let next_stage: *mut RolloutStage;
    if current_stages.len() == 0 {
        _current_stage = null_mut();
    } else {
        let current_stages_len = current_stages.len();
        _current_stage = &mut current_stages[current_stages_len - 1];
    }

    if next_stages.len() == 0 {
        next_stage = null_mut();
    } else {
        next_stage = &mut next_stages[0];
    }

    let mut current_stage = _current_stage;
    if _current_stage == null_mut() && rollout_start_date.lt(&current_date_time) {
        current_stage = &mut RolloutStage {
            _type: constants::ROLLOUT_TYPE_DISCRETE.parse().unwrap(),
            date: rollout_start_date,
            percentage: start,
        }
    }
    if current_stage == null_mut() {
        return 0.0;
    }
    unsafe {
        if next_stage == null_mut() || (*next_stage)._type == constants::ROLLOUT_TYPE_DISCRETE {
            return (*current_stage).percentage;
        }

        let current_date_percentage = (current_date_time
            .sub((*current_stage).date)
            .num_milliseconds()
            / (*next_stage)
                .date
                .sub((*current_stage).date)
                .num_milliseconds()) as f64;
        if current_date_percentage == 0.0 {
            return 0.0;
        }
        return (*current_stage).percentage
            + (current_date_percentage * ((*next_stage).percentage - (*current_stage).percentage));
    }
}

pub(crate) fn does_user_pass_rollout(rollout: Option<Rollout>, bounded_hash: f64) -> bool {
    match rollout {
        Some(r) => {
            let current_rollout_percentage = get_current_rollout_percentage(r, chrono::Utc::now());
            return current_rollout_percentage != 0.0 && bounded_hash <= current_rollout_percentage;
        }
        None => true, // No rollout means user passes by default
    }
}

pub(crate) fn evaluate_segmentation_for_feature(
    config: &ConfigBody,
    feature: &ConfigFeature,
    mut user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<(Target, bool), DevCycleError> {
    let merged_custom_data = user.combined_custom_data();
    let mut ret: Result<(Target, bool), DevCycleError> =
        Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS);
    for target in feature.configuration.targets.clone() {
        let passthrough_enabled = !config.project.settings.disable_passthrough_rollouts;
        let mut rollout_criteria_met = true;
        let mut is_rollout = false;
        if target.rollout.is_some() && passthrough_enabled {
            let bucketing_value = determine_user_bucketing_value_for_target(
                target.bucketingkey.clone(),
                user.user_id.clone(),
                merged_custom_data.clone(),
            );
            let bounded_hash =
                murmurhash::generate_bounded_hashes(bucketing_value, target._id.clone());
            rollout_criteria_met =
                is_user_in_rollout(target.rollout.clone().unwrap(), bounded_hash.rollout_hash);
            is_rollout = rollout_criteria_met;
        }
        let operator = &target.audience.filters;
        if rollout_criteria_met
            && operator.evaluate(&config.audiences, &mut user, &client_custom_data.clone())
        {
            ret = Ok((target.clone(), is_rollout.clone()));
            return ret;
        }
    }
    return ret;
}

pub(crate) fn is_user_in_rollout(rollout: Rollout, bounded_hash: f64) -> bool {
    let rollout_percentage = get_current_rollout_percentage(rollout, chrono::Utc::now());
    return rollout_percentage != 0.0 && (bounded_hash <= rollout_percentage);
}

pub(crate) fn does_user_qualify_for_feature(
    config: &Arc<ConfigBody>,
    feature: &ConfigFeature,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<TargetAndHashes, DevCycleError> {
    let target_pair =
        evaluate_segmentation_for_feature(config, feature, user.clone(), client_custom_data);
    if !target_pair.is_ok() {
        return Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS);
    }
    let (target, is_rollout) = target_pair.ok().unwrap();
    let merged_custom_data = user.combined_custom_data();
    let bucketing_value = determine_user_bucketing_value_for_target(
        target.bucketingkey.clone(),
        user.user_id.clone(),
        merged_custom_data.clone(),
    );

    let bounded_hashes = murmurhash::generate_bounded_hashes(bucketing_value, target._id.clone());
    let rollout_hash = bounded_hashes.rollout_hash;
    let passthrough_enabled = !config.project.settings.disable_passthrough_rollouts;

    if !passthrough_enabled && !does_user_pass_rollout(target.rollout.clone(), rollout_hash) {
        return Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_ROLLOUTS);
    }
    Ok(TargetAndHashes {
        target,
        bounded_hash: bounded_hashes,
        is_rollout,
    })
}
pub(crate) fn bucket_user_for_variation(
    feature: &ConfigFeature,
    hashes: TargetAndHashes,
) -> Result<(Variation, bool), DevCycleError> {
    let target_variation_result = hashes
        .target
        .decide_target_variation(hashes.bounded_hash.bucketing_hash);
    if !target_variation_result.is_ok() {
        return Err(errors::failed_to_decide_variation());
    }
    let (target_variation, is_random_distrib) = target_variation_result.ok().unwrap();
    for variation in &feature.variations {
        if variation._id == target_variation {
            return Ok((variation.clone(), is_random_distrib));
        }
    }
    Err(missing_variation())
}
pub async fn generate_bucketed_config(
    sdk_key: String,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    let config_result = configmanager::CONFIGS
        .read()
        .unwrap()
        .get(&sdk_key)
        .cloned()
        .ok_or(missing_config())?;

    let project = config_result.project.clone();
    let environment = config_result.environment.clone();
    let mut variables: HashMap<String, ReadOnlyVariable> = HashMap::new();
    let mut features: HashMap<String, Feature> = HashMap::new();
    let mut feature_variation_map: HashMap<String, String> = HashMap::new();
    let mut variable_variation_map: HashMap<String, FeatureVariation> = HashMap::new();

    for feature in &config_result.features {
        let target_hash = does_user_qualify_for_feature(
            &config_result,
            feature,
            user.clone(),
            client_custom_data.clone(),
        );
        if !target_hash.is_ok() {
            continue;
        }

        let target_and_hashes = target_hash?;
        let variation = bucket_user_for_variation(feature, target_and_hashes.clone());
        if !variation.is_ok() {
            return Err(variation.err().unwrap());
        }

        let (variation_instance, is_random_distrib) = variation.ok().unwrap();
        let eval_reason = if target_and_hashes.is_rollout || is_random_distrib {
            EvaluationReason::Split
        } else {
            EvaluationReason::TargetingMatch
        };

        features.insert(
            feature.key.clone(),
            Feature {
                _id: feature._id.clone(),
                _type: feature._type.clone(),
                variation: variation_instance._id.clone(),
                variationkey: variation_instance.key.clone(),
                variationname: variation_instance.name.clone(),
                key: feature.key.clone(),
                evalreason: Some(eval_reason.clone()),
            },
        );
        feature_variation_map.insert(feature._id.clone(), variation_instance._id.clone());

        for var in &variation_instance.variables {
            let variable = config_result.get_variable_for_id(&var._var);
            if variable.is_none() {
                continue;
            }
            let variable_instance = variable.unwrap();
            variables.insert(
                variable_instance.key.clone(),
                ReadOnlyVariable {
                    id: variable_instance._id.clone(),
                    key: variable_instance.key.clone(),
                    _type: variable_instance._type.clone(),
                    value: var.value.clone(),
                    eval: EvalDetails {
                        reason: eval_reason.clone(),
                        details: None,
                        target_id: Some(target_and_hashes.target._id.clone()),
                    },
                },
            );
            variable_variation_map.insert(
                variable_instance.key.clone(),
                FeatureVariation {
                    feature: feature._id.clone(),
                    variation: variation_instance._id.clone(),
                },
            );
        }
    }

    Ok(BucketedUserConfig {
        user,
        project,
        environment,
        features,
        feature_variation_map,
        variable_variation_map,
        variables,
    })
}
