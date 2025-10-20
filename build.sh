#!/bin/bash

# Build script for DevCycle Bucketing Library
# Supports building as Rust library, C library (FFI), and WebAssembly

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}==>${NC} $1"
}

print_error() {
    echo -e "${RED}Error:${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

# Parse command line arguments
BUILD_TYPE=${1:-all}

case $BUILD_TYPE in
    rust)
        print_status "Building Rust library..."
        cargo build --release
        print_status "Rust library built successfully!"
        print_status "Output: target/release/libdevcycle_bucketing_rs.rlib"
        ;;

    ffi|c)
        print_status "Building C library (FFI)..."
        cargo build --release --features ffi
        print_status "C library built successfully!"

        # Generate C header if cbindgen is available
        if command -v cbindgen &> /dev/null; then
            print_status "Generating C header file..."
            cbindgen --config cbindgen.toml --crate devcycle-bucketing-rs --output devcycle_bucketing.h
            print_status "Header file generated: devcycle_bucketing.h"
        else
            print_warning "cbindgen not found. Install with: cargo install cbindgen"
            print_warning "Run manually: cbindgen --config cbindgen.toml --crate devcycle-bucketing-rs --output devcycle_bucketing.h"
        fi

        # Show platform-specific library location
        if [[ "$OSTYPE" == "darwin"* ]]; then
            print_status "Output: target/release/libdevcycle_bucketing_rs.dylib"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            print_status "Output: target/release/libdevcycle_bucketing_rs.so"
        elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
            print_status "Output: target/release/devcycle_bucketing_rs.dll"
        fi
        ;;

    wasm)
        # Check if wasm-pack is installed
        if ! command -v wasm-pack &> /dev/null; then
            print_error "wasm-pack not found. Install with:"
            echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
            exit 1
        fi

        print_status "Building WebAssembly (web target)..."
        wasm-pack build --target web --features wasm --out-dir pkg-web

        print_status "Building WebAssembly (nodejs target)..."
        wasm-pack build --target nodejs --features wasm --out-dir pkg-node

        print_status "Building WebAssembly (bundler target)..."
        wasm-pack build --target bundler --features wasm --out-dir pkg-bundler

        print_status "WASM libraries built successfully!"
        print_status "Output directories:"
        print_status "  - pkg-web/ (for web browsers)"
        print_status "  - pkg-node/ (for Node.js)"
        print_status "  - pkg-bundler/ (for webpack, rollup, etc.)"
        ;;

    all)
        print_status "Building all library variants..."
        echo ""

        # Build Rust library
        print_status "1/3 Building Rust library..."
        cargo build --release
        echo ""

        # Build C library
        print_status "2/3 Building C library (FFI)..."
        cargo build --release --features ffi

        if command -v cbindgen &> /dev/null; then
            cbindgen --config cbindgen.toml --crate devcycle-bucketing-rs --output devcycle_bucketing.h 2>/dev/null || true
        fi
        echo ""

        # Build WASM
        if command -v wasm-pack &> /dev/null; then
            print_status "3/3 Building WebAssembly..."
            wasm-pack build --target web --features wasm --out-dir pkg-web 2>&1 | grep -v "warning" || true
            wasm-pack build --target nodejs --features wasm --out-dir pkg-node 2>&1 | grep -v "warning" || true
            wasm-pack build --target bundler --features wasm --out-dir pkg-bundler 2>&1 | grep -v "warning" || true
            echo ""
            print_status "All builds completed successfully!"
        else
            print_warning "Skipping WASM build (wasm-pack not installed)"
            print_status "Rust and C library builds completed successfully!"
        fi
        ;;

    clean)
        print_status "Cleaning build artifacts..."
        cargo clean
        rm -rf pkg-web pkg-node pkg-bundler
        rm -f devcycle_bucketing.h
        print_status "Clean complete!"
        ;;

    help|--help|-h)
        echo "Usage: $0 [BUILD_TYPE]"
        echo ""
        echo "Build types:"
        echo "  rust     - Build Rust library only"
        echo "  ffi|c    - Build C library with FFI bindings"
        echo "  wasm     - Build WebAssembly for web, Node.js, and bundlers"
        echo "  all      - Build all library variants (default)"
        echo "  clean    - Remove all build artifacts"
        echo "  help     - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0           # Build all variants"
        echo "  $0 rust      # Build only Rust library"
        echo "  $0 ffi       # Build only C library"
        echo "  $0 wasm      # Build only WASM"
        echo "  $0 clean     # Clean build artifacts"
        ;;

    *)
        print_error "Unknown build type: $BUILD_TYPE"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac

