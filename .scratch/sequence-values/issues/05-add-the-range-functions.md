# 05 — Add the Range Functions

**What to build:** Implement Number Range `:-` and Note Range `:#` as two monomorphic Functions
with distinct signatures, per ADR 0007 and ADR 0023. Each fixes its own operand and result type:
`:-` takes two Numbers and returns an inclusive unit-step Sequence of Numbers; `:#` takes two Notes
and returns an inclusive chromatic Sequence of Notes. Neither selects its behaviour from its
operands, and neither converts implicitly.

**Blocked by:** 06 — Project a Sequence result into Source; 07 — Make the Comment a Parser unit.

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
      recognised as a complete two-Cell Function. ADR 0035 spells the Comment introducer `||`, so
      no Comment rule reaches into `:#`.
- [ ] Range output takes the ordinary complete-fit Portal rule and never writes a partial Sequence.
- [ ] ADR 0036's reservation derivation is exercised end to end from a declaration: a Range row
      reserves the Cells from its destination through the end of that row, a pervasive parent
      widens over a Range child, a parent that is not pervasive does not, and the Turns inside each
      reservation are ordered after the computation that reserved them. Until this lands the
      derivation pass has no test at all — every `Reserved::Row` in the crate is stated by a
      fixture, so the pass can only ever answer `Reserved::Pair` from what Functions declare.

## Comments

Split out of `issues/03` during the `release/v1` issue alignment on 2026-09-04. Issue 03 owns the
four structural Sequence Functions `:<`, `:&`, `:?` and `:=`, which ADR 0023 keeps generic because
they do not reinterpret their Atoms. Range is the opposite case, so it does not belong there: ADR
0023 gives `:-` and `:#` distinct names precisely so that each has one fixed operand and result
signature and a mistyped bound diagnoses instead of silently selecting the other behaviour.

Both Range Functions are owned here, together. Do not open a second issue for `:#` — one Function
per issue would give the two halves of one decision two owners and two chances to disagree about
what a mistyped bound does.

The reservation acceptance line is owed by `test-only-seams/02`, which deleted
`live_a_nested_sequence_answer_widens_the_reservation_of_the_root_above_it`. That test asserted the
widening against a Sequence answer stated through a `cfg(test)` seam, so with the seam gone it
asserted a width the fixture had already stated.

The loss is wider than that one test. With nothing stating a Sequence answer through the scheduler,
and no Function declaring one, every input to the derivation pass says `Reserved::Pair`: the pass
runs on every Tick and cannot answer anything else, and the fixtures that need a row-wide
reservation state it directly. Declaring a Sequence answer here is what gives that pass its first
real input, which is why this line owes the derivation rather than only the widening.

These Range fixtures do not inherit the stated-reservation seam. A declared `:-` row derives its
own width, so a test written here spells the Function in Source and states nothing — the seam of
`test-only-seams/02` exists precisely because that is not possible yet.

The mixed-bounds acceptance line is the one carried over from issue 03 rather than dropped.
`orcvs-language-migration/04` settled that the same two Source characters take exactly one type
from the containing Expression and its Function signature; the diagnostic is what that decision
costs, and without this line nothing in the tracker requires it.

`CONTEXT.md:99-101` and ADR 0007 are the wording of record for both Functions.
