# A blank note or root is a Rest, however it got blank

Status: accepted. Reverses the clause of [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md) that a Bang reaching a Play whose note slot is unwritten finds an invalid Play, and its consequence that such an Expression is not a root, for the note operand of a Play and the root operand of a Chord Play; every other decision in ADR 0069 stands. Extends [ADR 0075](0075-a-play-with-a-cleared-note-rests.md). Settled in `.scratch/chords-and-pitch/`. The 2026-11-11 amendment below says where the Language Map, which reads Source as written, departs from the Turn for a Chord Play.

ADR 0075 made a Clear in a Play's note operand a Rest, so the same pattern rests or errors depending on how its Cells arrive. A nested Read of two empty Cells rests. A Read standing above the note slot, writing those same empty Cells through its Output Portal, leaves an unwritten slot and is diagnosed on every empty step. The shipped Melody example is that second layout, and [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) says the two layouts should mean the same thing.

## Decision

**A fully blank note or root operand is a Rest.** This holds on Raw, Timed and Monophonic Play and on both Chord Plays, whether the Source was written blank, a Portal write emptied the slot, or a nested Function answered a Clear. The Turn is taken, sends no Play Command and raises no diagnostic. A Play or Chord Play with a blank note or root is therefore a root of the Language Map and is scheduled like any other.

**A half-written pair stays an error.** One blank Cell in a note or root is invalid input, as ADR 0069 says.

**A Chord Play's blank root ignores its other operands.** When the root is fully blank, other blank or cleared operands, the channel included, are not diagnosed, so a row of a pattern left wholly empty rests. A half-written operand, or a written one that does not decode as its type, such as `ZZ` in the set, stays an error even then. A blank Interval Set with a root present stays an error, because a root with no set cannot say what to play.

**A note Play's other operands still decode.** ADR 0075's rule stands for Raw, Timed and Monophonic Play: their channel, velocity and length are usually literals, and a blank one is a mistake rather than a musical rest.

**A Rest checks no operand's range.** A Rest takes no evaluation, so on any of the five Plays a written operand outside its domain, such as channel `80`, is diagnosed only on a Turn with a note or root to play, as ADR 0075 says for a note Play. A written operand must still decode as its type.

## Consequences

- Blank steps in the Melody example stop being diagnosed. Nothing a performer hears changes, because an invalid Play and a Rest are both silent.
- The Melody example's explanation of rests needs rewriting to match.
- A Monophonic Play that rests leaves its channel's voice to its own Note Off, as ADR 0075 says.

## Considered options

- **Keep ADR 0069's rule for unnested slots.** Leaves two layouts that deliver identical Cells behaving differently.
- **Rest on a half-written pair.** A half-typed Note would go silent with no explanation during Live Editing.
- **Require a Chord Play's other operands to decode on a Rest.** Diagnoses every empty row of a pattern in which voicing, inversion and length are read from the same row.

## Amendment, 2026-11-11: a root written in Source refuses a blank beside it in the Language Map

The Decision holds at the Turn, and the Language Map is narrower. The Map reads each Expression as Source holds it, so a Chord Play whose root is written in Source beside an operand left blank in Source is refused there under [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)'s rule that an unwritten operand is invalid input: the root that would excuse the blank is written. The Map diagnoses that row. A root blank in Source, or a nested Function in the root, which may clear it, excuses the blank in the Map as at the Turn.

A Map refusal holds a Turn only while every operand still holds what Source held when the Map was built. A Portal write that empties the written root during a Tick changes that operand, so the Turn runs, finds the root blank and rests: it sends nothing and raises nothing, as the Decision says. The Map rebuilt from what the Tick left admits the row, and later Ticks rest. A Portal write that leaves the root written runs the Turn too, which refuses the blank operand; once Source holds that root, the Map's refusal holds each later Turn without repeating its diagnostic.

Letting the Map admit a Chord Play with a written root beside a blank, and leaving the refusal to the Turn, would match the Decision in the Map as well, but would drop the Map's diagnostic for a blank Interval Set beside a root written in Source.
