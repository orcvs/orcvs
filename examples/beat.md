# Beat

`beat.orcvs` plays one bar of a beat on the Built-in Synth's 808-style kit, on channel `09`: MIDI
channel 10, the General MIDI drum channel. Open it with **Help > Examples > Beat**, or as
`?example=beat`. Press Space to start Playback.

```text
col  0         1
     012345678901
  0
  1  || kick ||
  2  ~*0104
  3    !~097FC201
  4
  5  || snare ||
  6    ~.0108
  7  .=0004
  8    !~0964D201
  9
 10  || hat ||
 11  ~*0102
 12    !~0940f201
```

Each instrument stands under its own label, so a phone shows all three.

## What each part does

| Cells | Function | Role |
| --- | --- | --- |
| (0, 2) `~*0104` | Delay, rate `01`, modulus `04` | Bangs every fourth Tick, on each beat. |
| (2, 3) `!~097FC201` | Timed Play | The kick: Note `C2` at velocity `7F`, held one Tick. |
| (2, 6) `~.0108` | Clock, rate `01`, modulus `08` | Counts `00` to `07` once a Tick and writes the count into Equality's `left`. |
| (0, 7) `.=0004` | Equality | Bangs when the count is `04`: beats 2 and 4. The shipped `00` is overwritten on the first Tick. |
| (2, 8) `!~0964D201` | Timed Play | The snare: Note `D2` at velocity `64`. |
| (0, 11) `~*0102` | Delay, rate `01`, modulus `02` | Bangs every second Tick, on each eighth. |
| (2, 12) `!~0940f201` | Timed Play | The closed hat: Note `f2` (F#2) at velocity `40`. |

At the default 120 BPM a Tick is a sixteenth note, so a bar is sixteen Ticks. The kit plays General
MIDI drum keys: `C2` is key 36, the kick, `D2` key 38, the snare, and `f2` key 42, the closed hat.
The kit answers a few more keys, such as the pedal hat `g2` and the crash `c3`, which the reference's
Built-in Synth section lists; every other key is silent.

The Clock stands on its own row rather than nested in Equality's `left`: a nested Function still
writes its answer one row south of its anchor, which would land on the snare's Timed Play.
