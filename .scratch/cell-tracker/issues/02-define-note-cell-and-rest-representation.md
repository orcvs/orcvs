# 02 — Define the Note Cells, ownership and rest representation

Status: needs-triage
Blocked by: 01

## Problem

Bare `C2` has no intrinsic Note type in Orcvs. A row of such spellings requires
an explicit interpretation and ownership contract. Existing Sequences cannot
contain Absence, so silently skipping blanks changes the rhythm.

## Work

Define how the Parser and Language Map recognize the editable data region and
where its Note context comes from. Decide whether a declaration owns the region
or the read explicitly interprets permitted Source spans; describe how that
choice coexists with existing expressions and diagnostics. The receiving MIDI
operand must not retroactively type an arbitrary nested read result.

Specify the stored and evaluated representation of a rest. Compare a positional
Sequence representation with a scalar read/activation solution under 01's
operation contract. An Absence member or a new Atom kind is a language change
requiring an explicit amendment, including effects on Select, Replace,
Concatenate, pervasion and Source encoding where applicable.

## Acceptance

- [ ] One step's width, alignment, spacing and extent are unambiguous in Cells.
- [ ] Notes use existing pitch/octave interpretation, including sharps and MIDI
      limits; spelling is decoded through one authoritative path.
- [ ] Empty pair, partially empty pair, invalid Note, invalid Function spelling,
      Comment, Number-looking text, and partial Language Unit have stated outcomes.
- [ ] A blank step retains its index; an all-rest pattern retains its length.
- [ ] Overlapping declarations/readers and contact with executable expressions
      have defined ownership and diagnostic behavior. Data is never accidentally
      executed as a Function during the same or a later Tick.
- [ ] The contract distinguishes a rest, malformed data, an empty collection,
      and a failed supplier. It states whether unselected malformed steps block
      the selected step and why.
- [ ] Diagnostic anchors, Spans and normal successful paint classification are
      specified using the existing Language Map vocabulary.

This is a decision ticket. No special parser, implicit Note-list syntax or new
rest spelling is authorized until its contract is accepted.
