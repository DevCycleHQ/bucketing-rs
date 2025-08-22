pub(crate) mod murmurhash {
    use crate::constants;
    use murmur3::murmur3_32;

    pub(crate) fn generate_bounded_hash(input: String, seed: u32) -> f64 {
        let mut cursor = std::io::Cursor::new(input.as_bytes());
        let hash = murmur3_32(&mut cursor, seed).unwrap();
        return f64::from(hash) / f64::from(constants::MAX_HASH_VALUE);
    }
    pub(crate) struct BoundedHash {
        pub rollout_hash: f64,
        pub bucketing_hash: f64,
    }

    pub(crate) fn generate_bounded_hashes(
        bucketing_key_value: String,
        target_id: String,
    ) -> BoundedHash {
        let mut cursor = std::io::Cursor::new(target_id.as_bytes());
        let target_hash = murmur3_32(&mut cursor, constants::BASE_SEED).unwrap();
        let rollout_hash =
            generate_bounded_hash(bucketing_key_value.clone() + "_rollout", target_hash);
        let bucketing_hash = generate_bounded_hash(bucketing_key_value.clone(), target_hash);
        BoundedHash {
            rollout_hash,
            bucketing_hash,
        }
    }
}
