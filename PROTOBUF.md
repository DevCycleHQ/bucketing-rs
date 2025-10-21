# Protobuf Support for bucketing-rs

This document describes the protobuf support added to the bucketing-rs library.

## Overview

A new `set_config_from_protobuf` function has been added that allows setting configuration using Protocol Buffers as the
data model, providing an alternative to the existing JSON-based `set_config` function.

## Building with Protobuf Support

To enable protobuf support, build the project with the `protobuf` feature flag:

```bash
cargo build --features protobuf
```

## Usage

### Using the Protobuf Function

```rust
use devcycle_bucketing_rs::{set_config_from_protobuf, protobuf::proto};

async fn example() {
    let sdk_key = "my-sdk-key";

    // Create a protobuf ConfigBodyProto message
    let proto_config = proto::ConfigBodyProto {
        project: Some(proto::Project {
            id: "project-id".to_string(),
            key: "project-key".to_string(),
            a0_organization: "org-id".to_string(),
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
            id: "env-id".to_string(),
            key: "env-key".to_string(),
        }),
        sse: Some(proto::Sse {
            hostname: "sse-hostname".to_string(),
            path: "/sse-path".to_string(),
        }),
        audiences: std::collections::HashMap::new(),
        features: vec![],
        variables: vec![],
        etag: "etag-value".to_string(),
        ray_id: "ray-id-value".to_string(),
        last_modified: 1234567890, // Unix timestamp
    };

    // Set the config using protobuf
    let result = set_config_from_protobuf(sdk_key, proto_config).await;

    match result {
        Ok(_) => println!("Config set successfully"),
        Err(e) => eprintln!("Error setting config: {}", e),
    }
}
```

## Protobuf Schema

The protobuf schema is defined in `proto/config.proto`. Key message types include:

- `ConfigBodyProto`: The main configuration message
- `Project`: Project information
- `Environment`: Environment information
- `SSE`: Server-sent events configuration
- `Variable`: Variable definitions
- `Audience`: Audience targeting rules (JSON-encoded)
- `ConfigFeature`: Feature configurations (JSON-encoded)

## Comparison with JSON-based Function

### Original `set_config` (JSON-based)

```rust
pub async fn set_config(
    sdk_key: &str,
    config_body: ConfigBody<'static>,
) -> Result<(), DevCycleError>
```

### New `set_config_from_protobuf` (Protobuf-based)

```rust
pub async fn set_config_from_protobuf(
    sdk_key: &str,
    proto_config: protobuf::proto::ConfigBodyProto,
) -> Result<(), DevCycleError>
```

Both functions achieve the same result but accept different input formats. The protobuf version is useful when:

- Receiving configuration data from protobuf-based services
- Needing more efficient binary serialization
- Working with systems that standardize on protobuf

## Implementation Details

The conversion from protobuf to the internal `ConfigBody` structure is handled by the `convert_proto_to_config_body`
function in the `protobuf` module. This function:

1. Converts protobuf messages to internal Rust structs
2. Deserializes JSON-encoded fields (audiences and features)
3. Builds internal lookup maps for efficient access
4. Manages lifetime requirements for the configuration data

## Dependencies

The protobuf feature adds the following dependencies:

- `prost`: Protobuf implementation for Rust
- `prost-types`: Well-known protobuf types
- `prost-build`: Build-time protobuf code generation

These dependencies are optional and only included when the `protobuf` feature is enabled.

