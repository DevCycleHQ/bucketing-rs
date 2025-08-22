pub(crate) mod filters {
    use crate::constants;
    use crate::user::user::PopulatedUser;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug)]
    enum FilterType {
        All,
        User,
        OptIn,
    }
    #[derive(Serialize, Deserialize, Debug)]
    enum FilterSubType {
        UserId,
        Email,
        IP,
        Country,
        Platform,
        PlatformVersion,
        AppVersion,
        DeviceModel,
        CustomData,
    }
    pub trait FilterOrOperator {
        fn evaluate(
            self: &Self,
            audiences: &HashMap<String, NoIdAudience>,
            user: &mut PopulatedUser,
            client_custom_data: &HashMap<String, serde_json::Value>,
        ) -> bool;
    }
    pub type MixedFilters = Vec<Box<dyn FilterOrOperator>>;
    pub struct PassFilter {}
    impl PassFilter {
        pub fn evaluate(
            audiences: &HashMap<String, NoIdAudience>,
            user: &mut PopulatedUser,
            client_custom_data: HashMap<String, serde_json::Value>,
        ) -> bool {
            true
        }
    }
    pub struct FailFilter {}
    impl FailFilter {
        pub fn evaluate(
            audiences: &HashMap<String, NoIdAudience>,
            user: PopulatedUser,
            client_custom_data: HashMap<String, serde_json::Value>,
        ) -> bool {
            false
        }
    }
    pub type AllFilter = PassFilter;
    pub type OptInFilter = FailFilter;

    pub struct AudienceOperator {
        operator: String,
        filters: MixedFilters,
    }

    impl AudienceOperator {
        pub fn get_operator(&self) -> String {
            return self.operator.clone();
        }

        pub fn get_filters(&self) -> &MixedFilters {
            return &self.filters;
        }

        pub fn evaluate(
            &self,
            audiences: &HashMap<String, NoIdAudience>,
            user: &mut PopulatedUser,
            client_custom_data: &HashMap<String, serde_json::Value>,
        ) -> bool {
            if self.filters.len() == 0 {
                return false;
            }
            if self.operator == constants::OPERATOR_AND {
                for filter in self.get_filters() {
                    if !filter.evaluate(audiences, user, client_custom_data) {
                        return false;
                    }
                }
                return true;
            } else if self.operator == constants::OPERATOR_OR {
                for filter in self.get_filters() {
                    if filter.evaluate(audiences, user, client_custom_data) {
                        return true;
                    }
                }
                return false;
            } else {
                return false;
            }
        }
    }
    pub struct NoIdAudience<'a> {
        filters: &'a AudienceOperator,
    }

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
            audiences: &HashMap<String, NoIdAudience>,
            user: PopulatedUser,
            client_custom_data: &HashMap<String, serde_json::Value>,
        ) -> bool {
            false
        }

        pub fn compile(&mut self) -> Result<(), String> {
            if self.values.len() == 0 {
                return Ok(());
            }
            let first_value = self.values.get(0);
            return match first_value {
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
            };
        }
    }

    trait Filter {
        fn get_type(&self) -> FilterType;
        fn get_sub_type(&self) -> FilterSubType;
        fn get_comparator(&self) -> &String;
        fn get_operator(&self) -> &String;
    }
}
