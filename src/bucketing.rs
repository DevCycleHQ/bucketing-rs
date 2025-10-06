use std::arch::naked_asm;
use crate::config::*;
use crate::configmanager;
use crate::constants;
use crate::errors;
use crate::errors::{missing_config, missing_variation, DevCycleError};
use crate::feature::*;
use crate::murmurhash::murmurhash;
use crate::target::*;
use crate::target::{Rollout, RolloutStage};
use crate::user::{BucketedUserConfig, PopulatedUser, User};
use std::collections::HashMap;
use std::ops::{ControlFlow, Deref, Sub};
use std::ptr::{hash, null, null_mut};
use std::sync::Arc;

pub fn determine_user_bucketing_value_for_target(
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
        if current_date_time > rollout_start_date {
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
    if _current_stage == null_mut() && rollout_start_date < current_date_time {
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

pub(crate) unsafe fn does_user_pass_rollout(rollout: Option<Rollout>, bounded_hash: f64) -> bool {
    match rollout {
        Some(r) => {
            let current_rollout_percentage = get_current_rollout_percentage(r, chrono::Utc::now());
            return (current_rollout_percentage != 0.0 && bounded_hash <= current_rollout_percentage);
        }
        None => true, // No rollout means user passes by default
    }
}

pub(crate) unsafe fn evaluate_segmentation_for_feature(
    config: &ConfigBody,
    feature: &ConfigFeature,
    mut user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<Target, DevCycleError> {
    let merged_custom_data = user.combined_custom_data();
    let mut ret : Result<Target, DevCycleError> = Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS);
    // Use slice iteration to avoid moving out of the vector
    feature.configuration.targets.iter().try_for_each(| target | {
        let passthrough_enabled = config.project.settings.disable_passthrough_rollouts;
        let mut does_user_passthrough = true;
        if passthrough_enabled {
            let bucketing_value = determine_user_bucketing_value_for_target(
                target.bucketingkey.clone(),
                user.user_id.clone(),
                merged_custom_data.clone(),
            );
            let bounded_hash =
                murmurhash::generate_bounded_hashes(bucketing_value, target._id.clone());
            does_user_passthrough =
                does_user_pass_rollout(target.rollout.clone(), bounded_hash.rollout_hash);
        }
        let operator = &target.audience.filters;
        if does_user_passthrough
            && operator.evaluate((*config).audiences, &mut user, &client_custom_data.clone())
        {
            ret = Ok(target.clone());
            None // Break the loop
        } else {
            Some(())
        }
    });
    return ret;
}

pub(crate) unsafe fn does_user_qualify_for_feature(
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
    let target = target_pair.ok().unwrap();

    let merged_custom_data = user.combined_custom_data();
    let bucketing_value = determine_user_bucketing_value_for_target(
        target.bucketingkey.clone(),
        user.user_id.clone(),
        merged_custom_data.clone(),
    );

    let bounded_hashes =
        murmurhash::generate_bounded_hashes(bucketing_value, target._id.clone());
    let rollout_hash = bounded_hashes.rollout_hash;
    let passthrough_enabled = !(*config).project.settings.disable_passthrough_rollouts;

    if !passthrough_enabled && !does_user_pass_rollout(target.rollout.clone(), rollout_hash) {
        return Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_ROLLOUTS);
    }
    Ok(TargetAndHashes {
        target,
        bounded_hash: bounded_hashes,
    })
}
pub(crate) fn bucket_user_for_variation(
    feature: &ConfigFeature,
    hashes: TargetAndHashes,
) -> Result<Variation, DevCycleError> {

    let variation = hashes.target.decide_target_variation(hashes.bounded_hash.bucketing_hash);
    if !variation.is_ok() {
        return Err(errors::failed_to_decide_variation());
    }
    if feature.variations.len() > 0 {
        return Ok(feature.variations[0].clone());
    }
    Err(missing_variation())
}
pub async unsafe fn generate_bucketed_config(
    sdk_key: &str,
    user: PopulatedUser,
    client_custom_data: HashMap<String, serde_json::Value>,
) -> Result<BucketedUserConfig, DevCycleError> {
    let config_result =
        configmanager::CONFIGS.read().unwrap().get(sdk_key).cloned().ok_or(missing_config())?;

    let project_id = config_result.project._id.clone();
    let environment_id = config_result.environment._id.clone();
    let mut variables: HashMap<String, ReadOnlyVariable> = HashMap::new();
    let mut feature_key_map: HashMap<String, Feature> = HashMap::new();
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
        let variation = bucket_user_for_variation(feature, target_hash.ok().unwrap());
        if !variation.is_ok() {
            return Err(variation.err().unwrap());
        }
        let variation_instance = variation.ok().unwrap();
        feature_key_map.insert(
            feature.key.clone(),
            Feature {
                _id: feature._id.clone(),
                _type: feature._type.clone(),
                variation: variation_instance._id.clone(),
                variationkey: variation_instance.key.clone(),
                variationname: variation_instance.name.clone(),
                key: feature.key.clone(),
                evalreason: None,
            },
        );
        feature_variation_map.insert(feature.key.clone(), variation_instance._id.clone());

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
                },
            );
            variable_variation_map.insert(
                variable_instance.key.clone(),
                FeatureVariation {
                    feature: feature.key.clone(),
                    variation: variation_instance.key.clone(),
                },
            );
        }


    }


    Ok(BucketedUserConfig {
        user,
        project: project_id,
        environment: environment_id,
        features: feature_key_map,
        feature_variation_map,
        variable_variation_map,
        variables,
    })
}
