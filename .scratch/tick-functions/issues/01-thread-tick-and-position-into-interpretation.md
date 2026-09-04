# 01 — Thread Tick and Position into interpretation

**What to build:** Supply each root evaluation with the absolute Tick and its Language Map anchor
Position while keeping the Source Snapshot and Tick Plan deterministic.

**Blocked by:** language-map/03 — Move Source consumers behind the Language Map.

**Status:** resolved

**Tags:** release/v1

- [x] Playback begins at Tick `0` and increments one unsigned counter per executed Tick.
- [x] Source interpretation receives Tick explicitly rather than reading wall time or global state.
- [x] Function evaluation receives its Grid-minted anchor Position from the Language Map.
- [x] Live Editing affects the next unsampled Tick without resetting absolute Tick.
- [x] Identical Source Snapshot, Tick, and inputs produce identical Tick Plans.
- [x] Each new Playback run restarts at Tick `0`, matching ADR 0012's first-Tick rule.

## Comments

Tick belongs to one Playback run; tests should pin restart behavior without adding another clock
seam.

### Anchor granularity is per root, not per Function — check this before 04

`emit_expression_root` builds one `TickInputs` for the whole Expression and hands it to the
Interpreter, so a *nested* Function sees its root's anchor rather than its own. That matches this
ticket's wording — the Language Map mints root anchors — but ADR 0013 seeds Random from "the
Function column" and "the Function row", and requires that "Functions at different Positions have
independent reproducible streams". Two Randoms nested in one Expression would share a stream.

The Interpreter walks a flat `Atoms` sequence with no per-Atom Position, so closing this means
carrying a Position per Language Unit through parsing, not widening `TickInputs`.
`tick-functions/04` either reopens this seam or records why root granularity is the intended
reading.

## Answer

The Playback Engine owns the absolute Tick. `PlaybackInner` holds a `Tick`, resets it to
`Tick::ZERO` in both the native and the `wasm32` `start` — and deliberately not in `retune`, which
changes the Tick period of the run it is already in — and advances it by one per *executed* Tick.
A Tick the engine declines (stopped, superseded generation, or overrun) returns before the
increment, so it consumes no absolute Tick. The counter lives there rather than beside the Source
because the Playback Engine owns musical time and ADR 0003 keeps every piece of language state in
the Source Snapshot; a counter next to the Source would be neither.

The Tick threads through `SourceCommander::execute`, `Source::execute`, `plan_tick`, `Turn::emit`,
and `emit_expression_root`, which builds one `lang::TickInputs { tick, anchor }` per Expression
root. The anchor is the root's own Grid-minted Position, carried across the crate boundary as its
column and row: a Position can be obtained only from the Grid that contains it, and that invariant
stays in the crate that owns the Grid. `TickInputs` rides on the interpretation `Context`, so a
Function reads the Tick the way it reads its operands, and a further input is a field rather than a
parameter at every call site. Nothing in the interpretation path reads a clock, a static, or a
thread-local.

`Tick::next` saturates. It is unreachable at any musical Tick period, and the alternative is worse
than unreachable: wrapping would return a run to Tick `0` and re-fire every Delay and Euclidean at
once. `Tick`, `Anchor` and `TickInputs` deliberately do not derive `Default` — a caller that cannot
say which Tick and which anchor it means has nothing to interpret, and withholding `Default` makes
that a compile error rather than a silent zero.

### What is pinned, and what is not

Five of the six criteria are pinned by tests. The sixth — that Function evaluation *receives* the
anchor — is pinned only up to the inputs `emit_expression_root` builds. No Function reads
`TickInputs` yet, so evaluation returns nothing that varies with them and an end-to-end test would
need either a probe Function, which ADR 0015 retired, or a consumer. `tick-functions/02` (Clock,
Delay, and Euclidean) is the first ticket that has one and should pin the read.
