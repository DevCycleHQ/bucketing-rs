use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{DevCycleError, FAILED_TO_DECIDE_VARIATION};
use crate::filters::{AudienceOperator, NoIdAudience};
use crate::murmurhash::murmurhash;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Target {
    pub(crate) _id: String,
    #[serde(rename = "_audience")]
    pub(crate) audience: NoIdAudience,
    #[serde(default)]
    pub(crate) rollout: Option<Rollout>,
    pub(crate) distribution: Vec<TargetDistribution>,
    #[serde(rename = "bucketingKey", default = "default_bucketing_key")]
    pub(crate) bucketingkey: String,
}

impl Target {
    pub(crate) fn decide_target_variation(
        self,
        bounded_hash: f64,
    ) -> Result<(String, bool), DevCycleError> {
        let mut distribution_index: f64 = 0.0;
        let previous_distribution_index: f64 = 0.0;
        let is_randomized = self.distribution.len() > 1;
        for d in self.distribution {
            distribution_index += d.percentage;
            if bounded_hash >= previous_distribution_index
                && (bounded_hash < distribution_index
                    || (distribution_index == 1.0 && bounded_hash == 1.0))
            {
                return Ok((d.variation, is_randomized));
            }
        }
        return Err(FAILED_TO_DECIDE_VARIATION);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Audience {
    pub(crate) _id: String,
    pub(crate) filters: AudienceOperator,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Rollout {
    #[serde(rename = "type")]
    pub(crate) _type: String,
    #[serde(default)]
    pub(crate) start_percentage: f64,
    #[serde(default)]
    pub(crate) start_date: DateTime<Utc>,
    #[serde(default)]
    pub(crate) stages: Vec<RolloutStage>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RolloutStage {
    #[serde(rename = "type")]
    pub(crate) _type: String,
    pub(crate) date: DateTime<Utc>,
    pub(crate) percentage: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TargetDistribution {
    #[serde(rename = "_variation", alias = "_variation")]
    pub(crate) variation: String,
    pub(crate) percentage: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TargetAndHashes {
    pub(crate) target: Target,
    pub(crate) bounded_hash: murmurhash::BoundedHash,
    pub(crate) is_rollout: bool,
}

fn default_bucketing_key() -> String {
    "user_id".to_string()
}
