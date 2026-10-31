# 01 — Play a 303-style channel

Status: resolved

**What to build:** The Built-in Synth plays channel `09` as a 303-style voice, as ADR 0096 decides
it: mono, a bandlimited saw or square, a resonant 4-pole lowpass with its own decay envelope, accent
from velocity, slide on overlapping notes, and five Control Changes on that channel.

## Acceptance criteria

- [x] Synth core tests: a held note's spectrum darkens as its filter envelope decays; raising
      cutoff, resonance and env mod each measurably brighten it or sharpen its peak; decay lengthens
      the sweep.
- [x] An accented Note On (velocity at or above the threshold) is louder and brighter than the same
      note unaccented, and its envelope is the shortest decay.
- [x] A Note On while the channel's note is held glides to its pitch without restarting either
      envelope; a Note Off for a note that is no longer sounding changes nothing.
- [x] The saw and square do not alias audibly at the top of the range (polyBLEP or better).
- [x] Reset All Controllers (CC 121) on the channel returns its five controllers to their load values
      and leaves ADR 0088's effects alone; All Notes Off silences it; the output stays finite at
      every controller extreme.
- [x] The voice takes one of the sixteen voices while it sounds; a chord on the channel sounds as its
      last note.
- [x] Every other channel and every other controller behaves as ADR 0087 and 0088 decide.
- [x] The Knobs example's bassline moves from `02` to the acid channel `09`, with accents and slides written in, and
      its guide says how.
- [x] `CONTEXT.md`'s Built-in Synth entry and ADR 0087's Status line name the acid channel.

## Comments

### 2026-10-31 — Opened

Prototype on branch `prototype/acid-channel` (`synth/prototype/acid/`). The accent
threshold is open in ADR 0096 until Toby settles it.

### 2026-10-31 — Channel

Toby chose channel `03` for the 303 and kept the 808 on `08`, then moved the 303 to `09`, a
duplicate kit channel, so the square on `03` stays.

### 2026-10-31 — Built

Toby asked for "303 all round". The voice is `synth/src/acid.rs`, tested through `Synth`'s
`schedule` and `render`. The accent threshold stays `78`. Beyond this ticket, the Sequencer's last
voice and the Tracker's fourth voice play the bassline on `09`, and the Knobs example's knobs turn
the voice's cutoff, resonance and decay in place of the effects.
