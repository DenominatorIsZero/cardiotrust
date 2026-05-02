# OpenSpec Domain Documentation Format

Put stable domain language, behavior, and invariants into the relevant capability spec at `openspec/specs/<capability>/spec.md`. Put in-flight rationale or terminology trade-offs into the active change's `proposal.md` or `design.md`.

## Stable spec structure

```md
## Purpose

{One or two paragraphs describing the capability in domain language, what it governs, and how it differs from adjacent capabilities.}

## ADDED Requirements

### Requirement: {Canonical behavior or invariant}

{Describe the behavior or invariant in domain language. State what the system guarantees, not how it is implemented.}

#### Scenario: {Concrete example}

- **WHEN** {trigger or condition}
- **THEN** {observable outcome}
```

## In-flight change structure

When the terminology or behavior is still being shaped inside a change, capture the rationale in the active change folder:

```md
## Why

{Why this change or clarification matters now.}

## What Changes

 - {Capability or terminology change}

## Design

### Decision: {Short title}

{Context, choice, and why this is the right trade-off.}
```

## Rules

- **Be opinionated.** When multiple words exist for the same concept, pick the best one and list the others as aliases to avoid.
- **Put stable language in the right capability spec.** Sharpen `## Purpose`, requirement titles, and scenario wording instead of maintaining a separate glossary file.
- **Record rationale in change docs, not in specs.** Use `proposal.md` and `design.md` for why, trade-offs, and rejected alternatives.
- **Keep definitions behavioral.** Specs define what the system guarantees, not implementation details, file paths, or technology choices.
- **Show boundaries explicitly.** Use `## Purpose` to distinguish one capability from nearby capabilities when terminology could drift.
- **Flag conflicts explicitly.** If a term is overloaded, call out the conflict and resolve it in the active change docs or by tightening spec wording.
- **Create new specs lazily.** If no capability spec exists yet, introduce it through an active change at `openspec/changes/<change>/specs/<capability>/spec.md`.
