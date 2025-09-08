run:
  cargo run --bin main

release:
  cargo run --release --bin main

test:
  cargo nextest run --no-fail-fast

test-all:
  cargo nextest run -- --ignored

lint:
    clippy-tracing --action check --exclude target --exclude benches
    cargo clippy

bench:
  cargo bench --bench in_epoch_benches

work: lint test bench

fmt:
  cargo +nightly fmt

flamegraph:
  CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin main --release --root