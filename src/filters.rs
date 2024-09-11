mod filters {
    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};
    use crate::constants;

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
        fn evaluate(audiences: &HashMap<str, NoIdAudience>, user: crate::user, client_custom_data: &HashMap<str, std::any>) -> bool;
    }

    struct PassFilter {}
    impl PassFilter {
        pub fn evaluate(audiences: &HashMap<str, NoIdAudience>, user: crate::user, client_custom_data: HashMap<str, std::any>) -> bool {
            true
        }
    }
    struct FailFilter {}
    impl FailFilter {
        pub fn evaluate(audiences: &HashMap<str, NoIdAudience>, user: crate::user, client_custom_data: HashMap<str, std::any>) -> bool {
            false
        }
    }

    type AllFilter = PassFilter;


    type MixedFilters = Vec<dyn FilterOrOperator>;



    #[derive(Serialize, Deserialize, Debug)]
    pub struct AudienceOperator {
        operator: str,
        filters: MixedFilters,
    }

    impl AudienceOperator {
        pub fn evaluate(&self, audiences: &HashMap<str, NoIdAudience>, user: crate::user, client_custom_data: &HashMap<str, std::any>) -> bool {
            if self.filters.len() == 0 {
                return false;
            }
            if self.operator == constants::OPERATOR_AND {
                for filter in self.filters {
                    if !filter.evaluate(audiences, user, client_custom_data) {
                        return false;
                    }
                }
                return true;
            } else if self.operator == constants::OPERATOR_OR {
                for filter in self.filters {
                    if filter.evaluate(audiences, user, client_custom_data) {
                        return true;
                    }
                }
                return false;
            }
        }
    }


    #[derive(Serialize, Deserialize, Debug)]
    pub struct NoIdAudience<'a> {
        filters: &'a AudienceOperator,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct BaseFilter {
        _type: FilterType,
        sub_type: FilterSubType,
        comparator: str,
        operator: str,
    }

    struct UserFilter {
        base: BaseFilter,
        values: Vec<std::any>,

        CompiledStringVals: [str],
        CompiledBoolVals   []bool,
        CompiledNumVals    []float64,

    }
    trait Filter {
        fn get_type(&self) -> FilterType;
        fn get_sub_type(&self) -> FilterSubType;
        fn get_comparator(&self) -> str;
        fn get_operator(&self) -> str;
    }
}