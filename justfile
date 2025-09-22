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
    clippy-tracing --action check --exclude target --exclude benches
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
  cargo check --workspace --all-targets --all-features
  @echo "🔍 Running comprehensive clippy..."
  cargo clippy --workspace --all-targets --all-features -- -D warnings

# Combined workflows
work: check test bench

ci: fmt-check check test

dev: build test