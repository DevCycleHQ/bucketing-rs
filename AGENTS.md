# Project Coding Standards and Conventions

## Configuration and Tooling

- The project root contains configuration files for building, testing, and formatting.
- Key files include:
  - `Cargo.toml`: Package configuration, dependencies, and build targets
  - `Cargo.lock`: Locked dependency versions (committed to git)
  - `build.sh`: Multi-platform build script for Rust/FFI/WASM variants

## Project Structure

- Multi-platform Rust library supporting three output formats: native Rust (`rlib`), C FFI (`cdylib`/`staticlib`), and WebAssembly
- Source code organized in `src/` directory with test files alongside implementation
- Test resources in `tests/resources/`
- Build artifacts in `target/` (git-ignored)
- Planning documents in `.plans/` (git-ignored)

### Main Source Files

- `src/lib.rs`: Main library entry point
- `src/*_tests.rs`: Test files (always separate from implementation)
- Core modules: `bucketing`, `config`, `configmanager`, `event_queue`, `filters`, `user`, `platform_data`

### Build Targets

- **Rust library**: Standard `rlib` for Rust projects
- **C library (FFI)**: `cdylib` and `staticlib` for C/C++ interop (requires `ffi` feature)
- **WebAssembly**: Multiple WASM packages for web/node/bundler (requires `wasm` feature)

## Formatting

- **Always** run `cargo fmt` before committing
- Defer all formatting to `rustfmt` (configured via `rustfmt.toml` if present, otherwise uses defaults)
- Formatting is enforced, not optional

## Testing

- Tests should **always** be in their own files with `_tests.rs` suffix
- Test files located alongside source files in `src/`
- Test resources in `tests/resources/`
- Run tests with `cargo test` or feature-specific tests with `cargo test --features ffi`
- WASM tests use `wasm-pack test --node --features wasm`

## Git Commit Message Conventions

- Follow Conventional Commits specification: `<type>: <description>`
- Valid types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
- Description should be imperative mood, single sentence
- Branch names: Use `-` separator (not `/`), prefix with commit type (e.g., `feat-add-bucketing`, `fix-wasm-bindings`)
- **Examples**: `feat: add WASM bindings`, `fix: correct bucketing logic for edge case`, `test: add murmurhash test cases`

## Naming Conventions

- Files: Use snake_case (Rust convention)
- Modules: Use snake_case
- Structs/Enums/Traits: Use PascalCase
- Functions/Variables: Use snake_case
- Constants: Use SCREAMING_SNAKE_CASE

## Build Instructions

- See `BUILD.md` for detailed build instructions
- Use `build.sh` script for multi-platform builds:
  - `./build.sh all`: Build all variants
  - `./build.sh rust`: Rust library only
  - `./build.sh ffi`: C library only
  - `./build.sh wasm`: WebAssembly only
- Direct cargo commands:
  - `cargo build --release`: Standard Rust build
  - `cargo build --release --features ffi`: With FFI support
  - `cargo build --release --features wasm`: With WASM support

## Planning Documents

- If generating any planning documents, use the `.plans/` folder (git-ignored)
- Keep architectural decisions and design notes there

## Aviator CLI Workflow (optional)

- Use Aviator CLI (`av`) for managing stacked branches: `av branch chore-fix-invalid-input`
- Sync and push changes: `av sync --push=yes`
- Create PR: `av pr --title "<title>" --body "<body>"`
  - title follows Conventional Commits, body uses markdown/bullets, `av pr` will push the branch
- GitHub PR descriptions should be short and mainly focus on the reasons the changes were made in this PR, with minimal additional descriptions about testing state and listing the changes made.
