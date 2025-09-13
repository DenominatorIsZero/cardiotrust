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
    cargo clippy

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

check:
  cargo check --all-targets

# Combined workflows
work: lint test bench

ci: fmt-check lint test

dev: build test