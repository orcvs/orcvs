# 04 — Propagate unit kinds by Grid index

**What to build:** Replace the linear scan in `LanguageMap::build`'s unit-kind propagation with a
lookup keyed by Grid index, so building a map is linear in Source size rather than quadratic.

**Status:** resolved

**Tags:** release/v1

Today `build` walks every parsed unit of every Expression and, for each one, calls
`map.units.iter_mut().find(|unit| unit.anchor == parsed.anchor)`
(`orcvs/src/source/language_map.rs:161-171`). The scan is over the whole unit list, so the cost is
the product of parsed units and total units. `build` already holds the Grid and every anchor is a
Position the Grid can index, so the join does not need a search at all: a `Vec<Option<LanguageUnitKind>>`
sized to the Source and indexed by `grid.index(anchor)` gives the same assignment in one pass.

- [x] Unit-kind propagation performs no linear scan per parsed unit.
- [ ] The kind assigned to each unit and the anchor semantics are unchanged.
- [ ] `orcvs/tests/language_map.rs` passes unmodified.
- [x] The Language Map build benchmark shows the improvement on a populated Source.
- [x] Native, persistence, and WASM gates pass.

## Comments

Found by a CodeRabbit review of the `01-add-proptest-for-native-targets` branch. The defect predates
that branch and none of its commits touch this file, so it was left out of scope and recorded here.

The repository contract asks for a reproducible benchmark behind any performance claim.
`.scratch/benchmarks/issues/06-benchmark-populated-source-edits-and-language-map-rebuilding.md`
already owns the benchmark that covers Language Map rebuilding; use it as the measurement rather
than adding a new one.

## Answer

Resolved as overtaken, not as built. The goal — a Language Map build that is linear in Source size —
is met and measured. The method this ticket specified is moot, because the scan it targeted no
longer exists.

**The scan is gone.** ADR 0024, landed as `ccab028` "Record spellings in the Language Map, not Atom
types", deleted the propagation step whole. There is no second pass that joins parsed units back to
partitioned units, so there is no `map.units.iter_mut().find(..)` to replace with a Grid-index
lookup: the row partition establishes each unit's `LanguageUnitKind` at the moment it recognises the
spelling, and nothing revisits it. `grep -n "iter_mut()" orcvs/src/source/language_map.rs` returns
nothing.

**The improvement is measured**, by `source-module-depth/06`, which used the benchmark this ticket
pointed at rather than adding one. Comparing `f8f7bb6` — the last commit before that rework — with
the current tip, on a quiet machine, two runs per commit agreeing within 0.5%:

| Grid  | Cells | before    | after     |
| ----- | ----- | --------- | --------- |
| 16x16 | 256   | 26.42 µs  | 10.94 µs  |
| 32x32 | 1024  | 178.76 µs | 38.20 µs  |
| 64x64 | 4096  | 1.7273 ms | 145.83 µs |

Four times the Cells used to cost 6.8x and then 9.7x; it now costs 3.5x and then 3.8x. That is the
quadratic-to-linear change this ticket asked for, stated as the ticket asked for it — a reproducible
benchmark, not an argument. Full numbers, conditions and reproduction are in
`source-module-depth/issues/06-measure-the-rebuild-path.md`.

**Two acceptance lines are left unticked, for different reasons.**

"The kind assigned to each unit and the anchor semantics are unchanged" did not hold and was not
meant to: ADR 0024 changed what a kind records, from the Atom type an Expression parsed to the
spelling the characters hold. That was a language-design decision taken after this ticket was
written, so the line is superseded rather than failed.

"`orcvs/tests/language_map.rs` passes unmodified" is untrue as written. That file was edited twice
since — once by `source-module-depth/05`, which changed how every caller names a Cell, and once by
`language-map/05`, which added a test. Neither touched an assertion's meaning, so the intent behind
the line holds; the letter of it does not, and it is not ticked.

Nothing to do. This issue is resolved, and it has since been removed from the `Blocked by:` line of
`v1-release/01`, so it is no longer a blocker.

### Correction: the two unticked lines are settled, not outstanding

Added during the `release/v1` issue alignment on 2026-09-04, in the shape
`property-testing/issues/01` uses for the same situation.

This file reads `**Status:** resolved` above five acceptance lines, two of which are unticked. That
looks like a contradiction to anything scanning checkboxes — an audit, a reviewer, or a future
reader deciding whether this issue is really done. It is not one. The Answer above states why each
line stays unticked; this note says so where the boxes are, so the two are not read apart:

- "The kind assigned to each unit and the anchor semantics are unchanged" is **superseded**. ADR
  0024 (`ccab028`) changed what a kind records, from a parsed Atom type to the spelling the
  characters hold. The line described a constraint the language then dropped, so it cannot be
  ticked and must not be.
- "`orcvs/tests/language_map.rs` passes unmodified" is **untrue as written and satisfied in
  intent**. The file was edited twice afterwards, by `source-module-depth/05` and `language-map/05`,
  and neither edit changed an assertion's meaning.

Neither line has work left in it. Do not reopen this issue on the strength of an empty box.
