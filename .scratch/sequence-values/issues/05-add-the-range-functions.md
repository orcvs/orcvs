# 05 — Add the Range Functions

**What to build:** Implement Number Range `:-` and Note Range `:#` as two monomorphic Functions
with distinct signatures, per ADR 0007 and ADR 0023. Each fixes its own operand and result type:
`:-` takes two Numbers and returns an inclusive unit-step Sequence of Numbers; `:#` takes two Notes
and returns an inclusive chromatic Sequence of Notes. Neither selects its behaviour from its
operands, and neither converts implicitly.

**Blocked by:** 01 — Add the Sequence language value; orcvs-language-migration/04 — Confirm contextual Number and Note literals; inherited-defects/15 — Bound the Operand Stack by the Expression length.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `:-` returns an inclusive unit-step Number Sequence; `:#` returns an inclusive chromatic Note
      Sequence that preserves Note identity in every member.
- [ ] Bound order selects ascending or descending output, and equal bounds return a singleton, for
      both Functions.
- [ ] Mixed-type bounds diagnose and produce no Sequence. Neither Function converts a bound to its
      own type, and neither falls back to the other Range's behaviour.
- [ ] A Bang is an invalid bound for both Functions and diagnoses. So do Function and Sequence
      bounds.
- [ ] `:#` rejects a Number bound above `7F` through the Note domain rather than through Range.
- [ ] Both Functions parse and round-trip through their canonical two-Cell spellings, and `:#` is
      recognised as a complete two-Cell Function distinct from the `##` Comment introducer.
- [ ] Range output takes the ordinary complete-fit Portal rule and never writes a partial Sequence.

## Comments

Split out of `issues/03` during the `release/v1` issue alignment on 2026-09-04. Issue 03 owns the
four structural Sequence Functions `:<`, `:&`, `:?` and `:=`, which ADR 0023 keeps generic because
they do not reinterpret their Atoms. Range is the opposite case, so it does not belong there: ADR
0023 gives `:-` and `:#` distinct names precisely so that each has one fixed operand and result
signature and a mistyped bound diagnoses instead of silently selecting the other behaviour.

Both Range Functions are owned here, together. Do not open a second issue for `:#` — one Function
per issue would give the two halves of one decision two owners and two chances to disagree about
what a mistyped bound does.

The mixed-bounds acceptance line is the one carried over from issue 03 rather than dropped.
`orcvs-language-migration/04` settled that the same two Source characters take exactly one type
from the containing Expression and its Function signature; the diagnostic is what that decision
costs, and without this line nothing in the tracker requires it.

`CONTEXT.md:99-101` and ADR 0007 are the wording of record for both Functions.
