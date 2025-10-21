# Building DevCycle Bucketing Library

This library can be built in three different ways:

## 1. Rust Library (default)

Build as a standard Rust library:

```bash
cargo build --release
```

This creates `target/release/libdevcycle_bucketing_rs.rlib` which can be used by other Rust projects.

## 2. C Library (FFI)

Build with C FFI support for use in C/C++ applications:

```bash
cargo build --release --features ffi
```

This creates:

- `target/release/libdevcycle_bucketing_rs.dylib` (macOS)
- `target/release/libdevcycle_bucketing_rs.so` (Linux)
- `target/release/libdevcycle_bucketing_rs.dll` (Windows)

### C Header Generation

To generate C headers for the FFI functions, install `cbindgen`:

```bash
cargo install cbindgen
cbindgen --config cbindgen.toml --crate devcycle-bucketing-rs --output devcycle_bucketing.h
```

### Using from C/C++

```c
#include "devcycle_bucketing.h"

// Initialize event queue
int result = devcycle_init_event_queue("your-sdk-key", NULL);

// Generate bucketed config from user JSON
const char* user_json = "{\"user_id\":\"test-user\"}";
CBucketedUserConfig* config = devcycle_generate_bucketed_config_from_user(
    "your-sdk-key", 
    devcycle_user_from_json(user_json),
    NULL
);

// Get JSON representation
char* config_json = devcycle_bucketed_config_to_json(config);
printf("%s\n", config_json);

// Clean up
devcycle_free_string(config_json);
devcycle_free_bucketed_config(config);
```

## 3. WebAssembly (WASM)

Build for WebAssembly to use in browsers or Node.js:

### Prerequisites

Install `wasm-pack`:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build for Web

```bash
wasm-pack build --target web --features wasm
mv pkg pkg-web
```

### Build for Node.js

```bash
wasm-pack build --target nodejs --features wasm
mv pkg pkg-node
```

### Build for Bundlers (webpack, etc.)

```bash
wasm-pack build --target bundler --features wasm
mv pkg pkg-bundler
```

### Using in JavaScript/TypeScript

```javascript
import init, {init_event_queue, generate_bucketed_config_from_user} from './pkg-web/devcycle_bucketing_rs.js';

async function example() {
    // Initialize the WASM module
    await init();

    // Initialize event queue
    await init_event_queue('your-sdk-key', null);

    // Generate bucketed config
    const user = {
        user_id: 'test-user',
        email: 'user@example.com'
    };

    const config = await generate_bucketed_config_from_user(
        'your-sdk-key',
        user,
        null
    );

    console.log(config);
}
```

### Using in Node.js

```javascript
const {init_event_queue, generate_bucketed_config_from_user} = require('./pkg-node/devcycle_bucketing_rs.js');

async function example() {
    // Initialize event queue
    await init_event_queue('your-sdk-key', null);

    // Generate bucketed config
    const user = {
        user_id: 'test-user',
        email: 'user@example.com'
    };

    const config = await generate_bucketed_config_from_user(
        'your-sdk-key',
        user,
        null
    );

    console.log(config);
}

example().catch(console.error);
```

## Build All Targets

To build all three variants:

```bash
# Rust library
cargo build --release

# C library
cargo build --release --features ffi

# WASM (web)
wasm-pack build --target web --features wasm
mv pkg pkg-web

# WASM (node)
wasm-pack build --target nodejs --features wasm
mv pkg pkg-node

# WASM (bundler)
wasm-pack build --target bundler --features wasm
mv pkg pkg-bundler
```

## Testing

Run tests:

```bash
cargo test
```

Run tests with specific features:

```bash
cargo test --features ffi
# Note: WASM tests require wasm-pack test
wasm-pack test --node --features wasm
```

## Size Optimization

The WASM build is already optimized for size (see `[profile.release]` in `Cargo.toml`).

For further size reduction:

```bash
wasm-pack build --target web --features wasm --release -- -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
```

