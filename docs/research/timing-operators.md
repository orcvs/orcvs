# Timing Functions: research and proposals

2026-10-30. Research and design only; nothing here is implemented or decided. Every spelling, name
and operand order below is a proposal for Toby to accept, change or drop. A proposal that is
accepted goes through the repo's process: a ticket under `.scratch/`, a prototype on real Grid
Cells (`docs/agents/syntax-prototypes.md`), an ADR, then a test-first implementation.

Since 2026-11-14 the shipped `~*` is named Every and `~>` Slew, Clock's and Every's first
operand is `division`, and `~%` plays Tidal's Euclidean patterns rather than Orca's bucket
formula (ADR 0012); this note uses those names for what ships. The separate `~/`
Every proposed below is dropped (`.scratch/musical-terminology/decisions.md`); its section keeps
the name it was proposed under.

## Summary

- **Keep the Tick as the only unit of language time. Don't add seconds.** Every timing Function
  today is a pure function of the Tick number, the Source and its Position (ADR 0012, ADR 0013), so
  a Grid replays the same way after a Rewind and at any tempo. Seconds would tie the Source to the
  wall clock. Most of the tools surveyed count musical time (Tidal cycles, Sonic Pi beats, Elektron
  steps). Milliseconds turn up in tools with no tempo grid (Teletype `M`, Pd `metro`). Where real
  time does matter, it matters *inside* a Tick: ratchets, swing and nudges. Those are proposed
  below as fractions of a Tick.
- **"Every" is the biggest gap.** Every `~*` can only fire at phase 0, and its period tops out at
  `FF` Ticks unless you multiply. A three-operand **Every `~/ rate count phase`** covers `~*`,
  Elektron's `A:B`, Tidal's `every' n o` and Teletype's `EVERY` + `SYNC`.
- **"At"** becomes **`~@ high low`**: one Bang at an absolute Tick, like Elektron `1ST`, Bitwig
  "on First" and Sonic Pi `at`.
- **Combining rhythms is the second gap.** Orcvs can make pulses but can't AND, OR or NOT them. A
  new **Pulse operand type** (`**` or Clear) and a small **condition family `?`** (`?&` `?|` `?!`
  `?%`) would give Tidal's `mask`/`when`/`sometimesBy` and every Elektron/Bitwig trig condition by
  nesting, for example `?&~%0308~:100403` (a Euclidean rhythm only in bar 4 of every 4).
- **Hex rhythms `~# rate high low`** write a 16-step pattern as four hex digits, the way Steven
  Yi's hexbeats do. It's a natural fit for a language that already spells everything in hex.

## Where Orcvs stands today

The Tick family `~` reads the Tick number or its own last answer. Tick 0 is the first Tick of a run
or the first after a Rewind (ADR 0076). The console runs 4 Ticks per displayed BPM beat
(`orcvs/src/opts.rs`, `TICKS_PER_BEAT`), so one Tick is a 16th note and one 4/4 bar is `10` Ticks.
A Tick lasts 125 ms at the default 120 BPM, and BPM runs from 1 to 15000. A late Tick loses its
turn and the Tick grid holds (ADR 0037).

| Orcvs | Orca | What it answers |
|---|---|---|
| `~. division steps` Clock | `C` | `floor(t / division) % steps` |
| `~* division steps` Every | `D` | Bang when `t % (division × steps) == 0` |
| `~% hits steps` Euclidean | `U` | Bang on Tidal's Euclidean pattern, no rotation operand |
| `~+ amount modulus` Increment | `I` | previous + amount, kept in its Output Portal |
| `~> amount target` Slew | `Z` | moves toward target by at most amount |
| `~? seed min max` Random | `R` | deterministic per seed, Tick and Position |
| `*!` Halt | `H` | stops the root south of it for one Tick |
| `.=` Equality | `F` | Bang or Clear, the only comparison |

Here's what's missing, set against the tools surveyed below:

1. Every `~*` has no phase, so you can't express an off-beat, a backbeat or "bar 3 of 4" without
   arithmetic and Equality.
2. No period longer than `FF` Ticks (16 bars) can be written as one number. Every `~*` gets there
   only through the product `division × steps`.
3. There's no one-shot event at a given point in the performance.
4. Pulses can't be combined. A Bang can't be an operand, because a typed operand refuses `**` and a
   nested Clear leaves its parent invalid (CONTEXT.md, Clear; ADR 0073). As history: ADR 0039,
   since superseded by ADR 0063, refused a Sequence operand on Every and Euclidean and deferred
   AND versus OR for that case. ADR 0063 removed the Sequence, so the question no longer exists in
   that form.
5. There's no probability. Random answers Numbers, so a chance has to be built out of Random plus
   Equality on one value.
6. Nothing happens below one Tick: no ratchets, no swing, no micro-timing.
7. Euclidean rhythms have no rotation (Tidal `euclidOff`, the third operand of Strudel's
   `euclidRot`).
8. Triplets don't fit: 4 Ticks per beat doesn't divide by 3.

## What other tools do

### Orca (the baseline)

`C` clock, `D` delay, `U` uclid, `I` increment, `Z` lerp, `R` random. Orca's frame resolution is 4
per beat, like Orcvs. Tempo is set outside the grid, or with the `$` self-command (`bpm`, `apm`),
which Orcvs deferred as `!$` (ADR 0008).

### TidalCycles and Strudel

Time is the **cycle**, a rational number, with tempo set as cycles per second. Everything is a
pattern, a function from a time span to events, so there is no clock to tick.

- `every n f` applies `f` on cycle numbers divisible by `n`, and `every' n o f` adds an offset.
  Strudel names these `firstOf` and `lastOf`.
- `whenmod a b f` applies `f` when `cycle % a >= b`. It's the usual "last two bars of eight" fill.
- `sometimesBy p f`, `degradeBy p`, `someCyclesBy p f`, and the named shorthands `rarely`,
  `often` and `almostNever`. Randomness is seeded by time, so it replays the same way.
- `euclid`, `euclidOff`/`euclidRot` (with rotation), `euclidInv`, `euclidFull`.
- `mask`, `struct`, `when`, `inv`: boolean patterns that gate other patterns.
- `swingBy x n`, `nudge`, `early`/`late` (`<~`, `~>`): offsets within a cycle.
- `ply n` and `stut`/`echo` repeat an event inside its own span, which is a ratchet.
- `iter`, `palindrome`, `chunk`, `<a b c>` alternation, `{}` polymeter.

What carries over to Orcvs: **conditions on the cycle count**, **probability that replays**,
**boolean masks**, **rotation**, and **sub-step repeats**. What doesn't carry over: transforming a
whole pattern, because an Orcvs Function acts on one Tick and has no pattern value to act on.

### Sonic Pi

Time is **beats**, scaled by `use_bpm`, and threads run on virtual time (`sleep 0.25`). `live_loop`
plus `sync`/`cue` keep loops in phase. The `tick` and `look` counters walk rings. `spread(3, 8)` is
Euclidean. `one_in(n)` and `dice` give probability, seeded so it repeats. `at [times]` schedules
one-shots, `density n` compresses time to make ratchets, and `time_warp` shifts time. Sonic Pi
never asks for seconds unless you set BPM to 60.

### SuperCollider patterns

`TempoClock` counts beats, and `quant` and `phase` align a pattern's start to a grid. `Pbind` takes
`\dur` in beats. `Pbjorklund` makes Euclidean rhythms, `Pstutter`/`Pdup` repeat values, `Pgate`
advances a value only on a trigger, `Pif` branches, `Pfindur`/`Psync` cut or pad to a length in
beats, and `Pwrand` makes weighted choices.

### Max and Pure Data

Max's transport counts at 480 PPQ and takes note values (`4n`, `8nt` for triplets, `4nd`
dotted). `metro` can be bound to the transport or to milliseconds. `counter`, `sel`, `delay` and
`pipe` build "every N" and "after N". Pd counts in milliseconds (`metro`, `delay`, `pipe`,
`timer`), because it has no musical clock of its own. Both offer real time *as well as* musical
time, and their musical time is finer than a 16th (480 PPQ, triplets).

### Elektron trig conditions

A condition sits on one step of a pattern:

- `A:B` fires on the A-th of every B pattern loops (`1:2`, `3:4`, `8:8`).
- `X%` fires with a probability.
- `FILL` / `!FILL` follow a performer-held Fill mode.
- `PRE` / `!PRE` fire if the previous condition on the track was true, or false.
- `NEI` / `!NEI` do the same for the neighbouring track.
- `1ST` / `!1ST` fire on the first loop only, or on every loop but the first.
- Also on each step: retrigs (rate plus a velocity curve), micro-timing in 1/384-note steps,
  per-track scale (3/4×, 1.5× and so on, for triplet feels), and global swing.

### Bitwig operators

Bitwig puts four operators on each note:

- **Chance** (0–100%, seeded per clip).
- **Repeats**: a count, or a beat rate, with a curve and a velocity ramp.
- **Occurrence**: First, Not first, With/Without previous (note, key or channel), Fill on/off.
- **Recurrence**: a 1–8 loop cycle with one on/off toggle per loop, which is a bitmask over the
  loop count.

### monome Teletype

Teletype's conditions count **calls**, not time:

- `EVERY X` runs on the X-th call, and `SKIP X` runs on every call but the X-th.
- `OTHER` runs when the last `EVERY`/`SKIP` didn't, and `SYNC X` sets those counters.
- `PROB` gives probability.
- The milliseconds here (`M` metro, `DEL`, `TIME`, `LAST`) are there because Teletype has no tempo,
  and `BPM x` converts a tempo into milliseconds.

### Hexbeats (Steven Yi)

Steven Yi's hexbeats write a rhythm as hex digits, four steps per digit, one bit per step: `8` is
`X...` and `f` is `XXXX`. He uses them in csound-live-code. One byte is eight steps, so a pair of
Orcvs Numbers is a 16-step bar.

### Eurorack conventions

Clock dividers and multipliers, burst/ratchet generators, swing modules and the Turing Machine
(randomness you can lock).

## Proposals

Each proposal gets a spelling, a name, its operands, what it answers and where it comes from. All
operands are two-Cell Numbers. Unless a proposal says otherwise, it runs every Tick, answers a Bang
(`**`) or a Clear like Every `~*` and Euclidean do, diagnoses a zero divisor, and does its arithmetic in
a wider integer. `t` is the absolute Tick.

### 1. Tick pulses, in the `~` family

#### `~/` Every: `~/ rate count phase`

Bangs on step `phase` of every cycle of `count` steps, each step `rate` Ticks long, on the first
Tick of that step:

```text
t % rate == 0  and  (t / rate) % count == phase
```

A `phase` of `count` or more diagnoses, as do a `rate` or `count` of `00`.

- `~/010400`: every 4 Ticks, on the beat.
- `~/040201`: beats 2 and 4 (the backbeat), `t % 8 == 4`.
- `~/100403`: the first Tick of bar 4 of every 4 bars. That's Elektron `4:4`, or Tidal
  `every' 4 3`.
- `~/010200` is `~*0102`, so `~*` becomes a special case. Keep `~*` for Orca parity, or
  retire it; that's Toby's call.

The `/` reads as "divides". It's the clock-divider idea. Sources: Orca `D`, Elektron `A:B`, Tidal
`every'`, Teletype `EVERY` + `SYNC`, SuperCollider `quant`/`phase`.

#### `~@` At: `~@ high low`

Bangs once, on Tick `high × 100 + low` (hex), and answers a Clear on every other Tick. Two Numbers
reach Tick `FFFF`, about 136 minutes at 120 BPM.

- `~@0000` fires on the first Tick after a Rewind (Elektron `1ST`, Bitwig "on First"). Use it to
  set things up.
- `~@0100` fires at bar 17, so you can schedule a section change.

`@` reads as "at". Sources: Sonic Pi `at`, Elektron `1ST`, Bitwig Occurrence "on First".

#### `~#` Hex rhythm: `~# rate high low`

Steps through a 16-step bit pattern, one step every `rate` Ticks, most significant bit first, and
Bangs on the first Tick of each step whose bit is set:

```text
step = (t / rate) % 10 (hex)
Bang when t % rate == 0 and bit (F − step) of (high × 100 + low) is 1
```

- `~#018888`: `X...X...X...X...`, four on the floor.
- `~#010808`: `....X.......X...`, the backbeat.
- `~#019228`: `X..X..X...X.X...`, son clave.

You can type in the rhythm as you read it, one Cell per four steps, and change it by retyping a
digit. A Write can drive it, so another part of the Grid can rewrite the rhythm. Sources: Steven
Yi's hexbeats, Bitwig Recurrence's toggles, Teletype's bitmasked `DEL.B`.

#### `~:` During: `~: rate count phase`

The gate form of Every. It Bangs on **every** Tick of step `phase`, not only the first:

```text
(t / rate) % count == phase
```

On its own, it rolls 16ths through a window. Its real use is as a Pulse operand (section 2):
`?&~%0308~:100403` plays the Euclidean rhythm only during bar 4 of every 4.

An alternative reading follows Tidal `whenmod`: open while `(t / rate) % count >= phase`, so
`~:100806` is "the last two bars of eight". The one-step reading combines more simply with
`?|`/`?!`, and the `whenmod` reading is the fill musicians usually want. **Toby to choose.**
Sources: Tidal `whenmod`, `within`, Elektron `A:B` on a whole bar.

#### Euclidean rotation

There are two ways to spell rotation:

- **(a)** `~% hits steps rotation`, an extra operand on the existing Function. The language is
  pre-release, so this is design, not breakage. It changes every existing `~%` in the examples.
- **(b)** A new rotated Euclidean Function that leaves `~%` as it is.

I recommend **(a)**, because one Euclidean with a rotation is the simpler vocabulary. Sources:
Tidal `euclidOff`, Strudel `euclidRot`, SuperCollider `Pbjorklund` offset.

### 2. A Pulse operand and a condition family `?`

There's a precedent for the Pulse operand. ADR 0108 makes a Play's note `Optional<Note>`, an
operand type that admits an unwritten value, so a nested Clear there binds to "no note" and does not
leave the parent invalid. A **Pulse** operand type would follow the same pattern: it accepts `**`
or a Clear, written literally or returned by a nested Function, and reads them as on and off. Today
a typed operand refuses `**` (CONTEXT.md, Bang).

The `?` family would hold the Functions whose operands are Pulses. They answer a Bang or a Clear.
No spelling begins with `?` today.

| Spelling | Name | Answers a Bang when | Sources |
|---|---|---|---|
| `?& a b` | And | both are on | Tidal `mask`, `when`, `struct`, Elektron conditions |
| `?\| a b` | Or | either is on | layering two rhythms |
| `?! a` | Not | `a` is off | Tidal `inv`, Teletype `SKIP`, Elektron's `!` conditions |
| `?% seed chance a` | Chance | `a` is on and a die roll passes | Tidal `degradeBy`, Elektron `X%`, Bitwig Chance, Teletype `PROB` |

`?% seed chance a` rolls like Random `~?`, from the seed, the Tick and its own Position, so a
chance replays the same way after a Rewind (ADR 0013). It draws `r` from `00` to `FE` and passes
when `r < chance`, so `00` never passes and `FF` always does. Teletype had to fix exactly this
off-by-one in `PROB 100`.

Examples:

- `?%0180~/010200`: an 8th-note hat that plays about half the time.
- `?&~#01FFFF?!~:100403`: 16ths everywhere except bar 4 of 4.
- `?|~/040400~%0510`: the downbeat layered with a 5-in-16 Euclidean.

Two questions an ADR has to settle:

1. Under ADR 0061 a nested Function that answers a Bang writes `**` into its slot and activates the
   roots aligned with it. Should a pulse returned into a Pulse operand still activate those
   neighbours, or only feed its parent?
2. Should `?` Functions that read the Tick, such as Chance, sit in `?` or in `~`? I put Chance in
   `?` because its operand is a Pulse.

Elektron `PRE`/`NEI` and Bitwig "with previous" need no new Function. A Bang is visible Source, so a
Read (`&^`, `&<`) of another Function's output pair is a Pulse operand.

### 3. Counting Bangs, not Ticks

#### `*+` Count: `*+ step modulus`

An Increment that's inert until a Bang reaches it. When a Bang arrives, it reads the Number in its
Output Portal, adds `step` modulo `modulus`, and writes the result. Without a Bang it answers
nothing, so its last write stands. Combined with Equality it gives "every Nth time this happened",
for example every third time a `>>` hits a wall. That's what Teletype's `EVERY` and Elektron's loop
counter do, but it counts events, not time.

Lower priority: `~/` covers every case where the event is a point in time.

### 4. Tick-derived Numbers

#### `~^` Bounce: `~^ rate peak`

A Clock that goes up and back down: `00 01 … peak … 01 00 …`, one step every `rate` Ticks:

```text
s = (t / rate) % (2 × peak)
answer s if s <= peak, else 2 × peak − s
```

Use it for Control Change sweeps and ping-pong walks along a Track. Sources: Tidal `palindrome`,
Sonic Pi `.mirror`, Max `phasor~`.

These are already covered by composition, so no Function is proposed for them:

- Polymeter: two Clocks with different moduli.
- Alternation (`<a b c>`): Track `&t` indexed by a Clock.
- Tidal `iter`: Clock plus Add.
- Sonic Pi `tick`/`look`: Clock or Increment.
- SuperCollider `Pseq`/`Prand`: Track, or Random indexing a Track.

### 5. Inside a Tick (where real time belongs)

These need Playback to schedule MIDI at an instant inside a Tick period, which it doesn't do yet.
Note Offs are scheduled on whole Ticks. Each of these is a Playback change, so each carries a
concurrency risk and needs a benchmark.

#### `!r` Ratchet: `!r channel velocity note length count`

Plays `count` notes spaced evenly across `length` Ticks, each one a Timed Play `length / count`
Ticks long. When `count` is more than `length`, the notes fall between Ticks. Sources: Elektron
retrigs, Bitwig Repeats, Tidal `ply`, Sonic Pi `density`, Eurorack bursts.

#### Swing (a Playback setting)

Delays every second Tick by a fraction of a Tick period, set like BPM (Panel or `config.toml`). It
applies to the whole Grid, the way Elektron, the MPC and Ableton's groove do. A per-note nudge can
come later if one global swing turns out not to be enough. Sources: Tidal `swingBy`, Elektron
swing, Ableton's groove pool.

#### `!t` Tempo: `!t high low`

Sets the BPM to `high × 100 + low` when a Bang reaches it. `!t0078` is 120. Slew into its
low operand gives an accelerando. Tempo changes only the wall clock, so replay stays deterministic
in Ticks. This is Orca's `$bpm`, and it's related to the deferred `!$` (ADR 0008).

#### `?f` Fill

Bangs on every Tick while the performer holds a Fill control on the Panel. The control would be a
Grid Cell, like the other controls. It's a new explicit Tick input, in the same way the Tick is
one. Combine it with `?&`. Sources: Elektron `FILL`, Bitwig "Fill on/off".

### 6. Triplets and Tick resolution

At 4 Ticks per beat you can't play a triplet. Every tool surveyed has a way to do it: Max's `8nt`,
Elektron's per-track scale, Tidal's arbitrary subdivisions, Sonic Pi's fractional sleeps. There are
three options:

- **(a)** Make Ticks per beat a Playback setting (4, 6, 8, 12 or 24), like BPM. It's cheap, but
  every Grid's speed then depends on that setting.
- **(b)** Keep 4 and give triplets to Ratchet, by playing 3 notes across 2 Ticks.
- **(c)** Do nothing, which keeps parity with Orca.

I lean towards **(b)** first, because it needs no new global state, and **(a)** only if a real Grid
asks for it. **Toby to choose.**

## Seconds, decided against

The ask was "At Tick or Sec?". I recommend Ticks, for four reasons:

1. **Determinism.** ADR 0012 makes the Tick an explicit input, and with it a Grid replays exactly
   after a Rewind or a stop. A seconds-based Function would answer differently after a tempo
   change, a missed deadline (ADR 0037) or a stop.
2. **Resolution.** A Tick is 125 ms at 120 BPM, so a seconds operand would be rounded to Ticks
   anyway. Finer time belongs inside a Tick (section 5).
3. **Precedent.** Musical tools count musical time: cycles, beats, steps and loops. Milliseconds
   come from tools that have no tempo (Teletype, Pd), and Teletype immediately adds `BPM` to
   convert.
4. **Tempo is already the conversion.** If a performer wants "every 2 seconds", that's
   `~/` at a known BPM, and the proposed `!t` puts the BPM into the Source.

An installation or a timer that has to follow the wall clock would need a separate "wall clock"
input. Nothing in the current product needs one.

## Suggested order

1. Every `~/`, At `~@`, Hex `~#`: pure Tick functions on the existing machinery.
2. The Pulse operand plus `?&` `?|` `?!` `?%`. This needs an ADR (Pulse operand, nested activation,
   family boundary).
3. During `~:`, Bounce `~^`, Euclidean rotation, Count `*+`.
4. Ratchet `!r`, Swing, Tempo `!t`, Fill `?f`, triplets. These are Playback and console work.

## Decisions for Toby

1. Ticks only, no seconds: confirm.
2. A new `?` family and a Pulse operand type: yes or no.
3. Keep `~*` beside Every `~/`, or retire it.
4. Rotation as a third `~%` operand, or as a new Function.
5. During `~:` as one step, or as Tidal's `whenmod` "from step N".
6. Triplets through Ratchet first, or a Ticks-per-beat setting.

## Sources

- Orca operators: <https://scrapbox.io/orca/ORCA_Operators>
- Bitwig Operators user guide: <https://www.bitwig.com/userguide/latest/operators/>
- Elektron trig conditions (Octatrack OS, CDM):
  <https://cdm.link/new-elektron-octatrack-os-trig-conditions-awesome/>; Digitakt unofficial
  guide: <https://www.elektronauts.com/t/digitakt-unofficial-guide/40705>
- monome Teletype manual: <https://monome.org/docs/teletype/manual>
- Strudel random and conditional modifiers (docs mirror):
  <https://deepwiki.com/blakkd/strudel_documentation/4.5-random-and-conditional-modifiers>
- TidalCycles userbase, `every`, `sometimesBy`, `someCyclesBy`, `euclid`, `euclidInv`:
  <https://userbase.tidalcycles.org/>
- SuperCollider `Quant` and `Psync`: <https://docs.supercollider.online/Classes/Quant.html>,
  <https://docs.supercollider.online/Classes/Psync.html>; Pattern Guide 060, filter patterns:
  <https://docs.supercollider.online/Tutorials/A-Practical-Guide/PG_060_Filter_Patterns.html>
- Sonic Pi community examples of `spread`, `at` and rings: <https://in-thread.sonic-pi.net/>
- Orcvs: `docs/functions.md`, ADRs 0012, 0013, 0019, 0037, 0039, 0061, 0073, 0075, 0076,
  `CONTEXT.md` (Bang, Clear), `orcvs/src/opts.rs`.

The details for Tidal `whenmod`/`swingBy`/`ply`, Sonic Pi, Max/Pd, SuperCollider pattern classes
other than those linked, Elektron's `!`-prefixed conditions, micro-timing and scale, and Steven
Yi's hexbeats come from working knowledge of those tools, not from pages fetched for this note.
