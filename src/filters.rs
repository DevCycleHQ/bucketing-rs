use crate::constants;
use crate::user::PopulatedUser;
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
        if self.values.is_empty() {
            return false;
        }

        let sub_type = match &self.sub_type {
            Some(st) => st.as_str(),
            None => return false,
        };

        let comparator = self.comparator.as_deref().unwrap_or("=");
        let user_value = self.get_user_value(sub_type, user, client_custom_data);

        match user_value {
            Some(value) => self.compare_values(&value, comparator),
            None => comparator == constants::COMPARATOR_NOT_EXIST,
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

    fn compare_values(&self, user_value: &serde_json::Value, comparator: &str) -> bool {
        for filter_value in &self.values {
            let matches = match comparator {
                constants::COMPARATOR_EQUAL | "=" => user_value == filter_value,
                constants::COMPARATOR_NOT_EQUAL | "!=" => user_value != filter_value,
                constants::COMPARATOR_CONTAIN => {
                    if let (
                        serde_json::Value::String(user_str),
                        serde_json::Value::String(filter_str),
                    ) = (user_value, filter_value)
                    {
                        user_str.contains(filter_str)
                    } else {
                        false
                    }
                }
                constants::COMPARATOR_NOT_CONTAIN => {
                    if let (
                        serde_json::Value::String(user_str),
                        serde_json::Value::String(filter_str),
                    ) = (user_value, filter_value)
                    {
                        !user_str.contains(filter_str)
                    } else {
                        true
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
        // For audience match, check if user matches any of the referenced audiences
        for audience_id in &self._audiences {
            if let Some(audience) = audiences.get(audience_id) {
                if audience
                    .filters
                    .evaluate(audiences, user, client_custom_data)
                {
                    return true;
                }
            }
        }
        false
    }

    fn evaluate_nested_filters(
        &self,
        operator: &str,
        audiences: &HashMap<String, NoIdAudience>,
        user: &mut PopulatedUser,
        client_custom_data: &HashMap<String, serde_json::Value>,
    ) -> bool {
        if self.filters.is_empty() {
            return false;
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
            return true; // Empty filters pass by default
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
                                return Err(format!("Filter values must be all of the same type. Expected: bool, got: {:?}", value));
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
                                return Err(format!("Filter values must be all of the same type. Expected: string, got: {:?}", value));
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
                                return Err(format!("Filter values must be all of the same type. Expected: number, got: {:?}", value));
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
