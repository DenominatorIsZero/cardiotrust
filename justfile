wasm-build:
  cargo build --target wasm32-unknown-unknown --bin client

wasm-run: wasm-build
  wasm-server-runner '.\target\wasm32-unknown-unknown\debug\client.wasm'

r:
cargo run --bin main

alias w := wasm-run