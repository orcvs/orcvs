# 01 — Unify live operand resolution

**What to build:** Keep a Function's pending state, decoded operands and live
operand extent consistent through one private operand-resolution module within
execution. Track selects its Input Portal after the same live operands it reads,
including when a write suppresses a nested Function or its Turn waits for a
writer and retries. Preserve the Source, Play Commands and diagnostics a
performer observes.

**Blocked by:** None — can start immediately.

**Status:** resolved

## Acceptance criteria

- [x] One shared decision determines whether each operand consumes a surviving nested Function's Return or the Cells at its anchor. Pending checks, value decoding and live extent all use that decision rather than independently interpreting suppression. *(`orcvs/src/source/tick/execution/operands.rs:1-13`.)*
- [x] A surviving nested Function supplies its encoded Return, decoded according to the receiving operand's declared literal type under ADR 0061. Its live extent follows its last operand recursively, including nested operands. *(`orcvs/src/source/tick/execution/operands.rs:1-13`.)*
- [x] A suppressed nested Function instead supplies the two Source Cells at its anchor, and its operand ends immediately after those Cells. For Track's count, the selectable pairs begin there, including when the remaining characters formerly belonged to the suppressed Function's operands. *(`orcvs/src/source/tick/execution/operands.rs:1-13`; `orcvs/src/source/tick/track.rs:391`.)*
- [x] The module owns live-operand pending and error classification. The existing check for unchanged Source syntax remains separate and preserves Parser diagnostic ownership without adding a duplicate Tick diagnostic. *(`4a0f4f7`: the unchanged-syntax check stays in `syntax_blocks`.)*
- [ ] Pending detection considers all operands before decoding any operand. A later pending operand therefore leaves the Function pending even if an earlier operand would fail decoding. Preserve this diagnostic precedence in Source-level coverage. *(Superseded by ADR 0069 and pending-presentation/01: Orcvs has no pending state, and `a44d08e` removed the precedence test.)*
- [ ] Preserve ADR 0066's distinctions: an empty literal slot, pending child or copied empty Return leaves the parent pending; a missing Return from an evaluated child, including the Absence Marker, diagnoses. Encoding and receiving-token decoding failures retain their existing diagnostics. *(Superseded by ADR 0069, which supersedes ADR 0066.)*
- [x] Every attempted Turn resolves operands and their extent from current execution state. Resolution is temporary to that attempt, with no value or extent cache retained across retries. Completed children retain their Returns and are not interpreted again when their parent retries. *(`orcvs/src/source/tick/track.rs:415-436` asserts each completed child is interpreted once.)*
- [x] Track retains ADR 0067's selection and delivery behavior, including nested index/count expressions, same-Tick operand writes, zero-count and row-edge diagnostics, and its Output Portal and nested Return. The refactor changes no language semantics or dependency ordering. *(`4a0f4f7`: no language-semantics or ordering change.)*
- [x] Keep the Source Tick interface as the principal test surface. Preserve existing nested operand, suppressed-count, empty Return, current-Tick count-write and deferred-Return scenarios. Retain focused single-evaluation assertions where they verify the contract rather than implementation bookkeeping. *(Existing Source-level scenarios retained in `4a0f4f7`.)*
- [x] Add a Source-level regression scenario combining a suppressed nested count with an actual defer/retry caused by an unresolved writer of the selected pair. Verify the selected Cells, resulting output and diagnostics, and that completed nested Functions are interpreted at most once. *(`orcvs/src/source/tick/track.rs:391`.)*
- [x] Keep the module private to execution. Introduce no public interface, hypothetical adapter, or shipped parameter or branch populated only by tests. Retain the Source/Playback separation and atomic Tick publication. *(`mod operands;` is private at `orcvs/src/source/tick/execution.rs:25`.)*
- [ ] Review the complete diff and run repository-required verification for affected crates, setting the local proptest case count to 32. Preserve native, WASM and declared feature behavior; defer the broader checks designated for CI. *(No verification recorded; merged via PR #205.)*

## Scope and constraints

This is a consolidation under ADRs 0034, 0061, 0066 and 0067. The glossary's
Track definition records the agreed suppressed-count rule; no new domain term
or language-design ADR is required.

Turn ordering ownership is a separate effort. Neither refactor requires the
other's design or implementation, so there is no blocking edge. Sequence work
on the overlapping execution code to avoid conflicting edits, adapting to the
ordering implementation present when this ticket starts.

Locality is the objective: one module owns the live-operand decision and its
classification, giving pending checks, decoding and Portal placement leverage
from the same rule. Avoid a forwarding module that leaves those decisions
duplicated. Make no performance improvement claim from this refactor alone.

## Comments

### 2026-10-13 — Audit against origin/main cf01e3c

Delivered by `4a0f4f7` in PR #205: a private `mod operands`
(`orcvs/src/source/tick/execution.rs:25`) owns the operand decision, and the Source-level
regression is `a_track_whose_nested_count_is_suppressed_waits_for_the_writer_of_the_pair_after_it`
(`orcvs/src/source/tick/track.rs:391`). Boxes 5 and 6 describe ADR 0066's pending state, which
ADR 0069 superseded the next day; `a44d08e` removed the precedence test. The ticket records no
local verification, so box 12 stays unticked; the change merged through PR #205.
