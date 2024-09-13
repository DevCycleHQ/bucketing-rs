pub mod bucketing {
    use std::collections::HashMap;
    use std::ops::Sub;
    use std::ptr::{null, null_mut};
    use crate::{constants, murmurhash, target};
    use crate::config::config::ConfigBody;
    use crate::errors::errors;
    use crate::errors::errors::DevCycleError;
    use crate::feature::feature::{ConfigFeature, Variation};
    use crate::target::target::{Rollout, RolloutStage, Target, TargetAndHashes};
    use crate::user::user::PopulatedUser;


    pub fn determine_user_bucketing_value_for_target(target_bucketing_key: String, user_id: String, merged_custom_data: HashMap<String, serde_json::Value>) -> String {
        if target_bucketing_key == "" || target_bucketing_key == "user_id" {
            return user_id;
        }
        if merged_custom_data.contains_key(&target_bucketing_key) {
            if merged_custom_data.get(&target_bucketing_key).unwrap().is_null() {
                return String::from(constants::DEFAULT_BUCKETING_VALUE);
            }
            return match merged_custom_data.get(&target_bucketing_key) {
                Some(value) => {
                    value.to_string()
                }
                None => {
                    String::from(constants::DEFAULT_BUCKETING_VALUE)
                }
            };
        }
        return String::from(constants::DEFAULT_BUCKETING_VALUE);
    }

    pub fn get_current_rollout_percentage(rollout: Rollout, current_date: chrono::DateTime<chrono::Utc>) -> f64 {
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

        let _current_stage: *mut target::target::RolloutStage;
        let next_stage: *mut target::target::RolloutStage;
        if current_stages.len() == 0 {
            _current_stage = null_mut();
        } else {
            _current_stage = &mut current_stages[current_stages.len() - 1];
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

            let current_date_percentage = (current_date_time.sub((*current_stage).date).num_milliseconds() / (*next_stage).date.sub((*current_stage).date).num_milliseconds()) as f64;
            if current_date_percentage == 0.0 {
                return 0.0;
            }
            return (*current_stage).percentage + (current_date_percentage * ((*next_stage).percentage - (*current_stage).percentage));
        }
    }

    pub unsafe fn does_user_pass_rollout(rollout: Rollout, bounded_hash: f64) -> bool {
        let current_rollout_percentage = get_current_rollout_percentage(rollout, chrono::Utc::now());
        return current_rollout_percentage != 0.0 && bounded_hash < current_rollout_percentage;
    }

    pub unsafe fn evaluate_segmentation_for_feature(config: *const ConfigBody, feature: *const ConfigFeature, mut user: PopulatedUser, client_custom_data: HashMap<String, serde_json::Value>) -> *const Target {
        let merged_custom_data = user.combined_custom_data();
        for target in (*feature).configuration.targets {
            let passthrough_enabled = !(*config).project.settings.disable_passthrough_rollouts;
            let mut does_user_passthrough = true;
            if passthrough_enabled {
                let bucketing_value = determine_user_bucketing_value_for_target(target.bucketingkey, user.user_id.clone(), merged_custom_data.clone());
                let bounded_hash = murmurhash::murmurhash::generate_bounded_hashes(bucketing_value, target._id.clone());
                does_user_passthrough = does_user_pass_rollout(target.rollout, bounded_hash.rollout_hash);
            }
            let operator = target.audience.filters;
            if does_user_passthrough && operator.evaluate((*config).audiences, &mut user, &client_custom_data.clone()) {
                return &target;
            }
        }
        null()
    }

    pub unsafe fn does_user_qualify_for_feature(config: *const ConfigBody, feature: *const ConfigFeature, user: PopulatedUser, client_custom_data: HashMap<String, serde_json::Value>) -> Result<target::target::TargetAndHashes, DevCycleError> {
        let target =  evaluate_segmentation_for_feature(config, feature, user.clone(), client_custom_data);
        if target == null() {
            return Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_TARGETS);
        }

        let merged_custom_data = user.combined_custom_data();
        let bucketing_value = determine_user_bucketing_value_for_target((*target).clone().bucketingkey, user.user_id.clone(), merged_custom_data.clone());

        let bounded_hashes = murmurhash::murmurhash::generate_bounded_hashes(bucketing_value, (*target).clone()._id);
        let rollout_hash = bounded_hashes.rollout_hash;
        let passthrough_enabled = !(*config).project.settings.disable_passthrough_rollouts;

        if !passthrough_enabled && !does_user_pass_rollout((*target).clone().rollout, rollout_hash) {
            return Err(errors::FAILED_USER_DOES_NOT_QUALIFY_FOR_ROLLOUTS);
        }
        return Ok(target::target::TargetAndHashes {
            target: (*target).clone(),
            bounded_hash: bounded_hashes,
        });
    }
    pub unsafe fn bucket_user_for_variation(feature: ConfigFeature, hashes: TargetAndHashes ) -> Result<Variation, DevCycleError> {
        let variation = hashes.target.decide_target_variation(hashes.bounded_hash.bucketing_hash);
        if variation.is_err() {
            return Err(variation.unwrap_err());
        }
        let v = variation.cl
        for _v in feature.variations {
            if _v._id == variation {
                return Ok(variation);
            }
        }
        let target = hashes.target;
        let bounded_hash = hashes.bounded_hash;
        let variation = target.decide_target_variation(bounded_hash.variation_hash);
        return Ok(variation);
    }
}
