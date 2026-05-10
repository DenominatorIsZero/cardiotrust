# Demo projects for WASM builds

Place demo project directories here. Each project should contain:
- scenario.toml  (TOML metadata)
- data.bin       (postcard-serialized Data)
- results.bin    (postcard-serialized Results)

Generate demo projects via:
  cargo run --bin main --features native
Then copy scenarios from results/ into here with a descriptive name.

Example structure:
  wasm-projects/
    handcrafted-heart/
      scenario.toml
      data.bin
      results.bin
    mri-pre-processed/
      scenario.toml
      data.bin
      results.bin

Data files (.bin) are gitignored to keep repo size small.
They are embedded at WASM compile time via include_dir!.

