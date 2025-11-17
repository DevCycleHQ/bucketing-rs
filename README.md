# DevCycle Bucketing Library - Multi-Platform Support

This library has been configured to build as:

1. **Rust library** - for use in Rust projects
2. **C library (FFI)** - for use in C/C++ applications
3. **WebAssembly (WASM)** - for use in web browsers and Node.js

### Rust Library

- `target/release/libdevcycle_bucketing_rs.rlib` - Rust static library
- Use in Rust projects via `Cargo.toml` dependencies

### C Library (FFI)

- `target/release/libdevcycle_bucketing_rs.a` - Static library (39MB)
- `target/release/libdevcycle_bucketing_rs.dylib` - Dynamic library for macOS (2.7MB)
- `target/release/libdevcycle_bucketing_rs.so` - Dynamic library for Linux
- `target/release/libdevcycle_bucketing_rs.dll` - Dynamic library for Windows
- `devcycle_bucketing.h` - C header file (generated with cbindgen)

### WebAssembly

- `pkg-web/` - WASM package for web browsers
- `pkg-node/` - WASM package for Node.js
- `pkg-bundler/` - WASM package for webpack/rollup/etc.

## Usage Examples

### Rust

```rust
use devcycle_bucketing_rs::{generate_bucketed_config_from_user, User};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let user = User {
        user_id: "test-user".to_string(),
        ..Default::default()
    };

    let config = unsafe {
        generate_bucketed_config_from_user(
            "your-sdk-key",
            user,
            HashMap::new()
        ).await
    };
}
```

### C/C++

```c
#include "devcycle_bucketing.h"

int main() {
    // Initialize full SDK state (config + queues + platform & custom data)
    const char* sdk_key = "your-sdk-key";
    const char* config_json = "{ /* full config JSON here */ }"; // See test resources for example structure
    int init_rc = devcycle_init_sdk_key(sdk_key, config_json, NULL, NULL, NULL);
    if (init_rc != 0) {
        char* err = devcycle_get_last_error();
        if (err) { fprintf(stderr, "Init error: %s\n", err); devcycle_free_string(err); }
        return 1;
    }

    // Minimal user JSON (only userId needed, other fields defaulted internally)
    const char* user_json = "{\"userId\":\"test-user\"}";

    // Queue a custom event for the user (populated & bucketed internally)
    int qrc = devcycle_queue_event(
        sdk_key,
        user_json,
        "CustomEvent",   // event type
        "purchase",      // customType
        "sku-123",       // target
        19.99,            // value
        "{\"amount\":19.99}"
    );
    if (qrc != 0) {
        char* err = devcycle_get_last_error();
        if (err) { fprintf(stderr, "Queue event error: %s\n", err); devcycle_free_string(err); }
    }

    return 0;
}
```

Compile with:

```bash
gcc -o myapp myapp.c -L./target/release -ldevcycle_bucketing_rs -lpthread -ldl -lm
```

### JavaScript (Web)

```javascript
import init, {
    init_event_queue,
    generate_bucketed_config_from_user
} from './pkg-web/devcycle_bucketing_rs.js';

async function main() {
    await init();

    await init_event_queue('your-sdk-key', null);

    const config = await generate_bucketed_config_from_user(
        'your-sdk-key',
        {user_id: 'test-user'},
        null
    );

    console.log(config);
}
```

### JavaScript (Node.js)

```javascript
const {
    init_event_queue,
    generate_bucketed_config_from_user
} = require('./pkg-node/devcycle_bucketing_rs.js');

async function main() {
    await init_event_queue('your-sdk-key', null);

    const config = await generate_bucketed_config_from_user(
        'your-sdk-key',
        {user_id: 'test-user'},
        null
    );

    console.log(config);
}

main().catch(console.error);
```

## Dependencies

### For C Library

- Install cbindgen to generate headers: `cargo install cbindgen`

### For WebAssembly

- Install wasm-pack:
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```

## Features

The library supports conditional compilation via Cargo features:

- `ffi` - Enable FFI bindings for C library
- `wasm` - Enable WebAssembly bindings

Build with specific features:

```bash
cargo build --release --features ffi
cargo build --release --features wasm
```

## Project Structure

```
src/
├── lib.rs              # Main library entry point
├── ffi.rs              # C FFI bindings (feature: ffi)
├── wasm.rs             # WASM bindings (feature: wasm)
├── bucketing/          # Core bucketing logic
├── config/             # Configuration management
├── events/             # Event queue system
├── segmentation/       # User segmentation
├── user/               # User data structures
└── util/               # Utilities
```

## Documentation

For detailed build instructions, see [BUILD.md](BUILD.md)

## Testing

Run tests:

```bash
cargo test
cargo test --features ffi
wasm-pack test --node --features wasm
```

## Size Information

The release builds are optimized for size, especially for WASM:

- C static library: ~39 MB
- C dynamic library (macOS): ~2.7 MB
- WASM package: Optimized with LTO

## Platform Support

- **Rust**: All platforms supported by Rust
- **C Library**: Linux, macOS, Windows (x86_64, ARM64)
- **WebAssembly**: Modern browsers, Node.js 14+
