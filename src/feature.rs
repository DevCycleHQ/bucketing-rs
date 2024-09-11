mod feature {
    pub struct VariationVariable {
        var: str,     // `json:"_var"`
        value: std::any, // {} `json:"value"`
    }

    pub struct Variation {
        id: str,
        name: str,
        key: str,
        variables: [VariationVariable],
    }

    pub struct FeatureConfiguration {}

    pub struct ConfigFeature {
        id: str,
        featuretype: str,
        key: str,
        variations: [Variation],
        configuration: FeatureConfiguration,
        settings: str,
    }
}