# Help - show available commands
help:
  @just --list

# Development
run:
  rtk cargo run --bin main

release:
  rtk cargo run --release --bin main

planner:
  rtk cargo run --bin planner

# Testing
test:
  rtk cargo nextest run --no-fail-fast

test-all:
  rtk cargo nextest run -- --ignored

# Code Quality
lint:
    rtk cargo clippy --all-targets

fmt:
  rtk cargo +nightly fmt

fmt-check:
  rtk cargo +nightly fmt --check

# Build
build:
  rtk cargo build

build-release:
  rtk cargo build --release

# WASM
wasm-build:
  rtk cargo build --target wasm32-unknown-unknown --no-default-features

wasm-run:
  rtk cargo build --target wasm32-unknown-unknown --no-default-features
  rtk wasm-server-runner target/wasm32-unknown-unknown/debug/main.wasm

# Benchmarking (Research-specific)
bench:
  rtk cargo bench --bench in_epoch_benches

bench-all:
  rtk cargo bench

flamegraph:
  rtk CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin main --release --root

# Documentation
doc:
  rtk cargo doc --no-deps --open

doc-all:
  rtk cargo doc --open

# Maintenance
clean:
  rtk cargo clean
  rtk rm -rf results/*
  rtk rm -rf logs/*

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