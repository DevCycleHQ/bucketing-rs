use crate::constants;
use crate::user::PopulatedUser;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FilterType {
    All,
    User,
    #[serde(rename = "optIn")]
    OptIn,
    #[serde(rename = "audienceMatch")]
    AudienceMatch,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FilterSubType {
    #[serde(rename = "user_id")]
    UserId,
    Email,
    #[serde(rename = "ip")]
    IP,
    Country,
    Platform,
    PlatformVersion,
    AppVersion,
    DeviceModel,
    CustomData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Filter {
    #[serde(rename = "type", default)]
    pub _type: String,
    #[serde(default)]
    pub sub_type: Option<String>,
    #[serde(default)]
    pub comparator: Option<String>,
    #[serde(default)]
    pub values: Vec<serde_json::Value>,
    #[serde(default)]
    pub filters: Vec<Filter>,
    #[serde(default)]
    pub operator: Option<String>,
    #[serde(default)]
    pub _audiences: Vec<String>,
}

impl Filter {
    pub fn evaluate(
        &self,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        match self._type.as_str() {
            constants::TYPE_ALL => true, // "all" filter always passes
            constants::TYPE_USER => self.evaluate_user_filter(user, client_custom_data),
            constants::TYPE_OPT_IN => false, // optIn filter always fails for now
            constants::TYPE_AUDIENCE_MATCH => {
                self.evaluate_audience_match(audiences, user, client_custom_data)
            }
            _ => {
                // If there are nested filters, evaluate them
                if !self.filters.is_empty() {
                    let operator = self.operator.as_deref().unwrap_or("and");
                    self.evaluate_nested_filters(operator, audiences, user, client_custom_data)
                } else {
                    false
                }
            }
        }
    }

    fn evaluate_user_filter(
        &self,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        let sub_type = match &self.sub_type {
            Some(st) => st.as_str(),
            None => return false,
        };

        let comparator = self.comparator.as_deref().unwrap_or("=");

        // Special handling for EXIST and NOT_EXIST comparators
        if comparator == constants::COMPARATOR_EXIST {
            let user_value = self.get_user_value(sub_type, user, client_custom_data);
            return match user_value {
                Some(serde_json::Value::String(s)) if s.is_empty() => false,
                Some(_) => true,
                None => false,
            };
        }

        if comparator == constants::COMPARATOR_NOT_EXIST {
            let user_value = self.get_user_value(sub_type, user, client_custom_data);
            return match user_value {
                Some(serde_json::Value::String(s)) if s.is_empty() => true,
                Some(_) => false,
                None => true,
            };
        }

        // For custom data filters, we need at least one value (the key)
        if sub_type == constants::SUB_TYPE_CUSTOM_DATA && self.values.is_empty() {
            return false;
        }

        // For non-custom data filters with empty values
        if sub_type != constants::SUB_TYPE_CUSTOM_DATA && self.values.is_empty() {
            return false;
        }

        let user_value = self.get_user_value(sub_type, user, client_custom_data);

        match user_value {
            Some(value) => self.compare_values(&value, comparator),
            None => {
                // When user value doesn't exist (None), != should return true
                comparator == constants::COMPARATOR_NOT_EQUAL
            }
        }
    }

    fn get_user_value(
        &self,
        sub_type: &str,
        user: &PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        match sub_type {
            constants::SUB_TYPE_USER_ID => Some(serde_json::Value::String(user.user_id.clone())),
            constants::SUB_TYPE_EMAIL => Some(serde_json::Value::String(user.email.clone())),
            constants::SUB_TYPE_COUNTRY => Some(serde_json::Value::String(user.country.clone())),
            constants::SUB_TYPE_PLATFORM => Some(serde_json::Value::String(
                user.platform_data.platform.clone(),
            )),
            constants::SUB_TYPE_PLATFORM_VERSION => Some(serde_json::Value::String(
                user.platform_data.platform_version.clone(),
            )),
            constants::SUB_TYPE_APP_VERSION => {
                Some(serde_json::Value::String(user.app_version.clone()))
            }
            constants::SUB_TYPE_DEVICE_MODEL => {
                Some(serde_json::Value::String(user.device_model.clone()))
            }
            constants::SUB_TYPE_CUSTOM_DATA => {
                // For custom data, we need to look in the client_custom_data or user.custom_data
                if !self.values.is_empty() {
                    if let Some(key) = self.values[0].as_str() {
                        user.custom_data
                            .get(key)
                            .cloned()
                            .or_else(|| client_custom_data.get(key).cloned())
                            .or_else(|| user.private_custom_data.get(key).cloned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn compare_version_strings(&self, user_version: &str, comparator: &str) -> bool {
        // For != operator, we need ALL values to NOT match (return true if none match)
        if comparator == constants::COMPARATOR_NOT_EQUAL || comparator == "!=" {
            for filter_value in &self.values {
                if let serde_json::Value::String(filter_version) = filter_value {
                    let result = version_compare_equality(user_version, filter_version);
                    // If any value matches, return false
                    if result {
                        return false;
                    }
                }
            }
            // If no values matched, return true
            return true;
        }

        // For all other operators, return true if ANY value matches
        for filter_value in &self.values {
            if let serde_json::Value::String(filter_version) = filter_value {
                let matches = match comparator {
                    constants::COMPARATOR_EQUAL | "=" => {
                        version_compare_equality(user_version, filter_version)
                    }
                    _ => {
                        let result = version_compare(user_version, filter_version);
                        match comparator {
                            constants::COMPARATOR_GREATER | ">" => result > 0.0,
                            constants::COMPARATOR_GREATER_EQUAL | ">=" => result >= 0.0,
                            constants::COMPARATOR_LESS | "<" => result < 0.0,
                            constants::COMPARATOR_LESS_EQUAL | "<=" => result <= 0.0,
                            _ => false,
                        }
                    }
                };

                if matches {
                    return true;
                }
            }
        }
        false
    }

    fn compare_values(&self, user_value: &serde_json::Value, comparator: &str) -> bool {
        // Check if this is a version comparison
        let is_version_field = matches!(
            self.sub_type.as_deref(),
            Some(constants::SUB_TYPE_APP_VERSION) | Some(constants::SUB_TYPE_PLATFORM_VERSION)
        );

        if is_version_field {
            if let serde_json::Value::String(user_str) = user_value {
                return self.compare_version_strings(user_str, comparator);
            }
        }

        // Special handling for != and !contain - ALL values must NOT match
        if comparator == constants::COMPARATOR_NOT_EQUAL || comparator == "!=" {
            for filter_value in &self.values {
                if user_value == filter_value {
                    return false; // If any value matches, fail
                }
            }
            return true; // All values didn't match, pass
        }

        if comparator == constants::COMPARATOR_NOT_CONTAIN {
            for filter_value in &self.values {
                if let (
                    serde_json::Value::String(user_str),
                    serde_json::Value::String(filter_str),
                ) = (user_value, filter_value)
                {
                    if !filter_str.is_empty() && user_str.contains(filter_str) {
                        return false; // If it contains any value, fail
                    }
                }
            }
            return true; // Doesn't contain any of the values, pass
        }

        if comparator == constants::COMPARATOR_NOT_START_WITH {
            for filter_value in &self.values {
                if let (
                    serde_json::Value::String(user_str),
                    serde_json::Value::String(filter_str),
                ) = (user_value, filter_value)
                {
                    if !filter_str.is_empty() && user_str.starts_with(filter_str) {
                        return false; // If it starts with any value, fail
                    }
                }
            }
            return true; // Doesn't start with any of the values, pass
        }

        if comparator == constants::COMPARATOR_NOT_END_WITH {
            for filter_value in &self.values {
                if let (
                    serde_json::Value::String(user_str),
                    serde_json::Value::String(filter_str),
                ) = (user_value, filter_value)
                {
                    if !filter_str.is_empty() && user_str.ends_with(filter_str) {
                        return false; // If it ends with any value, fail
                    }
                }
            }
            return true; // Doesn't end with any of the values, pass
        }

        // For all other operators, ANY matching value passes
        for filter_value in &self.values {
            let matches = match comparator {
                constants::COMPARATOR_EQUAL | "=" => user_value == filter_value,
                constants::COMPARATOR_CONTAIN => {
                    if let (
                        serde_json::Value::String(user_str),
                        serde_json::Value::String(filter_str),
                    ) = (user_value, filter_value)
                    {
                        !filter_str.is_empty() && user_str.contains(filter_str)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_START_WITH => {
                    if let (
                        serde_json::Value::String(user_str),
                        serde_json::Value::String(filter_str),
                    ) = (user_value, filter_value)
                    {
                        !filter_str.is_empty() && user_str.starts_with(filter_str)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_END_WITH => {
                    if let (
                        serde_json::Value::String(user_str),
                        serde_json::Value::String(filter_str),
                    ) = (user_value, filter_value)
                    {
                        !filter_str.is_empty() && user_str.ends_with(filter_str)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_GREATER => {
                    if let (
                        serde_json::Value::Number(user_num),
                        serde_json::Value::Number(filter_num),
                    ) = (user_value, filter_value)
                    {
                        user_num.as_f64().unwrap_or(0.0) > filter_num.as_f64().unwrap_or(0.0)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_GREATER_EQUAL => {
                    if let (
                        serde_json::Value::Number(user_num),
                        serde_json::Value::Number(filter_num),
                    ) = (user_value, filter_value)
                    {
                        user_num.as_f64().unwrap_or(0.0) >= filter_num.as_f64().unwrap_or(0.0)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_LESS => {
                    if let (
                        serde_json::Value::Number(user_num),
                        serde_json::Value::Number(filter_num),
                    ) = (user_value, filter_value)
                    {
                        user_num.as_f64().unwrap_or(0.0) < filter_num.as_f64().unwrap_or(0.0)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_LESS_EQUAL => {
                    if let (
                        serde_json::Value::Number(user_num),
                        serde_json::Value::Number(filter_num),
                    ) = (user_value, filter_value)
                    {
                        user_num.as_f64().unwrap_or(0.0) <= filter_num.as_f64().unwrap_or(0.0)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_EXIST => true, // If we got a value, it exists
                constants::COMPARATOR_NOT_EXIST => false, // If we got a value, it exists, so NOT_EXIST is false
                _ => false,
            };

            if matches {
                return true; // Any matching value passes the filter
            }
        }
        false
    }

    fn evaluate_audience_match(
        &self,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        let comparator = self.comparator.as_deref().unwrap_or("=");

        // Check if user matches any of the referenced audiences
        let mut matches_any = false;
        for audience_id in &self._audiences {
            if let Some(audience) = audiences.get(audience_id) {
                if audience
                    .filters
                    .evaluate(audiences, user, client_custom_data)
                {
                    matches_any = true;
                    break;
                }
            }
        }

        // If comparator is NOT_EQUAL, invert the result
        if comparator == constants::COMPARATOR_NOT_EQUAL || comparator == "!=" {
            !matches_any
        } else {
            matches_any
        }
    }

    fn evaluate_nested_filters(
        &self,
        operator: &str,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        if self.filters.is_empty() {
            return true;
            // // Standard boolean logic: empty AND is true (vacuous truth), empty OR is false
            // return match self.operator.unwrap().clone().to_string() {
            //     constants::OPERATOR_AND => true,
            //     constants::OPERATOR_OR => false,
            //     _ => false,
            // };
        }

        match operator {
            constants::OPERATOR_AND => {
                for filter in &self.filters {
                    if !filter.evaluate(audiences, user, client_custom_data) {
                        return false;
                    }
                }
                true
            }
            constants::OPERATOR_OR => {
                for filter in &self.filters {
                    if filter.evaluate(audiences, user, client_custom_data) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

pub trait FilterOrOperator {
    fn evaluate(
        &self,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AudienceOperator {
    pub operator: String,
    pub filters: Vec<Filter>,
}

impl AudienceOperator {
    pub fn get_operator(&self) -> String {
        self.operator.clone()
    }

    pub fn get_filters(&self) -> &Vec<Filter> {
        &self.filters
    }

    pub fn evaluate(
        &self,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        if self.filters.is_empty() {
            // Standard boolean logic: empty AND is true (vacuous truth), empty OR is false
            return match self.operator.as_str() {
                constants::OPERATOR_AND => true,
                constants::OPERATOR_OR => false,
                _ => false,
            };
        }

        match self.operator.as_str() {
            constants::OPERATOR_AND => {
                for filter in &self.filters {
                    if !filter.evaluate(audiences, user, client_custom_data) {
                        return false;
                    }
                }
                true
            }
            constants::OPERATOR_OR => {
                for filter in &self.filters {
                    if filter.evaluate(audiences, user, client_custom_data) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl FilterOrOperator for AudienceOperator {
    fn evaluate(
        &self,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        self.evaluate(audiences, user, client_custom_data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoIdAudience {
    pub filters: AudienceOperator,
}

// Legacy filter types - keeping for backward compatibility but using new implementation
pub struct PassFilter {}
impl PassFilter {
    pub fn evaluate(
        _audiences: &HashMap<String, NoIdAudience>,
        _user: &mut PopulatedUser,
        _client_custom_data: HashMap<String, serde_json::Value>,
    ) -> bool {
        true
    }
}

pub struct FailFilter {}
impl FailFilter {
    pub fn evaluate(
        _audiences: &HashMap<String, NoIdAudience>,
        _user: PopulatedUser,
        _client_custom_data: HashMap<String, serde_json::Value>,
    ) -> bool {
        false
    }
}

pub type AllFilter = PassFilter;
pub type OptInFilter = FailFilter;

// Legacy filter structures - keeping for backward compatibility
pub struct BaseFilter {
    _type: FilterType,
    sub_type: FilterSubType,
    comparator: &'static String,
    operator: &'static String,
}

pub struct UserFilter {
    base: BaseFilter,
    values: Vec<serde_json::Value>,
    compiled_string_vals: Vec<String>,
    compiled_bool_vals: Vec<bool>,
    compiled_num_vals: Vec<f64>,
}

impl UserFilter {
    pub fn evaluate(
        &self,
        _audiences: &HashMap<String, NoIdAudience>,
        _user: PopulatedUser,
        _client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        // Legacy implementation - use new Filter struct instead
        false
    }

    pub fn compile(&mut self) -> Result<(), String> {
        if self.values.is_empty() {
            return Ok(());
        }
        let first_value = self.values.get(0);
        match first_value {
            Some(value) => match value {
                serde_json::Value::Bool(_) => {
                    let mut bool_values: Vec<bool> = Vec::new();
                    for value in &self.values {
                        match value {
                            serde_json::Value::Bool(val) => {
                                bool_values.push(*val);
                            }
                            _ => {
                                return Err(format!(
                                    "Filter values must be all of the same type. Expected: bool, got: {:?}",
                                    value
                                ));
                            }
                        }
                    }
                    self.compiled_bool_vals = bool_values;
                    Ok(())
                }
                serde_json::Value::String(_) => {
                    let mut string_values: Vec<String> = Vec::new();
                    for value in self.values.clone() {
                        match value {
                            serde_json::Value::String(val) => {
                                string_values.push(val);
                            }
                            _ => {
                                return Err(format!(
                                    "Filter values must be all of the same type. Expected: string, got: {:?}",
                                    value
                                ));
                            }
                        }
                    }
                    self.compiled_string_vals = string_values;
                    Ok(())
                }
                serde_json::Value::Number(_) => {
                    let mut num_values: Vec<f64> = Vec::new();
                    for value in &self.values {
                        match value {
                            serde_json::Value::Number(val) => {
                                num_values.push(val.as_f64().unwrap());
                            }
                            _ => {
                                return Err(format!(
                                    "Filter values must be all of the same type. Expected: number, got: {:?}",
                                    value
                                ));
                            }
                        }
                    }
                    self.compiled_num_vals = num_values;
                    Ok(())
                }
                _ => Err(format!(
                    "Filter values must be of type bool, string, or number. Got: {:?}",
                    value
                )),
            },
            None => Ok(()),
        }
    }
}

trait FilterTrait {
    fn get_type(&self) -> FilterType;
    fn get_sub_type(&self) -> FilterSubType;
    fn get_comparator(&self) -> &String;
    fn get_operator(&self) -> &String;
}

fn version_compare(v1: &str, v2: &str) -> f64 {
    // Extract only the first continuous version string (e.g., "1.2.3" from "v1.2.3-beta")
    let extract_version = |s: &str| -> Vec<u32> {
        let re = Regex::new(r"(\d+\.)*\d+").unwrap();
        if let Some(mat) = re.find(s) {
            mat.as_str()
                .split('.')
                .filter_map(|p| p.parse::<u32>().ok())
                .collect()
        } else {
            Vec::new()
        }
    };

    let v1_parts = extract_version(v1);
    let v2_parts = extract_version(v2);

    if v1_parts.is_empty() && v2_parts.is_empty() {
        return 0.0;
    }

    if v1_parts.is_empty() {
        return f64::NAN;
    }

    if v2_parts.is_empty() {
        return f64::NAN;
    }

    // Try to use semver for standard 3-part versions (or shorter)
    if v1_parts.len() <= 3 && v2_parts.len() <= 3 {
        let normalize_to_semver = |parts: &[u32]| -> String {
            match parts.len() {
                1 => format!("{}.0.0", parts[0]),
                2 => format!("{}.{}.0", parts[0], parts[1]),
                3 => format!("{}.{}.{}", parts[0], parts[1], parts[2]),
                _ => String::new(),
            }
        };

        let v1_semver_str = normalize_to_semver(&v1_parts);
        let v2_semver_str = normalize_to_semver(&v2_parts);

        if let (Ok(v1_semver), Ok(v2_semver)) = (
            Version::parse(&v1_semver_str),
            Version::parse(&v2_semver_str),
        ) {
            return if v1_semver > v2_semver {
                1.0
            } else if v1_semver < v2_semver {
                -1.0
            } else {
                0.0
            };
        }
    }

    // Fall back to custom comparison for 4+ part versions
    let min_len = v1_parts.len().min(v2_parts.len());

    for i in 0..min_len {
        let v1_part = v1_parts[i];
        let v2_part = v2_parts[i];

        if v1_part > v2_part {
            return 1.0;
        } else if v1_part < v2_part {
            return -1.0;
        }
    }

    // If all compared parts are equal, check if one version has more parts
    if v1_parts.len() > v2_parts.len() {
        // Check if remaining v1 parts are all zeros
        for i in min_len..v1_parts.len() {
            if v1_parts[i] > 0 {
                return 1.0;
            }
        }
    } else if v2_parts.len() > v1_parts.len() {
        // Check if remaining v2 parts are all zeros
        for i in min_len..v2_parts.len() {
            if v2_parts[i] > 0 {
                return -1.0;
            }
        }
    }

    0.0
}

fn version_compare_equality(v1: &str, v2: &str) -> bool {
    // For equality, extract the matched portion to see if strings are truly equivalent
    let re = Regex::new(r"(\d+\.)*\d+").unwrap();

    let v1_match = re.find(v1);
    let v2_match = re.find(v2);

    match (v1_match, v2_match) {
        (Some(m1), Some(m2)) => {
            let v1_version_str = m1.as_str();
            let v2_version_str = m2.as_str();

            // Parse the version parts
            let v1_parts: Vec<u32> = v1_version_str
                .split('.')
                .filter_map(|p| p.parse::<u32>().ok())
                .collect();

            let v2_parts: Vec<u32> = v2_version_str
                .split('.')
                .filter_map(|p| p.parse::<u32>().ok())
                .collect();

            // For equality, versions must have the same number of parts
            if v1_parts.len() != v2_parts.len() {
                return false;
            }

            // Check if numeric parts are equal
            if v1_parts != v2_parts {
                return false;
            }

            // Check for prefix/suffix, but ignore trailing dots/periods
            let v1_has_prefix = m1.start() > 0;
            let v2_has_prefix = m2.start() > 0;

            // For suffix, ignore trailing dots and whitespace
            let v1_suffix = &v1[m1.end()..].trim_start_matches('.');
            let v2_suffix = &v2[m2.end()..].trim_start_matches('.');
            let v1_has_suffix = !v1_suffix.is_empty();
            let v2_has_suffix = !v2_suffix.is_empty();

            // If one has prefix/suffix and other doesn't, they're not equal
            if v1_has_prefix != v2_has_prefix || v1_has_suffix != v2_has_suffix {
                return false;
            }

            // If both have suffixes (after ignoring trailing dots), they must be identical
            if v1_has_suffix && v2_has_suffix {
                if v1_suffix != v2_suffix {
                    return false;
                }
            }

            // If both have prefixes, they must be identical for equality
            if v1_has_prefix && v2_has_prefix {
                let v1_prefix = &v1[..m1.start()];
                let v2_prefix = &v2[..m2.start()];
                if v1_prefix != v2_prefix {
                    return false;
                }
            }

            true
        }
        _ => false,
    }
}
