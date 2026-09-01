# 08 — Separate Source analysis from strict parsing

**What to build:** Represent permissive Live Edit analysis and strict Expression parsing with
distinct structural outcomes, rather than changing parser behavior through mutable mode flags and
injecting sentinel Functions for incomplete Source.

**Blocked by:** 02 — Make Function definitions compiler-checked; 07 — Pair Expression syntax and values structurally.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Strict parsing returns parsed Atoms or a typed error and never produces recovery sentinels.
- [ ] Permissive analysis preserves complete entries while representing incomplete or invalid
      Source explicitly.
- [ ] `Function::Empty` is removed and cannot reach display, signature lookup, or evaluator dispatch.
- [ ] Parser validity is derived from the analysis outcome rather than a mutable behavior flag.
- [ ] Existing Live Edit, trailing-content, missing-operand, and over-capacity behavior remains
      covered.
- [ ] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.
