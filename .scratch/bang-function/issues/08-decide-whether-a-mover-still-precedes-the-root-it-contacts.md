# 08 — Decide whether a mover still precedes the root it contacts

Status: resolved
Blocked by: 01, source-bundle/01, dependency-derivation/02

**What to build:**

`producer_edges` (`orcvs/src/source/tick/dependencies.rs:215`; the `moves` rule at `:243-245`, the contact loop at `:263-284`) still orders a Self-Banging Function before every root its reservation contacts that waits for a Bang. The edge existed so that contact could deliver activation; ticket 01 removed contact activation (D16), so a blocked mover now writes `**` and activates nothing in its Turn. The edge was kept because the handoff fixed the existing scheduling edges as unchanged.

What the edge still does is order: it makes a mover's Turn precede a contacted Terminal Output root, Halt or Directional Bang. No case was found where it closes a cycle, but nothing in the language now asks for it. Removing it would let Source order decide between a mover and the root it meets, as it already does for an intrinsically active root (ADR 0060 rule 3b).

The owner decides whether the edge goes. It is a scheduling change, so it is not for an agent to make unasked.

## Acceptance criteria

- [x] The owner decides whether the mover → contacted-root edge stays or goes.
- [x] If it goes: a moving bundle orders no contacts (`orders_contacts` excludes `clears_own_span() && writes_displaced()`), so the `moves` exemption becomes unnecessary and is removed; a test pins a mover meeting a Bang-activated root in Source order; and the placement property shows no ordering defect or cycle.
- [ ] If it stays: its comment states the ordering it keeps and why, without referring to contact activation.

## Comments

2026-10-11 — Raised as finding 2 of the `/code-review` run on PR #213 and filed for a decision at the owner's request.
2026-10-12 — Source-bundle ticket 01 replaced the `advances` predicate with `SourceBundle::clears_own_span` and `writes_displaced`. The edge is now the `moves` rule over those two facts in `producer_edges`, so either outcome edits that one rule.

### 2026-10-13 — Audit against origin/main cf01e3c

`producer_edges` moved to `orcvs/src/source/tick/dependencies.rs:215` in `a8ca4cf`; the body's
path is updated. All three blockers are resolved. Criterion 2 as written removes the
intrinsic-root exemption, which would add edges rather than remove one: the comment at
`dependencies.rs:271-275` says ordering two mutually covering movers after each other makes the
pair a cycle. Removing the mover → contacted-root edge means a moving bundle orders no contacts
(`orders_contacts`, `:239`). Restating the criterion is the owner's decision.

2026-10-13 — Criterion 2 restated: as first written it removed the intrinsic-root exemption, which would add edges and recreate the mutual-cover cycle (`orcvs/src/source/tick/dependencies.rs:271-275`). It now removes the edge by having a moving bundle order no contacts. Decided 2026-10-13 by Toby. Whether the edge goes is still the owner's call, so Status stays `ready-for-human`.

2026-10-13 — Decided by Toby: the edge goes. A Source-writing bundle now orders none of the Functions its reservation contacts (`orders_contacts` in `producer_edges`, `orcvs/src/source/tick/dependencies.rs`). The `moves` exemption goes, and so does the self-edge exemption: a Fire's only contact is itself, so a bundle that orders no contacts never reaches it.

2026-10-14 — Correction: the outcome does change. For the mover and the root alone it doesn't: none of the roots that wait for a Bang (Terminal Output, Halt, Directional Bang) vacates its own Cells during its Turn, so the mover is blocked whichever goes first. But everything ordered after the contacted root also waited for the mover. In `["  **", "  *>  <<", "  ^^"]`, on main `*>` waited for `^^`, so `<<` moved first and the emission was refused. Without the edge, `*>` takes its Turn in Source order, emits `>>`, and blocks `<<`. Found by `/code-review` on PR #222; Toby to reconfirm the decision on that basis. `a_mover_meets_a_bang_activated_root_in_source_order` (`bang_function.rs`) pins the new outcome and fails against the edge. `a_mover_meets_every_root_it_contacts_in_source_order` (`dependencies.rs`) checks a Halt, a Directional Bang, a Terminal Output and a mover, from the east, the west and the north. The placement property, now also drawing `**` and `*!`, checks only defects and cycles, so it cannot show that outcomes are unchanged.
