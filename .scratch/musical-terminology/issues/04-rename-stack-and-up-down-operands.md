# 04 — Rename Stack's and Up/Down's operands

**What to build:** A musician reads Stack's position as an index, with the scale degree beside it,
and knows Up and Down count semitones.

**Blocked by:** 01 (the settled names).

**Status:** resolved

- [x] Stack's `degree` Role is `index`; Up's and Down's `number` Role is `semitones`, in Info, the
      reference signatures and prose, and the diagnostic that names Stack's position.
- [x] Every Pick and Stack example in the reference gives the musician's one-based degree beside the
      zero-based index (index `04` of C major is G, the 5th, V).
- [x] The glossary no longer both avoids "Degree" and says Pick picks a scale degree: `index` names
      the operand in Pick and Stack, and "degree" is kept for the one-based explanation.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 14 November 2026. The Function table in `lang/src/atom.rs` names Stack's position
`index` and Up's and Down's operand `semitones`, so Info and the console's Function Reference
show them, and the diagnostic Stack gives when no name fits reads "from index 00". In
`docs/functions.md` the signatures follow the table, as `function_reference_doc` checks, and each
Pick and Stack example gives the one-based degree beside the index (index `04` of C major is G, the
5th degree, V). `CONTEXT.md`'s Up, Down, Pick and Stack entries use the new names; Pick's
_Avoid_ keeps "Degree" off the operand and keeps it for the one-based count.
A dated amendment to ADR 0105 names Stack's operand `index`, since its Decision says Stack takes
"a degree". `docs/research/chords-and-patterns.md` keeps `degree`, since it records a proposal.
