# 01 — Play a 303-style channel

Status: needs-triage

**What to build:** The Built-in Synth plays one channel as a 303-style voice, as ADR 0096 decides
it: mono, a bandlimited saw or square, a resonant 4-pole lowpass with its own decay envelope, accent
from velocity, slide on overlapping notes, and four Control Changes on that channel. The core is
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
- [ ] Reset All Controllers and All Notes Off return the channel to its load values and silence; the
      output stays finite at every controller extreme.
- [ ] Every other channel and every other controller behaves as ADR 0087 and 0088 decide.
- [ ] The Knobs example's bassline plays the acid channel, with accents and slides written in, and
      its guide says how.
- [ ] `CONTEXT.md`'s Built-in Synth entry names the channel.

## Comments

### 2026-10-31 — Opened

Prototype on branch `prototype/acid-channel` (`synth/prototype/acid/`). Which channel, and the
accent threshold, are open in ADR 0096 until Toby settles them.
