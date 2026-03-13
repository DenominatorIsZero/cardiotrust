## Why

CardioTrust has no machine-readable behavioral specifications, making it hard to reason about domain contracts, validate correctness after refactoring, or onboard contributors to the domain model. Establishing an initial spec layer creates a stable, implementation-independent reference for all seven major domain areas.

## What Changes

- Add specification files for every top-level domain area in the codebase
- No code changes; no breaking changes to any API or behavior

## Capabilities

### New Capabilities

- `configuration`: Specification for the declarative configuration layer — how experiments are parameterized, what values are valid, and what constraints Config enforces.
- `data-simulation`: Specification for forward cardiac simulation — what the data layer produces, its determinism guarantees, and the shapes and semantics of its outputs.
- `cardiac-model`: Specification for the heart electrophysiology model — voxel anatomy, connectivity rules, filter network construction, Biot-Savart encoding, and GPU mirroring invariants.
- `inverse-algorithm`: Specification for the inverse estimation algorithm — the epoch loop, gradient descent, metrics, and GPU/CPU parity contract.
- `scenario-lifecycle`: Specification for the scenario entity — its lifecycle state machine, persistence contract, progress reporting, and execution dispatch.
- `scheduler`: Specification for concurrent simulation scheduling — how scenarios are queued, concurrency limits enforced, and completion detected.
- `visualization`: Specification for the 3-D volumetric visualization layer — scene construction, color-mode semantics, animation, and cutting-plane behavior.
- `ui-navigation`: Specification for the desktop UI shell — page routing, explorer, scenario editor, results viewer, and topbar enablement rules.

### Modified Capabilities

## Impact

- No production code changes; documentation and specification layer only.
- Affects `openspec/specs/` directory tree (new files).
- Provides a stable reference contract for all future refactoring, GPU kernel parity checks, and test coverage decisions.
