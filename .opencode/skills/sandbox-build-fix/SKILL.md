---
name: sandbox-build-fix
description: Fix build/test failures in the Linux Docker sandbox when the host is macOS. Use when cargo build or cargo nextest fails with linker errors, anon symbol errors, OOM kills, or stale artifact issues.
---

# Skill: sandbox-build-fix

Use this skill when `cargo build`, `cargo test`, or `cargo nextest` fails in the sandbox.

## Environment facts

- **Host**: macOS M2 (aarch64-apple-darwin)
- **Sandbox**: Linux aarch64 (aarch64-unknown-linux-gnu), 3.8 GB RAM, no swap, 12 cores
- **Files are synced** between host and sandbox — the project `target/` dir is shared and contains macOS-compiled artifacts. Never run `cargo clean` from the project dir; it would wipe the host's build cache too.
- **Cargo config**: `~/.cargo/config.toml` (sandbox-only, not synced). Sets `target-dir = "/home/agent/cargo-target"` so sandbox builds go to a separate location.
- **Cargo/rustc**: lives in `~/.rustup/`, not on `$PATH` by default. Always prefix commands with: `export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"`

## Known failure modes

### 1. Linker errors: `hidden symbol` / `undefined reference to anon.*`

**Symptom:**
```
/usr/bin/ld: ...: hidden symbol `anon.<hash>.<N>.llvm.<hash>' isn't defined
/usr/bin/ld: final link failed: bad value
```

**Cause:** `/home/agent/cargo-target/` contains rlibs compiled on macOS. They embed macOS LLVM `anon.*` private symbols. When the Linux-compiled test CGUs reference those symbols across rlib boundaries, the linker can't resolve them.

**Fix:** Wipe the sandbox-only build cache (safe — does NOT touch the synced project `target/`):
```bash
export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"
cargo clean
```
This removes `/home/agent/cargo-target/` contents. The next build will recompile everything for Linux. Takes ~8 minutes with `jobs = 4`.

### 2. OOM — rustc processes killed (SIGKILL / signal 9)

**Symptom:**
```
process didn't exit successfully: `rustc ...` (signal: 9, SIGKILL: kill)
error: could not compile `bevy_ecs` (lib)
```

**Cause:** Too many parallel jobs × large crates (bevy_ecs, moxcms) exhaust the 3.8 GB RAM.

**Fix:** Check and cap `jobs` in `~/.cargo/config.toml`:
```bash
cat ~/.cargo/config.toml
```
It should read:
```toml
[build]
target-dir = "/home/agent/cargo-target"
jobs = 4
```
`jobs = 4` is the safe limit for this sandbox. Do NOT use `codegen-units=1` — that causes even worse OOM by concentrating all code into one giant object file.

If the file is missing or has wrong values:
```bash
cat > ~/.cargo/config.toml << 'EOF'
[build]
target-dir = "/home/agent/cargo-target"
jobs = 4
EOF
```

### 3. `cargo nextest list` or `cargo nextest run` times out immediately

**Cause:** Either cargo is rebuilding from scratch (slow), or `~/.cargo/config.toml` is missing / has bad values (`jobs = 1`, wrong `target-dir`, or a stale `CARGO_MANIFEST_DIR` pointing to a macOS path).

**Fix:**
1. Check `~/.cargo/config.toml` — remove any `CARGO_MANIFEST_DIR` override under `[env]`.
2. Check if `/home/agent/cargo-target/debug/deps/` has Linux binaries: `file /home/agent/cargo-target/debug/deps/cardiotrust-*` (should say `ELF`, not `Mach-O`).
3. If Mach-O or missing: run `cargo clean` (sandbox-safe) then rebuild.

### 4. `cannot execute binary file: Exec format error`

**Cause:** The project-level `target/` dir (synced from macOS) contains macOS Mach-O binaries. These can't run in Linux.

**Fix:** Always use the binaries in `/home/agent/cargo-target/`, never in `./target/`. The `target-dir` in `~/.cargo/config.toml` ensures this automatically.

## Canonical working state

```toml
# ~/.cargo/config.toml  (sandbox-only, never synced)
[build]
target-dir = "/home/agent/cargo-target"
jobs = 4
```

No `[env]` section. No `CARGO_MANIFEST_DIR`.

## Running tests

```bash
export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"
cargo nextest run --no-fail-fast
```

Expected: all non-ignored tests pass in ~12 seconds after a warm build.

## Full recovery procedure (start here if unsure)

```bash
export PATH="/home/agent/.cargo/bin:/home/agent/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/bin:$PATH"

# 1. Ensure correct cargo config (sandbox-only)
cat > ~/.cargo/config.toml << 'EOF'
[build]
target-dir = "/home/agent/cargo-target"
jobs = 4
EOF

# 2. Wipe stale sandbox build cache (safe - does NOT touch synced project target/)
cargo clean

# 3. Rebuild (~8 min) and run tests
cargo nextest run --no-fail-fast
```
