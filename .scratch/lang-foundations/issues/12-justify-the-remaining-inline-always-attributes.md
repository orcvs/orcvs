# 12 — Justify the remaining `#[inline(always)]` attributes

**What to build:** Every `#[inline(always)]` in `lang` is kept only where a benchmark shows it pays; the rest become `#[inline]` or are removed, with the measurement recorded here.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

source-audit/24 measured only the paths it touched. It relaxed the eleven attributes in `interpreter.rs`, `functions/sequence.rs` and `functions/numeric_conversion.rs`, measured flat, and kept `stack.rs` and `sequence.rs` as groups: relaxing `stack.rs` moved the `execute_function_operands/Subtract` bench (then named `execute_function/Subtract`) from 376 to 454–471 ns, and relaxing `sequence.rs` on top of that from 373 to 422–428 ns. Which attributes inside those two files carry the cost is not known.

- [ ] Bisect `stack.rs` and `sequence.rs` attribute by attribute against the `execute_function` and `execute_function_operands` benches, keeping the ones that measure.
- [ ] Measure the attributes in the files source-audit/24 did not touch — `atom.rs`, `parser.rs`, `expression.rs`, `lib.rs`, `portal.rs`, `functions/mod.rs`, `functions/math.rs`, `functions/tick.rs` — against the `parse*` and `execute_function*` benches, starting from lang-foundations/02 and 07's recorded figures.
- [ ] Record each kept attribute's measurement here; a comment beside one is needed only where the reason is not the recorded bench.
