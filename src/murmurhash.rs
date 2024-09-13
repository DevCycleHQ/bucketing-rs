pub(crate) mod murmurhash {
    use fasthash::{murmur3};
    use crate::constants;
    pub(crate) fn generate_bounded_hash(input: String, seed: u32) -> f64 {
        let hash = murmur3::hash32_with_seed(input, seed);
        return f64::from(hash) / f64::from(constants::MAX_HASH_VALUE)
    }
    pub(crate) struct BoundedHash {
        pub rollout_hash: f64,
        pub bucketing_hash: f64,
    }

    pub(crate) fn generate_bounded_hashes(bucketing_key_value: String, target_id: String) -> BoundedHash {
        let target_hash = murmur3::hash32_with_seed(target_id, constants::BASE_SEED);
        let rollout_hash = generate_bounded_hash(bucketing_key_value.clone()+"_rollout", target_hash);
        let bucketing_hash = generate_bounded_hash(bucketing_key_value.clone(), target_hash);
        BoundedHash {
            rollout_hash,
            bucketing_hash,
        }
    }
}