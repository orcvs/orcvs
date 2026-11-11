# A blank note is invalid input, however it got blank

Status: superseded by [ADR 0108](0108-a-plays-note-is-optional-by-type.md): a Play's note is optional by type, so a blank note plays nothing and raises no diagnostic. It was accepted. Supersedes [ADR 0075](0075-a-play-with-a-cleared-note-rests.md) and
[ADR 0104](0104-a-blank-note-or-root-is-a-rest-however-it-got-blank.md): Orcvs has no Rest.
Restores [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)'s rule that an unwritten slot is
invalid input, and [ADR 0073](0073-a-function-with-nothing-to-write-answers-nothing.md)'s rule that a
nested Clear leaves its parent invalid, for the note operand of Raw Play `!>`, Timed Play `!~` and
Monophonic Play `!%`. Number 0106 is skipped: an unlanded branch claims it.

ADR 0075 and ADR 0104 made a blank note a Rest: a Turn taken that sends nothing and reports
nothing. Every other blank or cleared operand of every Function is invalid input, so a Rest was a
third outcome beside evaluating and being invalid. ADR 0069 rejected a third outcome because the
Language Map's roots, the schedule and every Turn had to carry it. The Rest had to be carried the
same way: the Function table declared a rest operand, the Parser recorded a blank one without
refusing it, the Language Map admitted the Play as a root, strict parsing refused it after the
fact, and each Turn checked for it while decoding.

## Decision

**A Play whose note is blank is invalid input, however the note got blank.** That holds whether the
Source was written blank, a Portal write emptied the slot, or a nested Function answered a Clear.
The Play does not evaluate, sends no Play Command and is diagnosed, as for any other operand that
holds no value. The Parser refuses a blank note written in Source and the Language Map diagnoses
the Play. A note emptied during the Tick is refused when the Play's Turn decodes it. A nested Clear
leaves the note unwritten, and the Play names the child that cleared it.

**The diagnostic is the Play's alone.** [ADR 0065](0065-an-error-never-stops-the-performance.md)
already makes an error cost only the Expressions it reaches. An invalid Play writes no Cell and
delivers no activation, so it reaches nothing: every other Expression, the Tick and the clock go on
exactly as they would if the note were written. A note an earlier Turn started still sounds until
its own Note Off, on a Monophonic Play too, because an invalid Play asks the Playback Engine for
nothing.

## Consequences

- An empty step in a tracker or sequencer is diagnosed at its voice's Play on every Tick that
  Bangs it. That includes the shipped Melody, Tracker, Sequencer and Knobs examples. What a
  performer hears does not change, because a Rest and an invalid Play were both silent.
- A channel, velocity or length outside its MIDI domain is diagnosed only on a Turn whose operands
  all decode, as for every other Function. This is no longer a special case for a blank note.
- A Play and a Function deriving a value from a Note now treat an empty Note the same way.

## Considered options

- **Keep the Rest.** A diagnostic on every empty step is noise in a pattern that is correct. This
  was rejected because the Rest needed a third outcome everywhere a Turn is decoded, and every
  attempt to keep it smaller moved that exception somewhere else rather than removing it.
- **Send nothing and report nothing for any empty MIDI operand.** This removes the special case
  for the note but adds the same third outcome for every MIDI operand.
