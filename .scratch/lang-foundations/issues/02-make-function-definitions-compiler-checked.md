# 02 — Make Function definitions compiler-checked

**What to build:** Make each real Function one compiler-checked definition of its canonical
two-Cell spelling and fixed operand signature, so adding a Function cannot leave parsing,
rendering, signature lookup, enumeration, or dispatch silently incomplete.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Parsing and rendering use the same canonical spelling definition.
- [x] Every real Function has exactly one fixed operand signature.
- [x] The complete real-Function enumeration cannot silently omit a newly added Function.
- [x] Signature lookup and evaluator dispatch are exhaustive and contain no wildcard fallback for a
      real Function.
- [x] Every enumerated Function parses from its spelling and renders back to the same spelling.
- [x] Parser benchmark commands and before/after results are recorded; measured forced-inlining
      choices are retained unless the measurements justify a targeted change.

## Comments

Parser recovery sentinels are removed separately after Expression entries become structural. They
must not be treated as real Functions or included in the canonical enumeration.

Implemented with one declarative definition per real Function. The definition generates the enum
variants, complete real-Function enumeration, canonical spelling/parser lookup, and fixed operand
signature. Evaluator dispatch names every variant explicitly; `Empty` remains a recovery-only
sentinel outside the enumeration.

Parser benchmark command (run at `bbe882e` and after the change):

```sh
cargo bench --package lang --locked --bench lang -- parse --noplot --output-format bencher
```

| Benchmark | Before | After |
| --- | ---: | ---: |
| `parse` | 107 ns/iter (+/- 2) | 85 ns/iter (+/- 2) |
| `parse_invalid` | 61 ns/iter (+/- 1) | 53 ns/iter (+/- 3) |

The ordinary `mise run bench` command was also exercised. Gnuplot failed intermittently while
Criterion produced plots, so `--noplot` was used for the recorded comparison. Existing
`#[inline(always)]` choices on parser lookup paths were retained; the measurements do not justify a
targeted inlining change.
