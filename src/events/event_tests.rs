#[cfg(test)]
mod tests {
    use crate::event::{DefaultReason, EvaluationReason};

    #[test]
    fn test_evaluation_reason_display() {
        assert_eq!(
            EvaluationReason::TargetingMatch.to_string(),
            "TARGETING_MATCH"
        );
        assert_eq!(EvaluationReason::Split.to_string(), "SPLIT");
        assert_eq!(EvaluationReason::Default.to_string(), "DEFAULT");
        assert_eq!(EvaluationReason::Disabled.to_string(), "DISABLED");
        assert_eq!(EvaluationReason::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_evaluation_reason_format() {
        assert_eq!(
            format!("{}", EvaluationReason::TargetingMatch),
            "TARGETING_MATCH"
        );
        assert_eq!(format!("{}", EvaluationReason::Split), "SPLIT");
        assert_eq!(format!("{}", EvaluationReason::Default), "DEFAULT");
        assert_eq!(format!("{}", EvaluationReason::Disabled), "DISABLED");
        assert_eq!(format!("{}", EvaluationReason::Error), "ERROR");
    }

    #[test]
    fn test_default_reason_display() {
        assert_eq!(DefaultReason::MissingConfig.to_string(), "Missing Config");
        assert_eq!(
            DefaultReason::MissingVariable.to_string(),
            "Missing Variable"
        );
        assert_eq!(DefaultReason::MissingFeature.to_string(), "Missing Feature");
        assert_eq!(
            DefaultReason::MissingVariation.to_string(),
            "Missing Variation"
        );
        assert_eq!(
            DefaultReason::MissingVariableForVariation.to_string(),
            "Missing Variable for Variation"
        );
        assert_eq!(
            DefaultReason::UserNotInRollout.to_string(),
            "User Not in Rollout"
        );
        assert_eq!(
            DefaultReason::UserNotTargeted.to_string(),
            "User Not Targeted"
        );
        assert_eq!(
            DefaultReason::InvalidVariableType.to_string(),
            "Invalid Variable Type"
        );
        assert_eq!(
            DefaultReason::VariableTypeMismatch.to_string(),
            "Variable Type Mismatch"
        );
        assert_eq!(DefaultReason::Unknown.to_string(), "Unknown");
        assert_eq!(DefaultReason::Error.to_string(), "Error");
        assert_eq!(DefaultReason::NotDefaulted.to_string(), "");
    }

    #[test]
    fn test_default_reason_format() {
        assert_eq!(
            format!("{}", DefaultReason::MissingConfig),
            "Missing Config"
        );
        assert_eq!(
            format!("{}", DefaultReason::MissingVariable),
            "Missing Variable"
        );
        assert_eq!(
            format!("{}", DefaultReason::MissingFeature),
            "Missing Feature"
        );
        assert_eq!(
            format!("{}", DefaultReason::MissingVariation),
            "Missing Variation"
        );
        assert_eq!(
            format!("{}", DefaultReason::MissingVariableForVariation),
            "Missing Variable for Variation"
        );
        assert_eq!(
            format!("{}", DefaultReason::UserNotInRollout),
            "User Not in Rollout"
        );
        assert_eq!(
            format!("{}", DefaultReason::UserNotTargeted),
            "User Not Targeted"
        );
        assert_eq!(
            format!("{}", DefaultReason::InvalidVariableType),
            "Invalid Variable Type"
        );
        assert_eq!(
            format!("{}", DefaultReason::VariableTypeMismatch),
            "Variable Type Mismatch"
        );
        assert_eq!(format!("{}", DefaultReason::Unknown), "Unknown");
        assert_eq!(format!("{}", DefaultReason::Error), "Error");
        assert_eq!(format!("{}", DefaultReason::NotDefaulted), "");
    }
}
