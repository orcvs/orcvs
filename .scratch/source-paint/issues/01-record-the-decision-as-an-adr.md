# 01 — Record the decision as ADR 0040

**What to build:** An accepted ADR saying that the console derives a per-Cell paint description as a value and converts it to Shapes in a separate step, and recording the alternative that was rejected.

The alternative worth recording is the one the repository contract forbids: publishing `RenderFrame::derive` so a test can build a Render Frame directly. The review that raised this candidate put it in one sentence — "publishing `RenderFrame::derive` is not the fix; the return value is the seam that already exists" — and that sentence will not survive in a `.scratch/` spec. It is what a future reader needs when they wonder why the console has two layers for one repaint.

The ADR also carries the causal chain from `spec.md`'s "Why the test surface is missing", because that is the part a reader will not reconstruct: the divergence from egui's architecture was forced by ADR 0038's layer-transform finding, not chosen for testability.

**Take number 0040**, the next number above the highest file in `docs/adr/`. That is the rule `docs/adr/README.md` states, and it is the whole of the decision — no judgement about which gaps look reusable enters into it.

**Do not take 0027.** `docs/adr/README.md` names that number specifically:

> Take the next number above the highest file in this directory. A gap is not a free number: `0027` is missing here because a branch that has not landed claims it, and reusing it would land the collision this directory has already had once.

An earlier draft of this ticket argued the gap was free because `docs/adr/0027-revalidate-play-commands-at-the-output-adapter.md` lives only on `revalidate-play-commands-at-the-output-adapter` — 307 commits behind `main`, four commits of its own, its decision reversed by shipped code, `orcvs::source::model` now exporting `MidiChannel`, `Velocity`, `Controller`, `ControlValue`, `BendLsb` and `BendMsb`. All of that is true and none of it matters: the README's rule is about the file the branch holds, not about whether its decision survived. Nothing local would catch the reuse either — `scripts/check-tooling-contract.sh` fails only on two files in `docs/adr/` sharing a leading number, and there is no 0027 on `main` to collide with.

Do not renumber anything.

**Blocked by:** None — can start immediately.

An earlier draft blocked this on `console-testing/02` — Define Console in the glossary — reasoning
that this effort's **Paint** entry is defined against the **Console** entry that ticket adds, so
Console had to land first. That is not what happened, and the block is stale. The Paint entry landed
on this branch in `20d2a8a` under ticket `05`, whose own `Blocked by:` line reads `03, 04` and never
named `console-testing/02`; the text it added to `CONTEXT.md` names no Console, defining a Paint as
"the per-Cell decision of how one Render Frame is drawn" and saying where the geometry it carries
none of belongs. Nothing in the checklist below needs a Console glossary entry either: every item is
about `docs/adr/0040`, and an ADR is not the glossary.

`console-testing/02` still owns the **Console** entry, is still `ready-for-agent`, and specifies it
down to its `_Avoid_` list. Do not write that entry here. It simply does not gate this ticket.

**Status:** resolved

- [x] `docs/adr/0040-the-console-paints-from-a-value.md` exists, status accepted, in the house format.
- [x] It states the decision: the paint is derived as a per-Cell value carrying no geometry, and a separate step converts it to Shapes.
- [x] It records the rejected alternative of publishing `RenderFrame::derive`, and why the return value is not the seam either.
- [x] It cites ADR 0038 for why the console left egui's widget surface, and does not restate that decision or reopen it.
- [x] It records the rejection of `epaint::Mesh::add_colored_rect`. Nothing else in epaint merges adjacent same-colour rectangles, so the coalescing fold is the console's own work — but one `Shape::Mesh` carrying a quad per Cell would skip per-rect tessellation and feathering, need no coalescing to cut vertex count, and tile exactly, since mesh quads have no anti-aliased edge. It is the one facility in epaint that does what the fold is for, and it is refused rather than passed over: it moves vertex placement and pixel snapping into this crate and gives up per-rect anti-aliasing. Say that, so the next reader does not have to find it.
- [x] It records that screenshot testing was investigated and refused, citing `console-testing/spec.md` as the owning decision.
- [x] No ADR is renumbered, and `.scratch/adr-numbering/` is not touched. That effort is settled: `728183c` renumbered the pulse decision to 0039, `adr-numbering/01` is `resolved`, and the policy it was asking about is now written in `docs/adr/README.md`. There is no open question left to add a data point to.

## Verification

Documentation only. `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` for the `.scratch/` change.
