pub(crate) mod target {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use serde_json;

    use crate::errors::errors::{DevCycleError, FAILED_TO_DECIDE_VARIATION};
    use crate::filters::filters::AudienceOperator;
    use crate::murmurhash::murmurhash;

    pub(crate) struct Target {
        pub(crate) _id: String,
        pub(crate) audience: Audience,
        pub(crate) rollout: Rollout,
        pub(crate) distribution: Vec<TargetDistribution>,
        //#[serde_json::serde(rename = "bucketingKey")]
        pub(crate) bucketingkey: String,
    }

    impl Target {
        pub(crate) fn decide_target_variation(
            self,
            bounded_hash: f64,
        ) -> Result<String, DevCycleError> {
            let mut distribution_index: f64 = 0.0;
            let mut previous_distribution_index: f64 = 0.0;

            for d in self.distribution {
                distribution_index += d.percentage;
                if bounded_hash >= previous_distribution_index && bounded_hash < distribution_index
                {
                    return Ok(d.variation);
                }
                previous_distribution_index = distribution_index;
            }
            return Err(FAILED_TO_DECIDE_VARIATION);
        }
    }

    pub struct Audience {
        pub(crate) _id: String,
        pub(crate) filters: AudienceOperator,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub(crate) struct Rollout {
        #[serde(rename = "type")]
        pub(crate) _type: String,
        pub(crate) start_percentage: f64,
        pub(crate) start_date: DateTime<Utc>,
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
        #[serde(rename = "_variation")]
        pub(crate) variation: String,
        pub(crate) percentage: f64,
    }

    pub(crate) struct TargetAndHashes {
        pub(crate) target_id: String,
        pub(crate) bounded_hash: murmurhash::BoundedHash,
    }
}
