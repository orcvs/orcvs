# 04 — Propagate unit kinds by Grid index

**What to build:** Replace the linear scan in `LanguageMap::build`'s unit-kind propagation with a
lookup keyed by Grid index, so building a map is linear in Source size rather than quadratic.

**Status:** ready-for-agent

**Tags:** release/v1

Today `build` walks every parsed unit of every Expression and, for each one, calls
`map.units.iter_mut().find(|unit| unit.anchor == parsed.anchor)`
(`orcvs/src/source/language_map.rs:161-171`). The scan is over the whole unit list, so the cost is
the product of parsed units and total units. `build` already holds the Grid and every anchor is a
Position the Grid can index, so the join does not need a search at all: a `Vec<Option<LanguageUnitKind>>`
sized to the Source and indexed by `grid.index(anchor)` gives the same assignment in one pass.

- [ ] Unit-kind propagation performs no linear scan per parsed unit.
- [ ] The kind assigned to each unit and the anchor semantics are unchanged.
- [ ] `orcvs/tests/language_map.rs` passes unmodified.
- [ ] The Language Map build benchmark shows the improvement on a populated Source.
- [ ] Native, persistence, and WASM gates pass.

## Comments

Found by a CodeRabbit review of the `01-add-proptest-for-native-targets` branch. The defect predates
that branch and none of its commits touch this file, so it was left out of scope and recorded here.

The repository contract asks for a reproducible benchmark behind any performance claim.
`.scratch/benchmarks/issues/06-benchmark-populated-source-edits-and-language-map-rebuilding.md`
already owns the benchmark that covers Language Map rebuilding; use it as the measurement rather
than adding a new one.
