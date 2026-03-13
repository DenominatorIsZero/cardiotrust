## Context

CardioTrust is a Rust application for cardiac electrophysiology simulation and inverse estimation. Its source is organized into eight distinct domain areas — configuration, data simulation, cardiac model, inverse algorithm, scenario lifecycle, scheduling, 3-D visualization, and UI navigation — each with well-defined behavioral contracts. None of these domains currently have machine-readable specifications, making behavioral intent dependent on reading implementation code and comments.

The immediate impetus is the need for a stable reference layer that:
- Survives refactoring without the contracts being silently lost
- Allows contributors to understand guarantees without reading GPU kernels or numerical routines
- Provides testable WHEN/THEN scenarios that can be mapped to integration tests

## Goals / Non-Goals

**Goals:**
- Produce one `openspec/specs/<domain>/spec.md` file per domain area
- Cover domain-level invariants, inputs/outputs, and behavioral scenarios
- Express contracts that are stable under implementation changes (algorithm internals, data structures, encoding choices)
- Keep spec language at the level of observable behavior and domain concepts

**Non-Goals:**
- Specifying internal implementation details (data structures, library choices, encoding formats)
- Covering every method or function — only the externally observable, caller-relied-upon contracts
- Writing test code or modifying any production source
- Specifying GPU kernel internals beyond the parity contract with CPU

## Decisions

### One spec file per top-level domain area
The eight top-level domain areas map cleanly to eight spec files. Each domain has a clear owner (a module subtree) and a distinct set of stakeholders (algorithm users, UI layer, scheduler, visualization). Grouping by domain rather than by file keeps specs refactoring-stable.

**Alternative considered:** One monolithic spec file. Rejected — it would conflate contracts from unrelated subsystems and become unmaintainable as the codebase grows.

### Spec language: behavior and invariants only
Specs will use normative SHALL/MUST language for invariants and WHEN/THEN for scenarios, explicitly avoiding implementation terms (no struct names, no file paths, no encoding choices). This ensures specs remain valid under refactoring.

**Alternative considered:** Auto-generate from code comments. Rejected — code comments drift and mix implementation notes with intent.

### Capability mapping follows proposal's eight domains
The proposal identifies eight capabilities: `configuration`, `data-simulation`, `cardiac-model`, `inverse-algorithm`, `scenario-lifecycle`, `scheduler`, `visualization`, `ui-navigation`. These become the eight spec directories. The naming is stable and matches the mental model of domain experts.

## Risks / Trade-offs

- [Risk: Spec completeness] Early specs may miss edge cases or subtle invariants not surfaced during exploration → Mitigation: specs are living documents; initial coverage targets the most caller-relied-upon contracts, with gaps identified explicitly in each spec.
- [Risk: Spec drift] Implementation changes may invalidate specs without updating them → Mitigation: the openspec workflow gates implementation tasks on specs, creating a natural review checkpoint.
- [Risk: Over-specification] Capturing implementation details as requirements constrains future refactoring → Mitigation: strict rule enforced during spec authoring: no internal names, no encoding details, no library references.

## Open Questions

None. All eight domain areas are well-understood from codebase exploration. Spec content can be authored directly.
