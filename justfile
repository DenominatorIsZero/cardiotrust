run:
  cargo run --bin main

release:
  cargo run --release --bin main

test:
  cargo nextest run

test-all:
  cargo nextest run -- --ignored

lint:
    clippy-tracing --action check --exclude target --exclude benches
    cargo clippy

bench:
  cargo bench

work: lint test bench

flamegraph:
  CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin main --release --root


wasm-build:
  cargo build --target wasm32-unknown-unknown --bin client

wasm-run: wasm-build
  WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML='index.html' wasm-server-runner './target/wasm32-unknown-unknown/debug/client.wasm'

wasm-build-deploy:
  cargo build --release --target wasm32-unknown-unknown --bin client

wasm-deploy: wasm-build-deploy
  wasm-bindgen --no-typescript --target web --out-dir ./wasm-client/ --out-name "client" ./target/wasm32-unknown-unknown/release/client.wasm

alias d := wasm-deploy

alias w := wasm-run