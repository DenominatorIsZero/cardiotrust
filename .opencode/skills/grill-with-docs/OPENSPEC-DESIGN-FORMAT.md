# OpenSpec Design Note Format

Record durable trade-offs in the active change folder:

- `openspec/changes/<change>/proposal.md` explains why the change exists
- `openspec/changes/<change>/design.md` records important design decisions and trade-offs

Create the change folder lazily — only when there is a real proposal or decision to capture.

## Template

```md
## Context

{What problem, constraint, or ambiguity forced a decision?}

## Decision

{1-3 sentences: what did we choose, and why is that the right trade-off?}

## Consequences

 - {Positive or negative downstream effect worth remembering}
```

That's it. A design note can be short. The value is in recording *that* a decision was made and *why* in the same OpenSpec change that carries the behavior change.

## Optional sections

Only include these when they add genuine value. Most design notes won't need them.

- **Status** — useful when decisions are revisited or superseded during review
- **Considered Options** — only when the rejected alternatives are worth remembering
- **Consequences** — only when non-obvious downstream effects need to be called out

If the change merges, the whole folder will later move under `openspec/changes/archive/`, preserving the rationale with the shipped spec delta.

## When to offer a design note

All three of these must be true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will look at the code and wonder "why on earth did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If a decision is easy to reverse, skip it — you'll just reverse it. If it's not surprising, nobody will wonder why. If there was no real alternative, there's nothing to record beyond "we did the obvious thing."

### What qualifies

- **Architectural shape.** "We're using a monorepo." "The write model is event-sourced, the read model is projected into Postgres."
- **Integration patterns between capabilities.** "Scenario storage resolves all scenario-owned output paths, not each UI flow independently."
- **Technology choices that carry lock-in.** Database, message bus, auth provider, deployment target. Not every library — just the ones that would take a quarter to swap out.
- **Boundary and scope decisions.** "This capability owns metadata only; payload loading is explicit elsewhere." The explicit no-s are as valuable as the yes-s.
- **Deliberate deviations from the obvious path.** "We're using manual SQL instead of an ORM because X." Anything where a reasonable reader would assume the opposite. These stop the next engineer from "fixing" something that was deliberate.
- **Constraints not visible in the code.** "We can't use AWS because of compliance requirements." "Response times must be under 200ms because of the partner API contract."
- **Rejected alternatives when the rejection is non-obvious.** If you considered GraphQL and picked REST for subtle reasons, record it — otherwise someone will suggest GraphQL again in six months.
