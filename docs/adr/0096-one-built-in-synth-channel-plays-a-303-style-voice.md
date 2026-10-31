# One Built-in Synth channel plays a 303-style voice

Status: proposed. Opened with Toby on 2026-10-31 in `.scratch/acid-channel/`. It would amend
[ADR 0087](0087-the-site-plays-the-console-through-a-built-in-synth.md), whose channels share one
envelope and which ignores every Control Change but All Sound Off and All Notes Off: on the acid
channel the synth would also act on CC 70, 71, 74, 75 and 79 and on Reset All Controllers (CC 121).
[ADR 0088](0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md)'s global
effects stay as they are, and CC 121 leaves them alone.

Acid basslines are the music a reader expects a sequencer like Orcvs to make, and the Built-in
Synth cannot make one. A 303's sound is its filter: a resonant lowpass swept down by an envelope on
every note, deepened by accent and tied across notes by slide. The synth has no filter, no
per-note envelope but loudness, and no voice that slides.

## Decision

**Channel `09` plays a 303-style voice.** ADR 0087 gives `08` to `0F` the same 808 kit, so `09`
adds nothing today; the kit stays on `08` and `0A`–`0F`, and every tone channel keeps its sound.
Toby chose `09` on 2026-10-31.

- **One voice, mono.** It takes one of ADR 0087's sixteen voices while it sounds, and a chord
  played on the channel sounds as its last note, legato from the one before. A Note On while the channel's note is still held is legato: the pitch glides
  to the new note over about 60 ms and neither envelope restarts. A Note On with nothing held starts
  both envelopes. A Note Off for a note that is no longer the one sounding changes nothing, so a
  Timed Play's late Note Off cannot cut the note that replaced it.
- **A bandlimited saw or square** (polyBLEP), chosen by a Control Change.
- **A resonant 24 dB/oct lowpass,** four saturating one-pole stages with feedback. Resonance is
  capped below self-oscillation, as a 303's is.
- **A filter envelope** that jumps on each new note and decays exponentially. Its decay and depth are
  Control Changes.
- **Accent:** a Note On at or above velocity `78` is accented: louder, its filter opened further,
  and its envelope at the shortest decay. Below that, velocity sets loudness as it does on every
  other channel. _Open: the threshold. `78` leaves `64`, `77` (Orca's default `f`) and the examples'
  velocities unaccented and makes `7F` accented._
- **Its Control Changes, on that channel only**, after General MIDI's sound controllers:

  | CC | Sets | At load |
  | --- | --- | --- |
  | 70 | waveform: below `40` saw, from `40` square | `00` |
  | 71 | resonance | `50` |
  | 74 | cutoff | `30` |
  | 75 | envelope decay | `30` |
  | 79 | envelope depth (env mod) | `50` |

  The load values make a recognisable acid sound with no Control Change at all. Reset All
  Controllers (CC 121) on the channel, which Stop's safety action sends, returns these five to their
  load values; it does not touch ADR 0088's global effects.

**The language needs nothing new.** A Control Change every Tick turns the knobs, as the Knobs
example turns the effects. Velocity carries accent. A Timed Play whose length runs past its step
overlaps the next note, which is how a slide is written. Monophonic Play stops the prior note before
it starts the next one, so it never slides.

## Rejected alternatives

**A tone channel's slot:** `02`, the saw the Knobs bassline already plays, or `03`, the square. Either
loses a sound the synth has, while `09` only repeats the kit on `08`.

**A Program Change that turns any tone channel into the acid voice.** A Source would have to send it
before a run and again after every Stop, because the safety action resets the channel. The synth
ignores Program Change today (ADR 0087: every other message is ignored).

**Accent and slide as Control Changes,** as some 303 clones take them. A Source would send a CC
before each Play, which doubles every step. Velocity and note overlap already say it.

**A filter on every tone channel.** It is more than the ask. One channel proves the voice first.

**Sample playback of a 303.** The synth has no samples (ADR 0087), and a 303's sound is the filter
moving, which a sample cannot follow.

## Consequences

- Channel `09` no longer plays the kit. No shipped example sounds on `09`: the Sequencer ships its
  `09` voice empty, so a reader who fills it hears the acid voice. The Knobs bassline moves to `09`
  from `02`.
- The core gains one voice type and a per-channel controller state; every other channel keeps one
  shared envelope and ignores these controllers.
- The Sound Module shows the channel's notes as it shows any channel's; it does not show the acid
  controllers. Showing them is the Sound Module's own decision.
- A MIDI device on the same channel reads CC 70 to 79 as General MIDI's sound controllers: 71, 74
  and 75 are resonance, cutoff and decay on most synths, and 70 and 79 vary by device. A Source
  written for the acid channel usually only changes timbre elsewhere.
- On acceptance, ADR 0087's Status line, `CONTEXT.md`'s Built-in Synth entry ("an 808-style kit
  ... on channels `08`–`0F`") and `examples/knobs.md` ("the Built-in Synth's saw") name the acid
  channel `09`.
