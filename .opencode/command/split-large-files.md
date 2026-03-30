---
description: Split oversized source files into smaller, well-structured modules
---

Split source files that exceed a line-count threshold into multiple smaller, coherent modules.

The command argument may be a numeric line limit (e.g. `/split-large-files 500`) or omitted entirely.

## Phase 1 — Determine the threshold

**If the user supplied a number**, use that as the limit and skip to Phase 2.

**If no number was given**, analyse the distribution of source file sizes:

1. Collect line counts for every source file (`.rs`, or whatever language this project uses). Ignore generated files, vendored code, and lock files.
2. Compute and display a summary table:
   - Total source files
   - Min / p25 / median / p75 / p90 / p95 / max line counts
   - A small ASCII histogram bucketed by size range (e.g. 0–100, 101–200, 201–500, 501–1000, 1001+)
3. Propose a threshold based on the distribution — typically the p90 value rounded up to a round number — and explain your reasoning briefly.
4. **Ask the user to confirm or adjust the threshold before continuing.** Do not proceed to Phase 2 until you have a confirmed limit.

## Phase 2 — Identify candidates and build a plan

Find every source file whose line count exceeds the confirmed threshold. For each candidate, read just enough of the file (structure, top-level items) to propose a split — do **not** read every file upfront. Work through them one at a time, reading each file only when you are actively planning it.

For each candidate, note:
- File path and current line count
- Proposed split plan: names of new submodule files and which items would move to each

**Present a plan to the user and ask for explicit approval before making any changes.** Format the plan as a checklist, one checkbox per file that will be touched (both files being split and files that need import fixes). Example:

```
- [ ] src/foo.rs (820 lines) → split into foo/bar.rs, foo/baz.rs
- [ ] src/other.rs (import path fix)
```

The user may ask to exclude files, rename proposed modules, or adjust the split boundaries. Incorporate their feedback, then wait for a go-ahead.

## Phase 3 — Execute the refactor (only after approval)

Work through the approved checklist **sequentially, one file at a time**. Read a file only when you are about to work on it — do not pre-load multiple files into context.

For each file being split:

1. Read the file.
2. Create the new submodule files under a directory named after the parent file (e.g. `foo.rs` → `foo/bar.rs`, `foo/baz.rs`).
3. Move the identified items to their respective new files, preserving all doc comments, attributes, and `#[tracing::instrument]` spans.
4. Update the parent file (`foo.rs`) to declare the submodules with `pub mod bar;` / `mod baz;` and re-export any items that were previously public from the parent.
5. Use `rg` to find all files that import the moved items, then fix those import paths — reading each affected file only as you edit it.
6. Preserve all existing `use` statements within each moved item; add any additional imports required by the new file's scope.
7. **Check off this file in the plan** before moving to the next one.

## Phase 4 — Verify

After all files are split:

1. Run `cargo check` (or the project's equivalent type-check command) and fix any errors introduced by the refactor.
2. Run the test suite (`just test` for this project) and confirm all tests pass.
3. Report a summary: files split, new files created, any issues encountered.

## Rules

- **Never change computational behaviour.** This is a pure structural refactor — do not alter logic, algorithm, or API surfaces.
- Follow the project's module layout convention: `foo.rs` declares the module; `foo/` holds its submodules.
- Every public function in new files must have `#[tracing::instrument]` (enforced by clippy-tracing in this project).
- Do not use `unwrap()`. Propagate errors with `?` or use `expect()` only where the project rules allow it.
- Run `just fmt` after all edits to ensure import ordering and formatting are correct.
- If a file is already close to the threshold (within ~10 %), skip it rather than making a trivial split.
