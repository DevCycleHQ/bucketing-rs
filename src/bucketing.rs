pub mod bucketing {
    use std::collections::HashMap;
    use std::ops::Sub;
    use std::ptr::{null, null_mut};
    use crate::{constants, murmurhash, target};
    use crate::target::target::{Rollout, RolloutStage};

    const default_bucketing_value: String = String::from("null");
    struct BoundedHash {
        rollout_hash: f64,
        bucketing_hash: f64,
    }

    pub fn generate_bounded_hash(input: &str, seed: u32) -> BoundedHash {
        let rollout_hash = murmurhash::murmurhash::generate_bounded_hash(input, seed);
        let bucketing_hash = murmurhash::murmurhash::generate_bounded_hash(input, seed + 1);
        BoundedHash {
            rollout_hash,
            bucketing_hash,
        }
    }

    pub fn determine_user_bucketing_value_for_target(target_bucketing_key: String, user_id: String, merged_custom_data: &mut HashMap<String, serde_json::Value>) -> String {
        if target_bucketing_key == "" || target_bucketing_key == "user_id" {
            return user_id;
        }
        if merged_custom_data.contains_key(&target_bucketing_key) {
            if merged_custom_data.get(&target_bucketing_key).unwrap().is_null(){
                return default_bucketing_value;
            }
            return match merged_custom_data.get(&target_bucketing_key) {
                Some(value) => {
                    value.to_string()
                }
                None => {
                    default_bucketing_value
                }
            }
        }
        return default_bucketing_value;
    }

    pub fn get_current_rollout_percentage(rollout: target::target::Rollout, current_date: chrono::DateTime<chrono::Utc>) -> f64 {
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
        let mut current_stages: Vec<target::target::RolloutStage> = Vec::new();
        let mut next_stages: Vec<target::target::RolloutStage> = Vec::new();
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
        if _current_stage == null_mut() && rollout_start_date < current_date_time{
            current_stage = &mut RolloutStage{
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

            let current_date_percentage = f64::from(current_date_time.sub((*current_stage).date).num_milliseconds()) / f64::from((*next_stage).date.sub((*current_stage).date).num_milliseconds());
            if current_date_percentage == 0.0 {
                return 0.0;
            }
            return (*current_stage).percentage + (current_date_percentage * ((*next_stage).percentage - (*current_stage).percentage));
        }
    }

    pub fn does_user_pass_rollout(rollout: Rollout, bounded_hash: f64) -> bool {
        let current_rollout_percentage = get_current_rollout_percentage(rollout, chrono::Utc::now());
        return current_rollout_percentage != 0.0 && bounded_hash < current_rollout_percentage;
    }

    pub fn evaluate_segmentation_for_feature()
}
