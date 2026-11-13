# 04 — Rename Stack's and Up/Down's operands

**What to build:** A musician reads Stack's position as an index, with the scale degree beside it,
and knows Up and Down count semitones.

**Blocked by:** 01 (the settled names).

**Status:** ready-for-agent

- [ ] Stack's `degree` Role is `index`; Up's and Down's `number` Role is `semitones`, in Info, the
      reference signatures and prose, and the diagnostic that names Stack's position.
- [ ] Every Pick and Stack example in the reference gives the musician's one-based degree beside the
      zero-based index (index `04` of C major is G, the 5th, V).
- [ ] The glossary no longer both avoids "Degree" and says Pick picks a scale degree: `index` names
      the operand in Pick and Stack, and "degree" is kept for the one-based explanation.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
