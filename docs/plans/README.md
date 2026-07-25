# Implementation Plans

This directory contains bounded implementation plans for active milestones and substantial features.

Permanent product truth belongs in:

- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/DESIGN.md`
- `docs/CANON.md`

Plans should not redefine those documents silently. A plan may narrow a milestone and sequence work, but an intentional product or architecture change must update the permanent document and decision log.

---

## Plan lifecycle

1. Create a numbered plan:
   ```text
   docs/plans/002-feature-name.md
   ```
2. Define goal, scope, non-goals, assumptions, and acceptance criteria.
3. Break work into vertical tasks.
4. Keep progress checkboxes current.
5. Record discovered risks and deliberate deferrals.
6. When complete, move the plan into:
   ```text
   docs/plans/completed/
   ```
7. Do not delete completed plans; they preserve implementation history.

---

## Plan template

```markdown
# Plan XXX — Title

**Status:** Proposed | Active | Blocked | Complete
**Target milestone:** Version or milestone
**Owner:** Drilon Reçica
**Related documents:** links

## Goal

One concrete outcome.

## User-visible result

What a user can do after completion.

## In scope

- ...

## Out of scope

- ...

## Assumptions

- ...

## Architecture impact

- affected crates;
- affected hosts;
- persisted data;
- determinism;
- content;
- performance.

## Work breakdown

### 1. Workstream

- [ ] Task
- [ ] Task

## Test plan

- unit;
- golden;
- rendering;
- host;
- manual.

## Acceptance criteria

- [ ] Observable condition

## Risks and mitigations

| Risk | Mitigation |
|---|---|

## Deferred follow-up

- ...

## Completion record

- Final commit:
- Checks run:
- Known limitations:
```

---

## Plan-writing rules

- Prefer a complete vertical slice over subsystem-only plans.
- Every platform-facing plan must cover native and browser.
- Every real-time game plan must address determinism.
- Every storage change must address versioning and recovery.
- Every visual change must address accessibility.
- Every fictional content change must match `CANON.md`.
- Keep plans specific enough that a coding agent can execute without this chat history.
- Do not make plans enormous; split independent milestones.
