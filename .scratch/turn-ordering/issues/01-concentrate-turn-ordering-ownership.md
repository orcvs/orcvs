# 01 — Concentrate Turn ordering ownership

**What to build:** Every Tick runs through one private Turn ordering module
that owns cached-order traversal, readiness, completion, discovered dependencies
and cycle diagnosis. A performer sees the same Source, Play Commands and
diagnostics: Track waits for writers of its selected pair, and independent
Expressions continue beside a dependency cycle. The refactor concentrates the
ordering invariants behind one internal seam while preserving the Source Tick
interface as the principal test surface.

**Blocked by:** None — can start immediately.

**Status:** resolved

## Acceptance criteria

- [x] One private module within Tick owns the complete ordering lifecycle, including the transition from cached traversal to continued ordering. Execution no longer reconstructs the remaining dependency graph or translates waiting, stopped and placed state to diagnose cycles. *(`orcvs/src/source/tick/ordering.rs:1-27`; private `mod ordering` at `orcvs/src/source/tick.rs:20`.)*
- [x] The immutable reusable schedule remains separate from mutable per-Tick progress. A Tick follows the cached order until a Turn discovers an unresolved writer; mutable dependency bookkeeping is created only when needed, with no unconditional per-Tick graph reconstruction. *(`orcvs/src/source/tick/ordering.rs:1-27`: the graph is rebuilt only when a Turn waits.)*
- [x] Execution owns operand resolution, Function interpretation and effects. It reports whether a Turn finished or must wait for writers; ordering decides which Turn runs next and owns progress, stopping and cycle diagnosis. No replaceable adapter abstraction is introduced. *(`Turn::Taken` / `Turn::Waits`, `orcvs/src/source/tick/ordering.rs:177-187`.)*
- [x] ADR 0032's dependency ordering and Grid-position tie-breaking remain intact. Every Function other than Track takes its Turns in the existing scheduled order, including Turns that settle without producing an effect. *(`orcvs/src/source/tick/ordering.rs:1-27`; `80bd343`.)*
- [x] ADR 0067's Track reads remain ordered at the Turn: unresolved writers of the selected pair go first, writers of unselected pairs introduce no dependency, and further dependencies discovered during continuation join the same ordering lifecycle. *(`orcvs/src/source/tick/track.rs` tests.)*
- [x] A deferred Track can read its nested Return again without interpreting the child twice. Track retains its current Input Portal, Output Portal and nested Return behavior. *(`orcvs/src/source/tick/track.rs:415-436`.)*
- [x] ADR 0065's cycle isolation and diagnostics remain intact for both precomputed and discovered dependencies: independent Expressions continue, a reader of a writer stopped by a cycle also stops, and cycle and downstream-waiting diagnostics retain their existing locations and ordering. *(`a_cycle_discovered_by_a_nested_track_stops_its_sibling`, `orcvs/src/source/tick/track.rs:640`.)*
- [x] Cached and freshly built schedules produce the same Tick Plans, diagnostics and contractual Turn ordering for Sources holding Track, including across revisions that reuse the schedule. Planning a Tick does not mutate the shared schedule. *(`orcvs/src/source/tick/schedule_reuse.rs`.)*
- [x] Existing Source-outcome, stopped-writer, dynamic-cycle, independent-Expression, schedule-reuse and single-evaluation tests remain the acceptance surface. Retain internal assertions where ordering or single evaluation is itself contractual; add focused regression coverage for uncovered behavior rather than tests that restate graph bookkeeping. *(Existing tests retained in `80bd343`, plus `track.rs:640`.)*
- [x] Test-only stated answers remain in test-only code below the shipped entry point. Adapting that helper introduces no shipped parameter or branch populated only by tests. *(`#[cfg(test)] mod stated`, `orcvs/src/source/tick/execution.rs:1016`.)*
- [ ] Review the complete diff and run the repository's required verification for the affected crates, with the local proptest case count set to 32. Preserve native, WASM and declared feature behavior; leave the broader checks designated for CI there. *(No verification recorded; merged via PR #205.)*

## Scope and constraints

This is an ownership refactor under ADRs 0032, 0065 and 0067, not a language
semantics change. Keep the Source/Playback separation and atomic Tick publication.
Operand value/extent consolidation is a separate opportunity and is outside this
ticket. Eagerly scheduling literal Track selections is also outside scope: ADR
0067 requires benchmark evidence before that optimization. Make no performance
improvement claim from this refactor alone.

The existing dependency loop already serves initial and continued ordering.
Deepen its ownership rather than deleting useful shared mechanics or merely
moving functions between files. Locality is the objective: readiness, completion
and stopping invariants should be understood in one module, with leverage across
both ordering paths.

## Comments

### 2026-10-13 — Audit against origin/main cf01e3c

Delivered by `80bd343` in PR #205: a private `mod ordering` (`orcvs/src/source/tick.rs:20`) whose
contract is its module doc (`orcvs/src/source/tick/ordering.rs:1-27`). `27587d9` later rebuilt its
internals on a plain `Graph` (`ordering.rs:414`) with the same lifecycle. The ticket records no
local verification, so the last box stays unticked.
