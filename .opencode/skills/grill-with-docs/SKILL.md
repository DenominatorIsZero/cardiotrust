---
name: grill-with-docs
description: Grilling session that challenges your plan against the existing domain model, sharpens terminology, and updates OpenSpec artifacts (capability specs, proposals, designs) inline as decisions crystallise. Use when user wants to stress-test a plan against their project's language and documented decisions.
---

<what-to-do>

Interview me relentlessly about every aspect of this plan until we reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one. For each question, provide your recommended answer.

Ask the questions one at a time, waiting for feedback on each question before continuing.

If a question can be answered by exploring the codebase, explore the codebase instead.

</what-to-do>

<supporting-info>

## Domain awareness

During codebase exploration, also look for existing OpenSpec artifacts:

### File structure

This repo records stable behavior in capability specs and in-flight decisions in change folders:

```
/
└── openspec/
    ├── specs/
    │   ├── cardiac-model/spec.md
    │   └── scenario-storage/spec.md
    └── changes/
        ├── some-change/
        │   ├── proposal.md
        │   ├── design.md
        │   ├── tasks.md
        │   └── specs/
        │       └── scenario-storage/spec.md
        └── archive/
```

Use `openspec/specs/<capability>/spec.md` as the stable source of truth for domain language, behavior, and invariants. Use `openspec/changes/<change>/proposal.md` and `design.md` for in-flight decisions and trade-offs. Archived changes under `openspec/changes/archive/` are historical context.

Create files lazily — only when you have something to write. Prefer updating an existing capability spec first. Create a new change under `openspec/changes/<change>/` only when a new proposal or design decision needs to be recorded.

## During the session

### Challenge against the specs

When the user uses a term that conflicts with the existing language in the relevant `openspec/specs/<capability>/spec.md`, call it out immediately. "Your spec defines 'cancellation' as X, but you seem to mean Y — which is it?"

### Sharpen fuzzy language

When the user uses vague or overloaded terms, propose a precise canonical term. "You're saying 'account' — do you mean the Customer or the User? Those are different things."

### Discuss concrete scenarios

When domain relationships are being discussed, stress-test them with specific scenarios. Invent scenarios that probe edge cases and force the user to be precise about the boundaries between concepts.

### Cross-reference with code

When the user states how something works, check whether the code agrees. If you find a contradiction, surface it: "Your code cancels entire Orders, but you just said partial cancellation is possible — which is right?"

### Update OpenSpec inline

When a term, invariant, or capability boundary is resolved, update the relevant OpenSpec artifact right there. Don't batch these up — capture them as they happen. Use the guidance in [OPENSPEC-DOMAIN-FORMAT.md](./OPENSPEC-DOMAIN-FORMAT.md).

Don't couple OpenSpec docs to implementation details. Specs should stay behavioral and use project language that domain experts would recognize.

### Offer design docs sparingly

Only offer to record a decision in `openspec/changes/<change>/design.md` when all three are true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will wonder "why did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If any of the three is missing, skip the design note. Use the format in [OPENSPEC-DESIGN-FORMAT.md](./OPENSPEC-DESIGN-FORMAT.md).

</supporting-info>
