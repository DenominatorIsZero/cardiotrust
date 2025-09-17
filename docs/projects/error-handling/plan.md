# Implementation Plan: Error Handling Standardization

Replace all unwrap/expect calls with anyhow::Result types and eliminate application crashes through systematic error handling improvements.

## Tasks

### 1. Prerequisites & Setup

**Status**: [x] Complete
**Dependencies**: None

**Implementation Steps**:

- [x] Add anyhow dependency to Cargo.toml
- [ ] Add anyhow prelude imports to lib.rs (deferred to later tasks)
- [x] Define common error handling patterns for research software
- [x] Document error handling strategy in CLAUDE.md

### 2. Core Data & Model Foundation

**Status**: [x] Completed
**Dependencies**: Task 1 complete

**Implementation Steps**:

- [x] `src/core/data.rs` (1 unwrap)
- [x] `src/core/data/simulation.rs` (1 unwrap)
- [x] `src/core/data/shapes.rs` (39 unwraps)
- [x] `src/core/model.rs` (1 unwrap)
- [x] `src/core/model/spatial.rs` (9 unwraps)
- [x] `src/core/model/spatial/nifti.rs` (10 unwraps)
- [x] `src/core/model/spatial/sensors.rs` (5 unwraps)
- [x] `src/core/model/spatial/voxels.rs` (19 unwraps)
- [x] `src/core/model/functional.rs` (2 unwraps)
- [x] `src/core/model/functional/allpass.rs` (44 unwraps)
- [x] `src/core/model/functional/allpass/delay.rs` (11 unwraps)
- [x] `src/core/model/functional/allpass/shapes.rs` (already converted - only proper error handling patterns remain)
- [x] `src/core/model/functional/control.rs` (8 unwraps converted to Result types with context)
- [x] `src/core/model/functional/measurement.rs` (5 unwraps converted to Result types with physics-accurate error handling)
- [x] `src/core/config.rs` (already clean - no unwrap patterns found)
- [x] `src/core/config/algorithm.rs` (already clean - configuration-only code)
- [x] `src/core/config/model.rs` (already clean - pure data structures)
- [x] `src/core/config/simulation.rs` (already clean - simple Default implementations)

### 3. Core Algorithms

**Status**: [ ] Pending
**Dependencies**: Task 2 complete

**Implementation Steps**:

- [x] `src/core/algorithm.rs` (14 unwraps)
- [x] `src/core/algorithm/estimation.rs` (3 unwraps)
- [x] `src/core/algorithm/estimation/prediction.rs` (1 unwrap)
- [x] `src/core/algorithm/refinement.rs`
- [x] `src/core/algorithm/refinement/derivation.rs` (20 unwraps)
- [x] `src/core/algorithm/refinement/update.rs` (4 unwraps)
- [x] `src/core/algorithm/metrics.rs` (25 unwraps)
- [x] `src/core/algorithm/gpu.rs` (13 unwraps)
- [ ] `src/core/algorithm/gpu/derivation.rs` (63 unwraps - critical GPU operations)
- [ ] `src/core/algorithm/gpu/epoch.rs` (13 unwraps)
- [ ] `src/core/algorithm/gpu/helper.rs` (6 unwraps)
- [ ] `src/core/algorithm/gpu/metrics.rs` (13 unwraps)
- [ ] `src/core/algorithm/gpu/prediction.rs` (27 unwraps)
- [ ] `src/core/algorithm/gpu/reset.rs` (22 unwraps)
- [ ] `src/core/algorithm/gpu/update.rs` (31 unwraps)

### 4. Scenario Management

**Status**: [ ] Pending
**Dependencies**: Task 3 complete

**Implementation Steps**:

- [ ] `src/core/scenario.rs` (30 unwraps + 2 critical panic! calls in loading)
- [ ] `src/core/scenario/results.rs` (9 unwraps)
- [ ] `src/core/scenario/summary.rs`

### 5. Scheduler & Application Control

**Status**: [ ] Pending
**Dependencies**: Task 4 complete

**Implementation Steps**:

- [ ] `src/scheduler.rs` (3 unwraps + 3 critical panic! calls in thread management)
- [ ] `src/lib.rs` (3 unwraps in scenario loading)

### 6. Visualization & UI

**Status**: [ ] Pending
**Dependencies**: Task 5 complete

**Implementation Steps**:

- [ ] `src/ui.rs`
- [ ] `src/ui/results.rs` (20 unwraps)
- [ ] `src/ui/explorer.rs` (2 unwraps)
- [ ] `src/ui/topbar.rs` (5 unwraps)
- [ ] `src/ui/vol.rs` (10 unwraps)
- [ ] `src/ui/scenario.rs` (9 unwraps)
- [ ] `src/ui/scenario/algorithm.rs`
- [ ] `src/ui/scenario/common.rs` (7 unwraps)
- [ ] `src/ui/scenario/data.rs`
- [ ] `src/vis.rs`
- [ ] `src/vis/heart.rs` (17 unwraps)
- [ ] `src/vis/sensors.rs` (2 unwraps)
- [ ] `src/vis/sample_tracker.rs` (1 unwrap)
- [ ] `src/vis/plotting.rs`
- [ ] `src/vis/plotting/gif/states.rs` (4 unwraps)
- [ ] `src/vis/plotting/gif/matrix.rs` (2 unwraps)
- [ ] `src/vis/plotting/png/delay.rs` (3 unwraps)
- [ ] `src/vis/plotting/png/propagation_speed.rs` (3 unwraps)
- [ ] `src/vis/plotting/png/states.rs` (26 unwraps)
- [ ] `src/vis/plotting/png/activation_time.rs` (6 unwraps)
- [ ] `src/vis/plotting/png/voxel_type.rs` (6 unwraps)
- [ ] `src/vis/plotting/png/line.rs` (13 unwraps)
- [ ] `src/vis/plotting/png/matrix.rs` (9 unwraps)

### 7. Application Entry Points

**Status**: [ ] Pending
**Dependencies**: Task 6 complete

**Implementation Steps**:

- [ ] `src/bin/main.rs` (3 unwraps - git hash retrieval and logging setup)
- [ ] `src/bin/planner.rs` (11 unwraps)

### 8. Validation & Integration

**Status**: [ ] Pending
**Dependencies**: Task 7 complete

**Implementation Steps**:

- [ ] Run `just work` to verify all tests pass
- [ ] Run benchmarks to ensure no performance degradation
- [ ] Test GPU fallback error scenarios with graceful degradation
- [ ] Verify error messages provide actionable context for debugging
- [ ] Test critical error paths (scenario loading, GPU initialization)
- [ ] Update CLAUDE.md with new error handling patterns

## Final Validation

- [ ] All critical panic! calls eliminated from scheduler and scenario loading
- [ ] GPU operations fail gracefully with CPU fallback where possible
- [ ] Error messages provide rich context for debugging research scenarios
- [ ] Algorithm correctness preserved - no behavioral changes
- [ ] Performance benchmarks show no degradation
- [ ] Two-run testing still works with improved error reporting
