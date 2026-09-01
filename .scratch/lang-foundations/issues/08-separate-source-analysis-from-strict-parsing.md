# 08 — Separate Source analysis from strict parsing

**What to build:** Represent permissive Live Edit analysis and strict Expression parsing with
distinct structural outcomes, rather than changing parser behavior through mutable mode flags and
injecting sentinel Functions for incomplete Source.

**Blocked by:** 02 — Make Function definitions compiler-checked; 07 — Pair Expression syntax and values structurally.

**Status:** resolved

**Tags:** release/v1

- [x] Strict parsing returns parsed Atoms or a typed error and never produces recovery sentinels.
- [x] Permissive analysis preserves complete entries while representing incomplete or invalid
      Source explicitly.
- [x] `Function::Empty` is removed and cannot reach display, signature lookup, or evaluator dispatch.
- [x] Parser validity is derived from the analysis outcome rather than a mutable behavior flag.
- [x] Existing Live Edit, trailing-content, missing-operand, and over-capacity behavior remains
      covered.
- [x] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.

## Implementation notes

`mise run bench` was run before and after the change. Before: parse 84 ns/iter (+/- 15),
parse_invalid 57 ns/iter (+/- 2), execute 11 ns/iter (+/- 0), and parse_source 328 ns/iter
(+/- 33). The after workload used `mise run bench`, plus the equivalent filtered commands
`cargo bench --package lang --locked execute -- --output-format bencher` and
`cargo bench --package lang --locked parse_source -- --output-format bencher` because the unfiltered
bencher output displayed only its first two cases. After: parse 87 ns/iter (+/- 8), parse_invalid
68 ns/iter (+/- 1), execute 11 ns/iter (+/- 0), and parse_source 330 ns/iter (+/- 14).

An independently repeated baseline of 309 ns/iter and an initial after measurement of 840 ns/iter
exposed a parse_source regression. Adding targeted inlining to the new strict and permissive public
parser boundaries restored the workload to 330 ns/iter. Existing forced-inlining annotations were
retained.
