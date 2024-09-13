pub mod murmurhash;
pub mod constants;
pub mod versioncompare;
pub mod filters;
pub mod feature;
pub mod user;
pub mod platform_data;
pub mod target;
pub mod errors;
mod config;
mod bucketing;
mod configmanager;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
