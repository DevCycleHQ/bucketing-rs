pub mod bucketing {
    use std::collections::HashMap;
    use std::ops::Sub;
    use std::ptr::{null, null_mut};
    use crate::{constants, murmurhash, target, user};
    use crate::config::config::{ConfigBody, Variable};
    use crate::configmanager::config_manager;
    use crate::errors::errors;
    use crate::errors::errors::DevCycleError;
    use crate::feature::feature::{ConfigFeature, Feature, FeatureVariation, ReadOnlyVariable, Variation};
    use crate::target::target::{Rollout, RolloutStage, Target, TargetAndHashes};
    use crate::user::user::{BucketedUserConfig, PopulatedUser};


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
    pub fn bucket_user_for_variation(feature: ConfigFeature, hashes: TargetAndHashes ) -> Result<Variation, DevCycleError> {
        let mut variation = hashes.target.decide_target_variation(hashes.bounded_hash.bucketing_hash);
        match variation {
            Err(e) => return Err(e),
            Ok(v) => for _v in feature.variations {
                if _v._id == v {
                    return Ok(_v);
                }
            },

        }
        Err(errors::MISSING_VARIATION)
    }

    pub async unsafe fn generate_bucketed_config(
        sdk_key: &str,
        user: PopulatedUser,
        client_custom_data: HashMap<String, serde_json::Value>,
    ) -> Result<user::user::BucketedUserConfig, DevCycleError> {
        let config: ConfigBody = config_manager::CONFIGS.lock().unwrap().get(sdk_key).await?;
        let mut variable_map: HashMap<String, ReadOnlyVariable> = HashMap::new();
        let mut feature_key_map: HashMap<String, Feature> = HashMap::new();
        let mut feature_variation_map: HashMap<String, String> = HashMap::new();
        let mut variable_variation_map: HashMap<String, FeatureVariation> = HashMap::new();

        for feature in config.features {
            let thash = does_user_qualify_for_feature(&config, &feature, user.clone(), client_custom_data.clone()).await?;

            let variation: Variation = bucket_user_for_variation(feature, thash).await?;

            feature_key_map.insert(feature.key.clone(), Feature {
                _id: feature._id.clone(),
                _type: feature.featuretype.clone(),
                key: feature.key.clone(),
                variation: variation._id.clone(),
                variationkey: variation.key.clone(),
                variationname: variation.name.clone(),
                evalreason: "".to_string(),
            });
            feature_variation_map.insert(feature._id.clone(), variation._id.clone());

            for variation_var in variation.variables {
                let variable: Variable = config.variable_id_map.get(&variation_var._var).await.ok_or(errors::MISSING_VARIABLE)?;

                variable_variation_map.insert(variable.key.clone(), FeatureVariation {
                    variation: variation._id.clone(),
                    feature: feature._id.clone(),
                });
                let new_var = ReadOnlyVariable {

                    id: variable._id.clone(),
                    key: variable.key.clone(),
                    _type: variable._type,
                    value: variation_var.value,
                };
                variable_map.insert(variable.key.clone(), new_var);
            }
        }

        Ok(BucketedUserConfig {
            user: user,
            project: config.project,
            environment: config.environment,
            features: feature_key_map,
            known_variable_keys: config.variables.iter().map(|v| v.key.clone()).collect(),
            feature_variation_map,
            variable_variation_map,
            variables: variable_map,
        })
    }


}
