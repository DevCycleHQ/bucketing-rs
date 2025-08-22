pub mod feature {
    use std::collections::HashMap;

    use crate::target::target::Target;

    pub struct VariationVariable {
        pub _var: String,             // `json:"_var"`
        pub value: serde_json::Value, // {} `json:"value"`
    }
    pub struct ReadOnlyVariable {
        pub id: String,
        pub key: String,
        pub _type: String,
        pub value: serde_json::Value,
    }
    pub(crate) struct Variation {
        pub(crate) _id: String,
        pub(crate) name: String,
        pub(crate) key: String,
        pub(crate) variables: Vec<VariationVariable>,
    }

    pub struct FeatureVariation {
        pub feature: String,   // `json:"_feature"`
        pub variation: String, // `json:"_variation"`
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

    pub struct ConfigFeature {
        pub _id: String,
        pub featuretype: String,
        pub key: String,
        pub variations: Vec<Variation>,
        pub configuration: FeatureConfiguration,
        pub settings: String,
    }

    pub struct FeatureConfiguration {
        pub _id: String,
        pub prerequisites: Vec<FeaturePrerequisites>,
        pub winning_variation: FeatureVariation,
        pub forced_users: HashMap<String, String>,
        pub targets: Vec<Target>,
    }
    pub struct FeaturePrerequisites {
        _feature: String,   // `json:"_feature"`
        comparator: String, // `json:"comparator"`
    }

    pub struct Feature {
        pub _id: String,
        pub key: String,
        pub _type: String,
        pub variation: String,
        pub variationkey: String,
        pub variationname: String,
        pub evalreason: String,
    }
}
