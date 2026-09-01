# 02 — Add the bench profile and the mise task

**What to build:** A `[profile.bench]` that does not inherit the release link settings, and a `mise` task that runs the benchmarks with the exact command CI uses.

**Blocked by:** 01 — Add criterion and the lang benchmarks.

**Status:** resolved

- [x] `[profile.bench]` sets `lto = "thin"` and `strip = "none"`.
- [x] `mise run bench` runs `cargo bench --package lang -- --output-format bencher`.
- [x] `mise run check` does not run the benchmarks.

## Comments

`cargo bench` uses `profile.bench`, which inherits `[profile.release]`: `lto = true`, `codegen-units = 1`, `strip = "symbols"`. Full LTO across criterion's tree plus `lang` is a slow link on every push to `main`, and stripped symbols make a local profiling run much less useful. Thin LTO keeps inlining close enough for the numbers to mean something. What a comparison gate needs is a profile identical between runs, not one byte-identical to the shipped binary.

`check` stays out of it. The benchmarks add measurement time plus a bench-profile compile to every local full gate, and their verdict gates nothing locally.
