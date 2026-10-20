# 01 — Prototype the Comment delimiter

Status: needs-triage
Blocked by: None — can start immediately

**What to build:**

A syntax prototype, as `docs/agents/syntax-prototypes.md` requires, that draws the tracker's pattern
rows under each delimiter in [`../spec.md`](../spec.md) side by side, so the delimiter can be chosen
by looking. It is a self-contained interactive page with guided walkthroughs that reset
deterministically, one Grid per Tick or edit, and each Grid's diagnostic beside it. Hypothetical
spellings are labelled in the page.

Scenarios:

- **A. The tracker plays.** Four voices' steps over Ticks 0 to 3, each Read's Input Portal lit and
  its Note played.
- **B. A Note typed into a step.** Empty step, half-typed `C ` (diagnosed by the Play), then `C4`.
- **C. The delimiter typed one Cell at a time.** While a two-Cell delimiter is half-typed, the row
  is not a Comment and its Notes diagnose as unknown functions; a one-Cell `#` is whole at once.
- **D. The delimiter's glyph inside Comment text.** Under each option, which text closes the
  Comment early.

## Acceptance criteria

- [ ] The prototype is built and committed on a throwaway branch, linked from this issue.
- [ ] The user has walked it and chosen a delimiter, recorded under `## Answer` here.
- [ ] If the delimiter changes: an ADR superseding ADR 0035's spelling, and issues to respell the
      Parser, `CONTEXT.md`, the Function reference, the shipped examples and the tests.
- [ ] The tracker example in #238 follows the decision.

## Comments

2026-10-20: The prototype is `lang/comment-delimiter-prototype.html` on the throwaway branch
`prototype/comment-delimiter`, built on `lang/bang-function-prototype.html`'s Grid cells, fonts and
walkthrough. It stays off `main`; delete the branch once the answer is recorded here.
