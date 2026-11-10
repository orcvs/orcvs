# Interval Set and Voicing are literal types

Status: accepted. Adds two Atoms beside Number, Note, Bang and Function. Settled in `.scratch/chords-and-pitch/`.

A Chord must be drivable by a Clock and a Read or Track, so what follows its root has to be a pair of Cells that those Functions can carry and a Chord Play can decode. Today a Copy, Read or Track refuses `M7`, because it is neither a Function, a Number nor a Note.

## Decision

**An Interval Set is one literal type for chord and scale names.** It names a list of semitones above a root. There are 39 names and no two name the same list.

- Chords: `M_ M7 M9 MD M6 69 +M m_ m7 mM m6 m9 mB mD o_ o7 h7 b9 7_ 9_ 13 s2 s4 +_ a9`.
- Scales: `MA mi do ph ly mx lo hm mm p5 p3 bl wt ch`.

**Every name fills both Cells.** A half-blank pair is refused, so a single-character chord name takes the filler `_`. Names follow three rules: drop an implied 7 (`mM`, `+M`, `b9`), use an established symbol (`h7`, `69`), and write 11 and 13 as the hexadecimal digits `B` and `D` (`mB`, `mD`, `MD`). `13` is the dominant 13th itself.

**A scale name is valid wherever a chord name is.** `C4MA` in a Chord Play sounds all seven tones. No rule is added against it, because one type is what lets Pick and Stack serve both.

**A Voicing is a separate literal type with eight names.** `CL` close, `OP` open, `D2`, `D3`, `D5`, `D6` drops, `SH` shell and `RL` rootless. Inversion is a Number operand, not part of the Voicing.

**Both are carried as Numbers and Notes are.** A Copy, Read, Track, Write or Push moves the two Cells and the receiving operand decodes them, as [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) says of any Return. `69` and `13` are also Numbers and `D2` is also a Note, which costs nothing because an Operand Literal has no type outside its operand position.

**Stack is the Function that answers one.** It takes an Interval Set, a degree and a size and answers the name of the Interval Set those tones form.

## Consequences

- An Atom can now be an Interval Set or a Voicing, and the Language Map and Source Paint gain an operand type for each.
- `mM` (a minor-major chord) and `mm` (the melodic minor scale) differ only by case. A slip between them is a valid different Interval Set and is not diagnosed. That is accepted to keep `mm` as the name.
- Stack diagnoses when no name fits. C major pentatonic stacked from C gives C, E and A, which no Interval Set names.

## Considered options

- **A chord type and a separate scale type.** Pick would need two Functions or an operand accepting either, and Stack's usual input and output would be different types.
- **Sharing the Note operand.** `C7` is already a Note, and an operand's width must be fixed at parse.
- **Half-blank names such as `M `.** A half-blank pair is partial input everywhere else.
