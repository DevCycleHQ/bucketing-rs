pub mod murmurhash {
    use fasthash::{murmur3};
    use crate::constants;
    pub fn generate_bounded_hash(input: &str, seed: u32) -> f64 {
        let hash = murmur3::hash32_with_seed(input, seed);
        return f64::from(hash) / f64::from(constants::MAX_HASH_VALUE)
    }
}