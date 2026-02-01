# https://just.systems

# Show available recipes
default:
    @just --list

# ---------- Build ----------

# Build all crates (debug)
build:
    cargo build --workspace

# Build all crates (release)
build-release:
    cargo build --workspace --release

# Build daemon only (release)
build-daemon:
    cargo build -p charond --release

# Build client only (release)
build-client:
    cargo build -p charon-tui --release

# ---------- Cross-compile (RP5) ----------

rp5_target := "aarch64-unknown-linux-gnu"
rp5_host := env("RP5_HOST", "charon.local")

# Build for RP5 (uses zigbuild on macOS, native cross on Linux)
build-rp5:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo zigbuild --workspace --target {{rp5_target}} --release
    else
        cargo build --workspace --target {{rp5_target}} --release
    fi

# Build daemon for RP5
build-rp5-daemon:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo zigbuild -p charond --target {{rp5_target}} --release
    else
        cargo build -p charond --target {{rp5_target}} --release
    fi

# Build client for RP5
build-rp5-client:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo zigbuild -p charon-tui --target {{rp5_target}} --release
    else
        cargo build -p charon-tui --target {{rp5_target}} --release
    fi

# Deploy binaries to RP5 (set RP5_HOST env var or defaults to charon.local)
deploy-rp5: build-rp5
    scp target/{{rp5_target}}/release/charond {{rp5_host}}:~/.local/bin/
    scp target/{{rp5_target}}/release/charon-tui {{rp5_host}}:~/.local/bin/
    @echo "Deployed to {{rp5_host}}"

# ---------- Test ----------

# Run unit tests only (fast, no harness)
test:
    cargo test --workspace

# Run all tests including integration tests
test-all:
    cargo test -p charond --features testing
    cargo test -p charon-tui

# Run daemon tests with harness
test-daemon:
    cargo test -p charond --features testing

# Run a specific test by name
test-one NAME:
    cargo test -p charond --features testing {{NAME}}

# ---------- Quality ----------

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Run all checks (fmt + clippy)
check: fmt-check clippy

# ---------- Development ----------

# Watch and run checks on change
watch:
    bacon

# Watch and run tests on change
watch-test:
    bacon test

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# ---------- CI ----------

# Run full CI pipeline locally
ci: fmt-check clippy test-all
    @echo "CI passed!"
