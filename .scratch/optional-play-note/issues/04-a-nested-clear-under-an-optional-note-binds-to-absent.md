# 04: A nested Clear under an optional note binds to absent

**What to build:** When a nested Function, such as a Read or Track reading an empty step, answers a Clear in a Play's optional note slot, the Play plays nothing and raises no diagnostic. A nested Clear under a plain operand still leaves its parent invalid, as ADR 0073 says. See `../spec.md`, "A nested Clear binds to absent".

**Blocked by:** 02

**Status:** resolved

- [x] The Tick's operand decode delivers an absent value for a cleared nested slot, and binding decides what it means.
- [x] Through `Source::execute`: `!>`, `!~` and `!%` with a nested Read of two empty Cells in the note send nothing, and no diagnostic is reported.
- [x] Through `Source::execute`: a nested Clear under a plain operand, such as a channel, is diagnosed `nested computation at column X, row Y cleared its operand`, as today.
- [x] A nested-cleared note costs no other Expression its Turn.
- [x] The nested route on Bang-driven Functions (`!c`, `!b`) is covered as in #373 and #375's empty-operand tests, adapted to this rule.
