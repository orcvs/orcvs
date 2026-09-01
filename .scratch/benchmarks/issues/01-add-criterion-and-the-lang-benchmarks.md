# 01 — Add criterion and the lang benchmarks

**What to build:** A criterion bench target in `lang` with four benchmarks over Source drawn from the existing parser and interpreter tests.

**Status:** resolved

- [x] `criterion` is a `lang` dev-dependency at `default-features = false, features = ["cargo_bench_support"]`, which drops `rayon` and `plotters` from the tree.
- [x] `lang/benches/lang.rs` is declared as a `[[bench]]` target with `harness = false`.
- [x] `parse` measures `Parser::try_parse` on a nested Expression.
- [x] `parse_invalid` measures `Parser::parse` on malformed Source, the path a half-typed grid hits on every Render Frame.
- [x] `execute` measures `Interpreter::execute` on pre-parsed `Atoms`, with parsing outside the measured closure.
- [x] `parse_source` measures a full grid of rows in one iteration.
- [x] No micro-benchmarks on `str_to_num`, `midi_note_to_number`, or `Stack`.
- [x] `cargo bench --package lang -- --output-format bencher` emits one `ns/iter` line per benchmark.

## Comments

The dependency is a plain dev-dependency and lands in the workspace graph. `required-features` was measured and rejected: with the feature off and the bench target skipped, `cargo check --all-targets` still compiles the dev-dependency, because cargo enables dev-dependencies per package rather than per target. A separate excluded crate would keep the 46 crates out of the graph, at the cost of a second lockfile to keep `--locked` clean by hand and a tree `cargo deny` no longer audits. Neither buys back more than one cold compile per lockfile change, which `Swatinem/rust-cache` already absorbs.

The micro-benchmarks are excluded deliberately. `str_to_num` and `midi_note_to_number` matter only as a share of `parse`, and measuring them alone invites optimising something no workload waits on.

`[lib] bench = false` is required. Without it `cargo bench --package lang` also runs the unit-test harness, which rejects `--output-format` with `error: Unrecognized option: 'output-format'` and fails the task before criterion runs.
