use regex::Regex;
use std::f64;
use std::str::FromStr;

#[derive(Debug)]
struct SemverOptions {
    lexicographical: bool,
    zero_extend: bool,
}

trait Compare {
    fn version_compare(&self, v1: &str, v2: &str) -> f64;
}

impl Compare for SemverOptions {
    fn version_compare(&self, v1: &str, v2: &str) -> f64 {
        let lexicographical = self.lexicographical;
        let zero_extend = self.zero_extend;
        let mut v1parts: Vec<String> = v1.split('.').map(String::from).collect();
        let mut v2parts: Vec<String> = v2.split('.').map(String::from).collect();
        let has_v1 = has_valid_parts(lexicographical, &v1parts);
        let has_v2 = has_valid_parts(lexicographical, &v2parts);

        if !has_v1 || !has_v2 {
            return f64::NAN;
        }

        if zero_extend {
            while v1parts.len() < v2parts.len() {
                v1parts.push("0".to_string());
            }
            while v2parts.len() < v1parts.len() {
                v2parts.push("0".to_string());
            }
        }

        let mut v1_parts_final: Vec<f64> = Vec::new();
        let mut v2_parts_final: Vec<f64> = Vec::new();

        if !lexicographical {
            for v1part in v1parts {
                match f64::from_str(&v1part) {
                    Ok(v1_float) => v1_parts_final.push(v1_float),
                    Err(_) => v1_parts_final.push(f64::NAN),
                }
            }
            for v2part in v2parts {
                match f64::from_str(&v2part) {
                    Ok(v2_float) => v2_parts_final.push(v2_float),
                    Err(_) => v2_parts_final.push(f64::NAN),
                }
            }
        }

        for (i, v1p) in v1_parts_final.iter().enumerate() {
            if v2_parts_final.len() == i {
                return 1.0;
            }
            if v1p == &v2_parts_final[i] {
                continue;
            } else if v1p > &v2_parts_final[i] {
                return 1.0;
            } else {
                return -1.0;
            }
        }

        if v1_parts_final.len() != v2_parts_final.len() {
            return -1.0;
        }
        0.0
    }
}

fn has_valid_parts(lexicographical: bool, parts: &[String]) -> bool {
    for part in parts {
        let regex: Regex = if lexicographical {
            Regex::new(r"^\d+[A-Za-z]*$").unwrap()
        } else {
            Regex::new(r"^\d+$").unwrap()
        };
        if !regex.is_match(part) {
            return false;
        }
    }
    !parts.is_empty()
}