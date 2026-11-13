# Optional Play note: a Play's note operand is typed as optional, and a blank note plays nothing

Status: resolved

## Problem Statement

A tracker or melody pattern leaves most of its steps empty. A performer reads
an empty step as "nothing plays here", and Orca's `:` with no note agrees. In
Orcvs that empty step has had two treatments, and neither works.

The Rest (ADR 0075, ADR 0104) made a blank note an exception. The Parser, the
Language Map, the Tick and the Function table each had to spot it on their own,
through a `rest:` clause, a `blank_rest` flag on a Positioned Entry and a
`Parsed::Rest` variant. When a rule is copied across four modules, the copies
drift apart. ADR 0104 exists because a nested Read and a Portal write of the same
empty Cells were treated differently.

ADR 0107 removed the Rest, so a blank note is invalid input again. Errors stay
contained to their Expression, so nothing the performer hears changes. But every
empty step of the shipped Melody, Tracker and Knobs examples now raises a
diagnostic on every Tick it is reached. The docked diagnostics fill with noise
that reports no mistake, and a real error is lost among them. Each of those
diagnostics also allocates its message on the Tick path.

"Blank means nothing plays" is part of what a Play's note *is*. It is not an
exception to the rules for operands. The program has no type that says so.

## Solution

Model the optional note in the operand types. The note slot of a Play is
declared as `Optional<Note>`, so the Play body receives `Option<Note>`. A body
that gets no note sends no Play Command. That is the whole rule, and it lives in
one place, the Play's signature.

For the performer:

- A blank note plays nothing and raises no diagnostic. That holds whether the
  Source was written blank, a Portal write emptied the slot, or a nested Function
  cleared it.
- A half-typed or mistyped note is still diagnosed, exactly as `Note` diagnoses
  it today.
- Every other operand of the Play binds as it always has. A blank channel or an
  out-of-range velocity is still an error, even when the note is blank.
- Errors stay contained. A blank note costs no other Expression its Turn, and the
  Play still takes its own Turn in Tick order.

Below the Function table, an operand slot carries an optional Atom from the
Parser entry through to binding. The Parser, the Language Map and the Tick only
carry an absent value. None of them decides what it means. Only the operand type
decides.

## User Stories

1. As a performer, I want a blank step in a Play's note to play nothing, so that a pattern with empty steps sounds like a pattern with rests.
2. As a performer, I want a blank note to raise no diagnostic, so that the docked diagnostics show only real mistakes.
3. As a performer, I want a half-typed note such as `C ` to be diagnosed, so that a note I am in the middle of typing does not go silent without explanation.
4. As a performer, I want a mistyped note such as `ZZ` to be diagnosed, so that a typo is reported rather than heard as a rest.
5. As a performer, I want a note that a Portal write emptied to play nothing, so that a Read above the note slot and a Read nested in it mean the same thing (ADR 0061).
6. As a performer, I want a note that a nested Read or Track cleared to play nothing, so that a tracker voice reading an empty step is quiet.
7. As a performer, I want a note I left blank in the Source to play nothing, so that I can lay out a Play before I choose its note.
8. As a performer, I want a Play with a blank channel or velocity to stay diagnosed, so that a missing literal is not mistaken for a rest.
9. As a performer, I want a Play with an out-of-range channel to be diagnosed even on a blank step, so that the mistake shows the moment I write it and not only when a note arrives.
10. As a performer, I want a Play with a blank note to keep its place in Tick order, so that nothing else in the Source moves or retimes around it.
11. As a performer, I want a Play with a blank note to cost no other Expression its Turn, so that errors and absences stay contained to their row.
12. As a performer, I want a Timed Play `!~` with a blank note to play nothing, so that timed voices rest the same way raw ones do.
13. As a performer, I want a Monophonic Play `!%` with a blank note to leave its channel's held voice to its own Note Off, so that an empty step does not cut a sounding note short.
14. As a performer, I want the shipped Melody example to play with no diagnostics, so that the first thing I open shows a clean Source.
15. As a performer, I want the shipped Tracker example to play with no diagnostics on empty steps, so that its voices read as written.
16. As a performer, I want the shipped Knobs example to play with no diagnostics on empty rows, so that the example teaches the Functions and not the error list.
17. As a performer, I want a note being typed during Live Editing to go from diagnosed to playing, without a silent undiagnosed state in between, so that I always know why a Play is quiet.
18. As a performer reading the Function reference, I want the note operand of each Play documented as optional, so that I know an empty step is allowed.
19. As a performer reading the Function reference, I want it to say that a half-written note is an error, so that I know where the line is.
20. As a maintainer, I want "may be blank" to be a property of an operand type, so that the rule is written once and cannot drift between modules.
21. As a maintainer, I want the Parser, the Language Map and the Tick to carry an absent operand without interpreting it, so that none of them holds a copy of the rule.
22. As a maintainer, I want a plain operand to refuse an absent value with the same diagnostic it gives today, so that every Function that does not opt in behaves exactly as before.
23. As a maintainer, I want a plain operand left blank to stay diagnosed in the Language Map before any Tick, so that static diagnosis does not regress to a per-Tick one.
24. As a maintainer, I want a nested Clear under an optional operand to bind to "absent", so that ADR 0073 needs only a type-shaped exception rather than a per-Function one.
25. As a maintainer, I want a nested Clear under a plain operand to keep leaving its parent invalid, so that ADR 0073 stands everywhere else.
26. As a maintainer, I want a Play with a blank note to allocate nothing on the Tick path, so that empty steps cost nothing per Tick.
27. As a maintainer, I want `Optional<O>` to work for any operand `O`, so that a later decision to make another operand optional needs only a signature change.
28. As a maintainer, I want the Play body to receive `Option<Note>`, so that the compiler forces every Play to say what an absent note does.
29. As a maintainer, I want an ADR recording that the note is optional by type and superseding ADRs 0075, 0104 and 0107, so that the next reader finds one current rule.
30. As a maintainer, I want CONTEXT.md to describe the optional note in existing vocabulary, so that the glossary still names no Rest.
31. As an agent picking up a ticket, I want each behaviour testable through `Source::execute`, so that I test what a performer sees rather than how it is wired.

## Implementation Decisions

- **An operand adapter, `Optional<O>`.** It is a new operand type in lang's
  operand module, generic over any `O: Operand`. Its Token is `O`'s Token, so
  the Parser decodes it as it decodes `O`. It binds an absent value to `None` and
  a present one through `O::bind` to `Some`, so a present value that `O` refuses
  is refused with `O`'s diagnostic. Its bound type is `Option<O::Bound>`. It sits
  beside `Domain<D>` and follows that type's pattern.
- **Where "may be blank" comes from.** Each operand type answers whether it
  admits a blank slot. The answer is `true` for `Optional<O>` and `false` for
  everything else. The Function table derives each slot's admission from its
  declared operand type, so the Function table holds no separate list and no
  `rest:` clause.
- **The slot carries an optional Atom.** From the Parser's Positioned Entry
  through the Language Map's resolved operands to `bind_atom`, an operand slot
  holds `Option<Atom>`. `None` means the slot is blank, or a nested Function
  answered a Clear. No module on that path branches on `None`, except to pass it
  on, until binding.
- **Plain operands keep their static diagnosis.** When the Parser meets a blank
  slot whose operand does not admit blank, it diagnoses it exactly as it does
  today, with the same message and anchor. A blank plain operand is therefore
  still reported in the Language Map before any Tick. The Parser reads the
  admission from the signature. It does not judge what the value means.
- **A nested Clear binds to absent.** The Tick's operand decode delivers `None`
  for a slot whose nested Function cleared. Binding then decides. An optional
  operand takes `None`, and a plain one refuses it with today's message, `nested
  computation at column X, row Y cleared its operand`. This amends ADR 0073 by
  operand type, not by Function.
- **Play rows declare `note: Optional<Note>`.** These are Raw Play `!>`, Timed
  Play `!~` and Monophonic Play `!%`. The shared Play body receives `Option<Note>`
  and returns no Play Command for `None`. The Play's Turn, Tick order and
  containment are untouched. Every other operand binds before the body runs, so
  a range or decode error on another operand is reported even when the note is
  blank. This reverses ADR 0104's "a Rest checks no operand's range".
- **A blank note allocates nothing.** Binding `None` and returning no command
  build no message and no collection.
- **An ADR records the decision.** It takes the next free number above the
  highest ADR, skipping any number an unlanded branch claims, which today means
  0108. It supersedes 0075, 0104 and 0107, and it amends 0069 and 0073 by operand
  type.
- **It builds on PR #376.** #376 removed the Rest and made a blank note invalid.
  This effort lands on that branch, so #376 merges with the optional note in
  place rather than shipping the noisy interim.

## Testing Decisions

- A good test states what a performer sees: which Play Commands a Tick sends and
  which diagnostics it reports, at which anchors, for a given Source. It does not
  assert how a slot is threaded through the modules.
- **The behaviour seam is `Source::execute`.** Tests read the Tick Plan's Play
  Commands and diagnostics together with the Language Map's diagnostics. Every
  user-visible claim in the user stories is tested here. That covers blank,
  half-typed and mistyped notes, Portal-emptied and nested-cleared notes, other
  operands still diagnosed, containment and Tick order, and the shipped examples.
- **The binding seam is lang's `evaluate`, given a Function and its operands.**
  Tests check that two blank Cells bind to absent, a valid note binds to a note,
  and anything else is refused exactly as `Note` refuses it. They also check that
  a plain operand refuses absent with today's message. `Operand::bind` gets no
  direct tests, because `evaluate` covers it from above.
- **Allocation.** A Play with a blank note allocates nothing per Tick, measured
  with the existing allocation-counting test harness.
- **Prior art.**
  - The tick module's blank-note tests, which #376 rewrote to assert diagnostics,
    are the fixture shape to invert.
  - The observed-route fixtures cover the written, Portal and nested routes.
  - The shipped tracker and knobs tests in playback cover the examples. Restore
    their exact Language Map check that #376 dropped, so that any diagnostic
    they gain is caught.
  - lang's allocation tests include the resting allocation test that #376
    removed.
  - The `evaluate` tests sit in lang's functions module.
  - The console's live-editing kittest covers typing a note in.
- Run proptest at `PROPTEST_CASES=32` locally, as the pull-request tier does.

## Out of Scope

- **Defaults for operands.** For example, `Default<O, V>`, an operand that binds
  a blank slot to a fixed value. It would sit at the same seam, and it is a
  separate decision.
- **Chord Play roots.** Making `!:` and `!#` take `root: Optional<Note>` is not
  enough on its own. An empty pattern row also blanks the Interval Set, voicing
  and inversion, and ADR 0104 let a blank root excuse them. Whether those become
  optional too, or the row stays diagnosed, needs its own decision. It is listed
  as a follow-up ticket.
- **Optional leading operands.** #375's ticket 01 found that empty leading
  operands silently nest the next Expression. That case does not arise while
  only the note, a trailing operand, is optional.
- **Unifying a Positioned Entry's binding into one enum (Atom, Empty or
  Refused).** That is #375's ticket 02. It is related, but this effort needs only
  the optional Atom.
- **The other #376 review items, #371's Chord Play driving, and the research
  doc citation of ADR 0075.** They are tracked on their own PRs.

## Further Notes

- Vocabulary: this spec names no new domain term. A note is "blank". The Rust
  types are `Optional<O>` and `Option<Note>`. CONTEXT.md keeps "Rest" and
  "silent Play" in the Play Function's Avoid list.
- #375's per-Function admission idea, where each Function decides what its empty
  slots mean, is the closest earlier work. This spec makes that decision the
  operand type's, so a Function makes it by declaring a type.
