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

- [ ] One shared decision determines whether each operand consumes a surviving nested Function's Return or the Cells at its anchor. Pending checks, value decoding and live extent all use that decision rather than independently interpreting suppression.
- [ ] A surviving nested Function supplies its encoded Return, decoded according to the receiving operand's declared literal type under ADR 0061. Its live extent follows its last operand recursively, including nested operands.
- [ ] A suppressed nested Function instead supplies the two Source Cells at its anchor, and its operand ends immediately after those Cells. For Track's count, the selectable pairs begin there, including when the remaining characters formerly belonged to the suppressed Function's operands.
- [ ] The module owns live-operand pending and error classification. The existing check for unchanged Source syntax remains separate and preserves Parser diagnostic ownership without adding a duplicate Tick diagnostic.
- [ ] Pending detection considers all operands before decoding any operand. A later pending operand therefore leaves the Function pending even if an earlier operand would fail decoding. Preserve this diagnostic precedence in Source-level coverage.
- [ ] Preserve ADR 0066's distinctions: an empty literal slot, pending child or copied empty Return leaves the parent pending; a missing Return from an evaluated child, including the Absence Marker, diagnoses. Encoding and receiving-token decoding failures retain their existing diagnostics.
- [ ] Every attempted Turn resolves operands and their extent from current execution state. Resolution is temporary to that attempt, with no value or extent cache retained across retries. Completed children retain their Returns and are not interpreted again when their parent retries.
- [ ] Track retains ADR 0067's selection and delivery behavior, including nested index/count expressions, same-Tick operand writes, zero-count and row-edge diagnostics, and its Output Portal and nested Return. The refactor changes no language semantics or dependency ordering.
- [ ] Keep the Source Tick interface as the principal test surface. Preserve existing nested operand, suppressed-count, empty Return, current-Tick count-write and deferred-Return scenarios. Retain focused single-evaluation assertions where they verify the contract rather than implementation bookkeeping.
- [ ] Add a Source-level regression scenario combining a suppressed nested count with an actual defer/retry caused by an unresolved writer of the selected pair. Verify the selected Cells, resulting output and diagnostics, and that completed nested Functions are interpreted at most once.
- [ ] Keep the module private to execution. Introduce no public interface, hypothetical adapter, or shipped parameter or branch populated only by tests. Retain the Source/Playback separation and atomic Tick publication.
- [ ] Review the complete diff and run repository-required verification for affected crates, setting the local proptest case count to 32. Preserve native, WASM and declared feature behavior; defer the broader checks designated for CI.

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
