#[cfg(test)]
mod tests {
    use crate::murmurhash::murmurhash::murmur_hash;

    #[tokio::test]
    async fn test_hash_expected_result() {
        let input = "test".to_string();
        let seed = crate::constants::BASE_SEED;
        let hash = murmur_hash(input, seed);
        assert_eq!(hash, 0x99c02ae2)
    }
}