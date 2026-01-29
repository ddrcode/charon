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

# ---------- Test ----------

# Run unit tests only (fast, no harness)
test:
    cargo test --workspace

# Run all tests including integration tests
test-all:
    cargo test -p charond --features testing
    cargo test -p charon-tui
    cargo test -p charon-lib

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
