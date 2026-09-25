# 24 — Reduce lang's per-Tick work and allocations

**What to build:** Interpreting a Tick's Turns does only the work the answers need. The shipped path is `Interpreter::execute_function`, called once per Turn by `orcvs`: it builds a `Context` sized `operands.len() + 1` and clones every operand `Value` onto it, so a Sequence operand is cloned as well. Reverse, Concatenate and Replace rebuild through `Sequence::new`, which re-checks membership of values that are already valid members, and note conversion checks its range twice. The operand stack's own allocation is allocation-reduction/03; this ticket covers the rest of the path.

Random's per-element generator is out of scope: ADR 0013 requires a fresh 32-byte ChaCha seed per scalar result, pinned by `random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping`, and `ChaCha8Rng` lives on the stack, so it costs CPU, not allocation. Changing it means superseding the ADR first.

**Blocked by:** 05 — Diagnose silent drops in lang; 09 — Move lang test-only API behind cfg(test).

**Status:** ready-for-agent

- [ ] A Sequence operand's members are not cloned anywhere on the binding path, or each remaining clone is measured and kept with the measurement recorded here. The path clones up to three times today: `orcvs` copies each child result into the Turn's operands (`source/tick/execution.rs`), `Interpreter::execute_function` pushes a clone of every operand, and the whole-value binds `bind_sequence_required` and `bind_sequence_operand` (`lang/src/stack.rs`) clone again.
- [ ] Reverse and Concatenate build their Sequence without re-checking membership; Replace checks only the replacement.
- [ ] Note conversion checks its range once.
- [ ] The allocation tests measure `execute_function`, the path `orcvs` calls, not `execute`, and are updated to the new, lower ceilings; a benchmark over `execute_function` shows the reduction. allocation-reduction/03 adds that test and benchmark; if this ticket lands first, it adds them and 03 reuses them.
- [ ] Every `#[inline(always)]` is kept only where a benchmark justifies it; the rest become `#[inline]` or are removed, with the measurement recorded in this ticket. lang-foundations/02 and 07 recorded the benchmarks that kept the current attributes; they are the baseline.
- [ ] Interpreter property tests pass unchanged.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `lang/` is unchanged since the baseline. Corrected: the shipped path is `execute_function` per Turn, not a fresh stack per Expression through `execute`, and the allocation test measured the latter. The operand-stack criterion duplicated allocation-reduction/03 and moved there. The per-call Random criterion contradicted ADR 0013 and is out of scope. Membership re-checks are CPU work, so the ticket now speaks of work as well as allocation.

**2026-09-25 — binding path widened.** Criterion 1 named only the push onto the stack; the whole-value binds clone a Sequence again, and `orcvs` clones it before `lang` sees it. Ownership of the `execute_function` allocation test and benchmark is settled with allocation-reduction/03.
