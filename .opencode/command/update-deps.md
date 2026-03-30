---
description: Update all Rust dependencies and migrate breaking changes
---

Update all dependencies in Cargo.toml to their latest compatible versions, apply any necessary code migrations, and verify all tests pass.

## Context from previous runs

### Sandbox constraints
- The Linux sandbox has **3.9 GB RAM, no swap**. `cargo nextest run` OOM-kills the linker on large Bevy+WGPU binaries.
- Fix: install `mold` and configure it in `~/.cargo/config.toml`. Always do this before running tests:
  ```bash
  sudo apt-get install -y mold
  cat > ~/.cargo/config.toml << 'EOF'
  [build]
  target-dir = "/home/agent/cargo-target"
  jobs = 1

  [target.aarch64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]

  [profile.test]
  opt-level = 0
  debug = 0
  EOF
  ```
- **Never `cargo clean`** the project `target/` dir — it's shared with host macOS. Only `cargo clean` from within the sandbox (it uses `/home/agent/cargo-target`).
- After cleaning the sandbox build cache, a full rebuild takes ~8 minutes.

### Known dependency constraints

**bincode**: Do NOT upgrade past 2.0.1. `bincode 3.0.0` is a tombstone/abandoned crate that only emits a compiler error.

**egui ecosystem versioning quirk**: The `egui_plot` crate is versioned offset by +1 from `egui`:
- `egui 0.32` ↔ `egui_plot 0.33`
- `egui 0.33` ↔ `egui_plot 0.34`
- `egui 0.34` ↔ `egui_plot 0.35`

`bevy_egui` pins a specific `egui` version. Always align `egui`, `egui_extras`, and `egui_plot` to match what `bevy_egui` requires — otherwise you get "multiple versions of crate `egui`" mismatched-types errors. Use `cargo tree -i "egui@X.Y.Z"` to diagnose which version conflict exists.

## Steps

1. **Check what's outdated**

   Fetch latest versions from crates.io for all direct dependencies in `Cargo.toml`. Use the crates.io API: `https://crates.io/api/v1/crates/<name>` and extract `newest_version`. Check in parallel for all deps.

2. **Update `Cargo.toml`**

   Bump versions, respecting the constraints above. Key things to verify:
   - `egui` / `egui_extras` / `egui_plot` must all target the same egui version (the one `bevy_egui` requires)
   - Keep `bincode` at `2.0.1`
   - Check `bevy_egui`, `bevy_editor_cam`, `bevy_obj` compatibility with the new `bevy` version using their dependency manifests: `https://crates.io/api/v1/crates/<name>/<version>/dependencies`

3. **Consult migration guides for major version bumps**

   For each major Bevy version crossed, read `https://bevyengine.org/learn/migration-guides/X-Y-to-X-Z/` and look for breaking changes affecting this codebase. Key areas to check:
   - Event/Message API changes
   - UI component changes (Node fields, BorderRadius, BorderColor, etc.)
   - Light/Camera component vs resource changes
   - Import path changes in render crates

4. **Apply code migrations**

   Use grep to find all affected patterns, then fix them. Common patterns:
   ```bash
   grep -rn "EventReader\|EventWriter\|derive(Event)\|add_event" src/
   grep -rn "BorderColor(\|BorderRadius::\|border\.0\|insert_resource(AmbientLight" src/
   grep -rn "bevy::render::render_asset::RenderAssetUsages" src/
   ```

5. **Set up sandbox and run `cargo check`**

   ```bash
   export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"
   # Set up mold + config (see Sandbox constraints above)
   cargo check 2>&1 | grep "^error"
   ```

   Iterate until zero errors. Use the full error output to find remaining issues — the compiler often suggests the exact fix.

6. **Run tests**

   ```bash
   export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"
   cargo nextest run --no-fail-fast 2>&1 | tail -20
   ```

   Expected: all non-ignored tests pass. The `just test` command is the canonical test runner.
