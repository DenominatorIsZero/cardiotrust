## 1. Promote Spec Files to openspec/specs/

- [x] 1.1 Move `specs/configuration/spec.md` to `openspec/specs/configuration/spec.md`
- [x] 1.2 Move `specs/data-simulation/spec.md` to `openspec/specs/data-simulation/spec.md`
- [x] 1.3 Move `specs/cardiac-model/spec.md` to `openspec/specs/cardiac-model/spec.md`
- [x] 1.4 Move `specs/inverse-algorithm/spec.md` to `openspec/specs/inverse-algorithm/spec.md`
- [x] 1.5 Move `specs/scenario-lifecycle/spec.md` to `openspec/specs/scenario-lifecycle/spec.md`
- [x] 1.6 Move `specs/scheduler/spec.md` to `openspec/specs/scheduler/spec.md`
- [x] 1.7 Move `specs/visualization/spec.md` to `openspec/specs/visualization/spec.md`
- [x] 1.8 Move `specs/ui-navigation/spec.md` to `openspec/specs/ui-navigation/spec.md`

## 2. Validate Spec Content

- [x] 2.1 Verify each spec file uses only `### Requirement:` and `#### Scenario:` headers (no implementation details)
- [x] 2.2 Confirm all scenario blocks use WHEN/THEN format with `####` (four hashtags)
- [x] 2.3 Confirm all normative statements use SHALL/MUST language
- [x] 2.4 Confirm no spec file references internal type names, file paths, or encoding details
- [x] 2.5 Run `openspec status` to confirm all 8 capability specs are recognized

## 3. Review and Refine Per Domain

- [x] 3.1 Review `configuration` spec — verify propagation velocity, control function, and anatomy source scenarios are accurate
- [x] 3.2 Review `data-simulation` spec — verify determinism, measurement shape, and activation time requirements are accurate
- [x] 3.3 Review `cardiac-model` spec — verify state index invariant, connectivity rules, stability, and GPU parity requirements
- [x] 3.4 Review `inverse-algorithm` spec — verify epoch-zero, beat randomization, divergence halt, clamping, metrics sweep, and pseudo-inverse requirements
- [x] 3.5 Review `scenario-lifecycle` spec — verify state machine transitions, config lock, unification, persistence, and progress requirements
- [x] 3.6 Review `scheduler` spec — verify three-state machine, terminal state guarantee, non-blocking relay, and persistence order requirements
- [x] 3.7 Review `visualization` spec — verify connectable-only rendering, scene rebuild, color modes, cutting plane, sensor beat positions, and relative scaling requirements
- [x] 3.8 Review `ui-navigation` spec — verify single active view, precondition gating, data preloading, Explorer list, Scenario editor lock, image caching, and scheduler controls

## 4. Link Specs to Existing Tests (Optional)

- [x] 4.1 Search the test suite for tests that exercise invariants in `configuration` spec; annotate scenarios with test references in comments if desired
- [x] 4.2 Search the test suite for tests covering `data-simulation` determinism and shape invariants; annotate where applicable
- [x] 4.3 Search the test suite for tests covering `cardiac-model` connectivity and stability invariants; annotate where applicable
- [x] 4.4 Search the test suite for tests covering `inverse-algorithm` epoch-zero and clamping behavior; annotate where applicable
- [x] 4.5 Search the test suite for tests covering `scenario-lifecycle` state machine; annotate where applicable
