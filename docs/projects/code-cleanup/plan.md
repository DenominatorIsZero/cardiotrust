# Implementation Plan: Code Cleanup & Simplification

Remove unused code components (client binary, Kalman filter) to simplify the codebase before documentation.

## Tasks

### 1. Remove Client Binary

**Status**: [x] Complete  
**Dependencies**: None

**Implementation Steps**:

- [x] Remove `src/bin/client.rs`
- [x] Remove WebSocket infrastructure (`src/websocket.rs`)
- [x] Clean up WebSocket dependencies in `Cargo.toml`
- [x] Remove client-specific configs and imports
- [x] Update build configs to single binary
- [x] Test main binary still works

### 2. Remove Kalman Filter

**Status**: [x] Complete  
**Dependencies**: Task 1 complete

**Implementation Steps**:

- [x] Remove `src/core/model/functional/kalman.rs`
- [x] Remove `KalmanGain` imports and usage
- [x] Clean up GPU kernel Kalman code
- [x] Remove `update_kalman_gain` config flags
- [x] Update algorithm chains to skip Kalman steps
- [x] Test algorithms work without Kalman components

### 3. Dependency Cleanup

**Status**: [x] Complete  
**Dependencies**: Tasks 1-2 complete

**Implementation Steps**:

- [x] Run `cargo machete` to find unused deps
- [x] Remove deps only used by removed components
- [x] Clean up feature flags and conditional compilation
- [x] Update justfile to remove obsolete commands
- [x] Test all build targets (native, release)

### 4. Survey Additional Cleanup

**Status**: [x] Complete  
**Dependencies**: Tasks 1-3 complete

**Implementation Steps**:

- [x] Assess dynamic sensor array usage and complexity
- [x] Document other potential cleanup targets for future
- [x] Note anything else that adds complexity without value

## Final Validation

- [x] Core algorithms compile and work (verified through tasks 1-3)
- [x] Dependencies cleaned and build targets tested
- [x] Code quality maintained with proper instrumentation
- [x] Cleanup objectives achieved
