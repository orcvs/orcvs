# 13 — Add legato to Monophonic Play

**What to build:** A musician plays legato on a mono synth: the acid voice and external mono synths
slide.

**Blocked by:** 12 (the operand type).

**Status:** resolved

- [x] `!%` takes an optional last operand, `legato`, a Number: blank or `00` retriggers (Note Off
      before Note On, as today); anything else sends the new Note On before the previous Note Off.
- [x] A legato note at the pitch it replaces keeps sounding to its new end, with no Note Off in
      between.
- [x] The reference documents `legato` and says an external synth slides only with its portamento on.
- [x] The glossary's Monophonic Play entry and signature, ADRs 0016 and 0099, and the scheduler's
      comments state the new behaviour.
- [x] Runnable reference examples cover a retrigger, a slide, and two consecutive legato notes at the
      same pitch.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Decisions

The implementer made these; the spec above did not ask for them. Each is for Toby to confirm.

- A legato note at the pitch sounding sends its Note On again with no Note Off, rather than
  nothing: it keeps the new velocity and accent, and sounds the pitch again after an aliased Raw or
  Timed Note Off. A receiver that stacks a second voice on a repeated Note On may hold one past the
  single Note Off; ADR 0016's amendment records the trade.
- A legato note on the Tick the held note's stop is due takes that stop over, delivering it after
  its Note On, so a line whose lengths equal its steps slides.
- A legato note on a silent channel starts as any note does, and velocity `00` or length `00` stops
  the channel's note whatever the `legato`.
- `legato` takes two Cells east of `length`, so a `!%` now needs twelve Cells, and a slot running
  past the row end is refused as before; on the one Grid shape that matters only at its east edge.
  Tests that put `!%` on ten-column Grids use twelve.
- `PlayCommand::Mono` carries an `Articulation` (`Retrigger` or `Legato`), not a `bool`.
- The row declares `legato: Defaulted<Number, 0x00>` directly, through the row grammar ticket 12
  widened.
