mod feature {
    pub struct VariationVariable {
        var: String,     // `json:"_var"`
        value: serde_json::Value, // {} `json:"value"`
    }

    pub struct Variation {
        id: String,
        name: String,
        key: String,
        variables: Vec<VariationVariable>,
    }

    pub struct FeatureConfiguration {}

    pub struct ConfigFeature {
        id: String,
        featuretype: String,
        key: String,
        variations: Vec<Variation>,
        configuration: FeatureConfiguration,
        settings: String,
    }
}