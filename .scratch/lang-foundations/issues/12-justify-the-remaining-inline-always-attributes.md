# 12 — Justify the remaining `#[inline(always)]` attributes

**What to build:** Every `#[inline(always)]` in `lang` is kept only where a benchmark shows it pays; the rest become `#[inline]` or are removed, with the measurement recorded here.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

source-audit/24 measured only the paths it touched. It relaxed the eleven attributes in `interpreter.rs`, `functions/sequence.rs` and `functions/numeric_conversion.rs`, measured flat, and kept `stack.rs` and `sequence.rs` as groups: relaxing the attributes in all five files it touched moved the `execute_function_operands/Subtract` bench (then named `execute_function/Subtract`) from 376 to 454–471 ns, restoring `stack.rs`'s attributes alone left it at 422–428 ns, and restoring `sequence.rs`'s as well returned it to 373–374 ns. Which attributes inside those two files carry the cost is not known. `lang/src/operand.rs`, added by source-audit/25 in orcvs/orcvs#164 after that measurement, carried 21 `#[inline(always)]` that no benchmark has measured (10 on cf01e3c).

- [ ] Take a fresh baseline on `main`, then bisect `stack.rs`'s six attributes (`lang/src/stack.rs:105,118,144,169,183,194`) one at a time against the `execute_function` and `execute_function_operands` benches, keeping the ones that measure.
- [ ] Measure the attributes in the files source-audit/24 did not touch — `atom.rs`, `parser.rs`, `expression.rs`, `lib.rs`, `portal.rs`, `functions/mod.rs`, `functions/math.rs`, `functions/tick.rs`, and `operand.rs` — against the `parse*` and `execute_function*` benches, starting from lang-foundations/02 and 07's recorded figures.
- [ ] Record each kept attribute's measurement here; a comment beside one is needed only where the reason is not the recorded bench.

## Comments

**2026-09-27 — bisection summary corrected; `operand.rs` added.** source-audit/24's 422–428 ns is the Subtract figure with `stack.rs`'s attributes restored and the other four files still relaxed, not a figure for relaxing `sequence.rs`; restoring `sequence.rs` as well gave 373–374 ns. `lang/src/operand.rs` (21 `#[inline(always)]`, from orcvs/orcvs#164) postdates that measurement and joins the second criterion's list.

**2026-09-30 — two attributes added since the last count.** orcvs/orcvs#180 added `#[inline(always)]` on the generated `Function::from_spelling` in `lang/src/atom.rs` (31 → 32 on `main`), and orcvs/orcvs#183 added one on `Stack::with_operands` in `lang/src/stack.rs` (17 → 18). Both join the criteria as they stand: `from_spelling` under the second criterion's `atom.rs` against the `parse*` benches, `with_operands` under the first criterion's `stack.rs` bisection. source-audit/24's Subtract figures (376, 454–471, 422–428 and 373–374 ns) predate the inline Operand Stack, so the `stack.rs` bisection needs a fresh baseline taken on `main` before any attribute is relaxed.

**2026-10-13 — Audit against origin/main cf01e3c.** `lang/src/functions/sequence.rs` was deleted in
`5e8a307` (the Sequence value was retired), so source-audit/24's `sequence.rs` group and the
373–374 ns figure that restoring it gave no longer exist; the first criterion is restated against
`stack.rs` alone. Recounted: 76 `#[inline(always)]`, all in `lang/src`. None remain in
`interpreter.rs` or `functions/numeric_conversion.rs`, and the newer `functions/copy.rs`,
`read.rs`, `track.rs` and `write.rs` carry none. Attribute lines:

| File | Count | Lines |
|---|---|---|
| `lang/src/atom.rs` | 27 (was 32) | 19, 28, 50, 59, 78, 109, 118, 172, 178, 185, 226, 231, 238, 451, 462, 547, 576, 589, 607, 619, 626, 664, 717, 1060, 1065, 1072, 1079 |
| `lang/src/stack.rs` | 6 (was 18) | 105 `with_operands`, 118 `push`, 144 `extract`, 169 `apply`, 183 `perform`, 194 `predicate` |
| `lang/src/operand.rs` | 10 (was 21) | 111, 124, 137, 150, 166, 179, 199, 231, 238, 244 |
| `lang/src/parser.rs` | 7 | 106, 177, 301, 314, 325, 331, 345 |
| `lang/src/functions/math.rs` | 9 | 25, 33, 39, 55, 62, 69, 80, 89, 95 |
| `lang/src/functions/tick.rs` | 9 | 30, 47, 68, 78, 106, 135, 175, 208, 255 |
| `lang/src/functions/mod.rs` | 5 | 21, 44, 70, 96, 119 |
| `lang/src/expression.rs` | 1 | 190 |
| `lang/src/lib.rs` | 1 | 344 |
| `lang/src/portal.rs` | 1 | 268 |

In `atom.rs`, lines 78, 109 and 118 sit inside `define_data_byte_roles!`, 226–238 inside
`function_kind!`, 451 and 462 inside `operands!`, and 547–664 inside `define_functions!`
(`from_spelling` is 547), so a source count is not an expansion count. The benches are in
`lang/benches/lang.rs`: `parse`, `parse_invalid`, `execute_function`, `execute_function_operands`
(including `Subtract`), `parse_source` and `parse_records`.
