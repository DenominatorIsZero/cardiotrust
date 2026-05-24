# Help - show available commands
help:
  @just --list

# Development
run:
  cargo run --bin main

release:
  cargo run --release --bin main

planner:
  cargo run --bin planner

# Testing
test:
  cargo nextest run --no-fail-fast

test-all:
  cargo nextest run -- --ignored

# Code Quality
lint:
    cargo clippy --all-targets

fmt:
  cargo +nightly fmt

fmt-check:
  cargo +nightly fmt --check

# Build
build:
  cargo build

build-release:
  cargo build --release

# WASM
wasm-build:
  cargo build --target wasm32-unknown-unknown --no-default-features

wasm-run:
  cargo build --target wasm32-unknown-unknown --no-default-features
  npx wasm-server-runner target/wasm32-unknown-unknown/debug/cardiotrust.wasm

# Benchmarking (Research-specific)
bench:
  cargo bench --bench in_epoch_benches

bench-all:
  cargo bench

flamegraph:
  CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin main --release --root

# Documentation
doc:
  cargo doc --no-deps --open

doc-all:
  cargo doc --open

# Maintenance
clean:
  cargo clean
  rm -rf results/*
  rm -rf logs/*

# Comprehensive check - everything including tests, benches, examples
check:
  @echo "🔍 Running comprehensive cargo check..."
  rtk cargo check --workspace --all-targets --all-features
  @echo "🔍 Running comprehensive clippy..."
  rtk cargo clippy --workspace --all-targets --all-features -- -D warnings

# Combined workflows
work: check test bench

ci: fmt-check check test

dev: build test