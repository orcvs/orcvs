# 05 — Extend the Terminal Output Functions over Sequences

**What to build:** Let the Terminal Output Functions extend pervasively over a Sequence operand per
ADR 0030, so one Expression answers an ordered group of Play Commands and a chord needs no new
spelling.

**Blocked by:** sequence-values/02 — Broadcast Atomic Functions over Sequences; sequence-values/05 —
Add the Range Functions, without which no Source text can spell a Sequence operand to reach this.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `!>` and `!~` declare that they extend, and the declaration is the only place that decides it.
- [ ] A scalar operand repeats across every element, equal-length operands pair element-wise, and
      incompatible non-scalar lengths diagnose.
- [ ] One Expression answers an ordered group of Play Commands; a scalar Expression still answers
      exactly one and no group of one.
- [ ] Commands from one Expression order by element index, and ADR 0020's Source-position order
      between Expressions is unchanged.
- [ ] A domain or type failure at any element emits no MIDI output at all, not the elements that
      validated.
- [ ] Timed Play schedules one Note Off per element at that element's own length.
- [ ] Monophonic Play leaves the last element owning the channel, per ADR 0016, and needs no
      separate diagnostic.
- [ ] The `lang` to `orcvs` seam carries the group, and the Playback Engine and output adapter
      deliver it in order.
- [ ] `CONTEXT.md` records that Terminal Output Functions extend, replacing the claim that they take
      Atom operands only.

## Comments

ADR 0030 records the decision and the reasoning. Two points matter most for whoever builds this.

The seam change is the real cost. `Interpretation::Play` carries one `PlayCommand` and has to carry
an ordered group. Control Change, Pitch Bend, and Monophonic Play are unbuilt, so doing this before
`midi-output-family/03` and `04` avoids writing three more call sites against the single-command
shape and changing them afterwards. If this issue is taken after those, expect the diff to be wider
for that reason alone.

The claim being replaced was never decided. `sequence-values/02` declared `!>` and `!~` scalar and
wrote into `CONTEXT.md` that "the Terminal Output Functions take Atom operands only". That came from
an implementation brief rather than from an ADR — ADR 0007 grants pervasive extension to the Atomic
Functions, which answer values, and is simply silent about Functions that answer effects. The
sentence beside it in the same entry cites ADR 0012 for why Increment stays scalar; the Terminal
clause cited nothing, which is what exposed it.
