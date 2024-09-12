pub mod feature {
    use std::collections::HashMap;
    use serde_json::Value;
    use crate::target::target::Target;

    pub struct VariationVariable {
        _var: String,     // `json:"_var"`
        value: serde_json::Value, // {} `json:"value"`
    }

    pub struct Variation {
        _id: String,
        name: String,
        key: String,
        variables: Vec<VariationVariable>,
    }

    pub struct FeatureVariation {
        feature: String, // `json:"_feature"`
        variation: String, // `json:"_variation"`
    }

    impl Variation {
        pub fn get_variable_by_id(&self, id: &str) -> Option<&VariationVariable> {
            for variable in &self.variables {
                if variable._var == id {
                    return Some(variable);
                }
            }
            None
        }
    }

    pub struct ConfigFeature<'a> {
        _id: String,
        featuretype: String,
        key: String,
        variations: Vec<Variation>,
        configuration: FeatureConfiguration<'a>,
        settings: String,
    }

    pub struct FeatureConfiguration<'a> {
        _id: String,
        prerequisites: Vec<FeaturePrerequisites>,
        winning_variation: FeatureVariation,
        forced_users: HashMap<String, String>,
        targets: Vec<Target<'a>>,
    }
    pub struct FeaturePrerequisites {
        _feature: String, // `json:"_feature"`
        comparator: String, // `json:"comparator"`
    }
}