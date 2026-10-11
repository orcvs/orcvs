# 08 — Decide whether a mover still precedes the root it contacts

Status: ready-for-human
Blocked by: 01, source-bundle/01

**What to build:**

`producer_edges` (`orcvs/src/source/tick.rs`) still orders a Self-Banging Function before every root its reservation contacts that waits for a Bang. The edge existed so that contact could deliver activation; ticket 01 removed contact activation (D16), so a blocked mover now writes `**` and activates nothing in its Turn. The edge was kept because the handoff fixed the existing scheduling edges as unchanged.

What the edge still does is order: it makes a mover's Turn precede a contacted Terminal Output root, Halt or Directional Bang. No case was found where it closes a cycle, but nothing in the language now asks for it. Removing it would let Source order decide between a mover and the root it meets, as it already does for an intrinsically active root (ADR 0060 rule 3b).

The owner decides whether the edge goes. It is a scheduling change, so it is not for an agent to make unasked.

## Acceptance criteria

- [ ] The owner decides whether the mover → contacted-root edge stays or goes.
- [ ] If it goes: the edge and the `advances` predicate's last caller are removed, a test pins a mover meeting a Bang-activated root in Source order, and a property shows removing the edge introduces no ordering defect or cycle.
- [ ] If it stays: its comment states the ordering it keeps and why, without referring to contact activation.

## Comments

2026-10-11 — Raised as finding 2 of the `/code-review` run on PR #213 and filed for a decision at the owner's request.
