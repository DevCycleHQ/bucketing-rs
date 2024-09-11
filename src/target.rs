pub mod target {
    use crate::filters::filters::AudienceOperator;
    use crate::errors;
    pub struct Target<'a> {
        _id: String,
        _audience: &'a Audience<'a>,
        rollout: &'a Rollout,
        distribution: Vec<TargetDistribution>,
        #[serde(rename = "bucketingKey")]
        bucketingkey: String,
    }

    impl Target<'_> {
        pub fn decide_target_variation(self, bounded_hash: f64) -> Result<String, DevCycleError> {
            let mut distribution_index: f64 = 0.0;
            let mut previous_distribution_index: f64 = 0.0;

            for d in self.distribution {
                distribution_index += d.percentage;
                if bounded_hash >= previous_distribution_index && bounded_hash < distribution_index {
                    return Ok(d.variation);
                }
                previous_distribution_index = distribution_index;
            }
            return Err(FAILED_TO_DECIDE_VARIATION);
        }
    }

    pub struct Audience<'a> {
        _id: String,
        filters: &'a AudienceOperator
    }

    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use errors::FAILED_TO_DECIDE_VARIATION;
    use crate::errors::errors::DevCycleError;

    #[derive(Serialize, Deserialize)]
    struct Rollout {
        #[serde(rename = "type")]
        _type: String,
        start_percentage: f64,
        start_date: DateTime<Utc>,
        stages: Vec<RolloutStage>,
    }

    #[derive(Serialize, Deserialize)]
    struct RolloutStage {
        #[serde(rename = "type")]
        _type: String,
        date: DateTime<Utc>,
        percentage: f64,
    }

    #[derive(Serialize, Deserialize)]
    struct TargetDistribution {
        #[serde(rename = "_variation")]
        variation: String,
        percentage: f64,
    }
}