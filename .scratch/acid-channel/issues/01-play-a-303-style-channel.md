# 01 — Play a 303-style channel

Status: needs-triage

**What to build:** The Built-in Synth plays channel `03` as a 303-style voice, as ADR 0096 decides
it: mono, a bandlimited saw or square, a resonant 4-pole lowpass with its own decay envelope, accent
from velocity, slide on overlapping notes, and five Control Changes on that channel. The core is
shared, so the web worklet and native both get it.

## Acceptance criteria

- [ ] Synth core tests: a held note's spectrum darkens as its filter envelope decays; raising
      cutoff, resonance and env mod each measurably brighten it or sharpen its peak; decay lengthens
      the sweep.
- [ ] An accented Note On (velocity at or above the threshold) is louder and brighter than the same
      note unaccented, and its envelope is the shortest decay.
- [ ] A Note On while the channel's note is held glides to its pitch without restarting either
      envelope; a Note Off for a note that is no longer sounding changes nothing.
- [ ] The saw and square do not alias audibly at the top of the range (polyBLEP or better).
- [ ] Reset All Controllers (CC 121) on the channel returns its five controllers to their load values
      and leaves ADR 0088's effects alone; All Notes Off silences it; the output stays finite at
      every controller extreme.
- [ ] The voice takes one of the sixteen voices while it sounds; a chord on the channel sounds as its
      last note.
- [ ] Every other channel and every other controller behaves as ADR 0087 and 0088 decide.
- [ ] The Knobs example's bassline moves from `02` to the acid channel `03`, with accents and slides written in, and
      its guide says how.
- [ ] `CONTEXT.md`'s Built-in Synth entry and ADR 0087's Status line name the acid channel.

## Comments

### 2026-10-31 — Opened

Prototype on branch `prototype/acid-channel` (`synth/prototype/acid/`). The accent
threshold is open in ADR 0096 until Toby settles it.

### 2026-10-31 — Channel

Toby chose channel `03` for the 303 and kept the 808 on `08`.
