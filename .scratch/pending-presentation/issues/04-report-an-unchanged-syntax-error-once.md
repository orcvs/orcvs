# 04 — Report an unchanged syntax error once

Status: ready-for-human
Blocked by: None — can start immediately

**Requires design before implementation.** This ticket names a problem and one candidate rule. The rule changes how every syntax error is reported, not only an unwritten slot, so it needs an ADR clause agreed by a human before any code changes.

**Problem:**

The Language Map diagnoses a syntax error in the Source revision. During a Tick, `syntax_blocks` in `orcvs/src/source/tick/execution.rs` keeps the Turn silent about that error only while the Function is unchanged and every operand's working Cells equal the original. When a Portal write changes any one operand, the whole Expression is re-decoded, and an operand that is still unchanged and still invalid is diagnosed a second time.

Reproduction (found while reviewing PR #207, finding F6): `[".+  01","    &^","    05"]`. The Map reports `expected a number, found "  "` for the unwritten first operand; the Jump writes `05` over the second; the Tick reports the same `expected a number, found "  "` again. A partly written slot such as `" 1"` gets the same pair, so this predates ADR 0069 and applies to all invalid input.

The console shows no Source or Tick diagnostic messages today (ADR 0069), so no performer sees the duplicate. This matters once diagnostics are presented.

**Candidate rule (to be designed, not adopted):**

Test each operand separately: the Tick stays silent about an operand that the Map rejected and whose Cells are unchanged, even when another operand was written. Questions the design must settle:

- Does a Turn whose only invalid operand is unchanged still run, stay blocked, or fail silently? What does its parent see?
- When a written operand is itself invalid, which diagnostic names the Turn — the Map's for the unchanged operand, or the Tick's for the written one? Today the first operand in signature order that fails to decode names it.
- How does a Function replacement interact with per-operand silence?
- Does the rule need the Map's per-operand diagnostics recorded where `syntax_blocks` can read them, rather than one `syntax_valid` flag per node?

## Acceptance criteria

- [ ] An ADR (or a clause in ADR 0034 or 0069) states when the Tick reports a syntax error the Map already reported.
- [ ] The reproduction above yields one diagnostic for the unwritten slot, or the ADR records why two is correct.
- [ ] Tests cover an unchanged unwritten slot, an unchanged partly written slot, and an unchanged malformed slot beside a Portal-written operand.

## Comments

Accepted as intended behaviour for PR #207: `syntax_blocks` is unchanged there, and that PR's test `an_unwritten_operand_in_the_source_blocks_its_function_as_the_map_reports` claims silence only when nothing writes the Expression.
