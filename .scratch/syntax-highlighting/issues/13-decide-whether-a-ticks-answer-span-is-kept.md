# 13 — Decide whether a Tick's admitted answer span is kept for the next frame

**What to build:** A recorded decision on whether each root's admitted answer span is carried past the Tick Plan, so the next frame can read exactly which Cells a root last wrote. The span exists during a Tick, on each admitted write, and is lost when the Tick Plan flattens writes into per-Cell writes. After commit, only Cell contents remain.

**Blocked by:** 12 — Fit a Sequence-capable Output Portal to its answer.

**Status:** needs-triage

**Tags:** release/v1

- [ ] The decision is recorded: keep the span, or do not, and why.
- [ ] If kept: where it lives, when it is discarded after an edit, and how it reads where a later producer overwrote part of an earlier one's span. Also whether it needs a `CONTEXT.md` entry and an ADR.

## Comments

**2026-09-19.** Raised while fixing the Sequence-capable highlight. `12` chose the written-content rule instead, because it is paint-only and reads the current revision.

Costs of keeping the span:

- The Tick Plan is public and crosses into playback and tests.
- It is the first Tick outcome the model would keep. `05` chose the current revision alone so the highlight never goes stale, and a kept span is a fact about the last Tick.
- It needs a rule for discarding the span after an edit to the root or its Output Portal row.
- Writes resolve Cell by Cell, and a later producer wins each Cell it overlaps. An earlier root's span can therefore name Cells that now hold another root's answer.

It would not remove the Output Portal derivation: before the first Tick, after an edit, and for an empty Sequence there is no span to read.

What it would fix: `12` absorbs stale or hand-written content directly after an answer, and a kept span would not.

When to do it: when a second consumer needs "what did this root write last Tick", such as inspection or a Tick history view. The highlight alone does not justify it.
