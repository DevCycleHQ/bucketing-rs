#[cfg(all(test, feature = "protobuf"))]
mod protobuf_tests {
    use crate::protobuf::convert_proto_to_config_body;
    use crate::protobuf::proto;

    #[test]
    fn test_convert_proto_to_config_body() {
        let proto_config = proto::ConfigBodyProto {
            project: Some(proto::Project {
                id: "test-project-id".to_string(),
                key: "test-project-key".to_string(),
                a0_organization: "test-org".to_string(),
                settings: Some(proto::ProjectSettings {
                    edgedb: Some(proto::EdgeDbSettings { enabled: false }),
                    optin: Some(proto::OptInSettings {
                        enabled: false,
                        title: "".to_string(),
                        description: "".to_string(),
                        image_url: "".to_string(),
                        colors: Some(proto::OptInColors {
                            primary: "".to_string(),
                            secondary: "".to_string(),
                        }),
                    }),
                    disable_passthrough_rollouts: false,
                }),
            }),
            environment: Some(proto::Environment {
                id: "test-env-id".to_string(),
                key: "test-env-key".to_string(),
            }),
            sse: Some(proto::Sse {
                hostname: "test-hostname".to_string(),
                path: "/test-path".to_string(),
            }),
            audiences: std::collections::HashMap::new(),
            features: vec![],
            variables: vec![],
            etag: "test-etag".to_string(),
            ray_id: "test-ray-id".to_string(),
            last_modified: 1234567890,
        };

        let result = convert_proto_to_config_body(proto_config);
        assert!(result.is_ok());

        let config_body = result.unwrap();
        assert_eq!(config_body.project.key, "test-project-key");
        assert_eq!(config_body.environment.key, "test-env-key");
        assert_eq!(config_body.etag, "test-etag");
        assert_eq!(config_body.ray_id, "test-ray-id");
    }
}
