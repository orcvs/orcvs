# Run Clock Render Frames without a second Tick clock

## Problem Statement

While Playback is Playing, the Run Clock Readout is wall-clock of the current run and must change
its `mm:ss` glyphs once a second. The Panel currently Paints that Readout only when a Tick is
published or a Cursor Effect asks for a Render Frame. At 1 BPM a Tick lasts 15 seconds, so a
performer can watch the clock sit still and then jump. Ticket 01 asked for a Render Frame every
second while Playing; the console never grew that wake. A later test then forbade starting a Tick
period from this Render Frame, which is right — that would be a second musical clock — but it left
no honest wake for the Run Clock.

## Solution

Each Render Frame samples Run Clock once and uses that value for both the Readout text and the next
wake. A console function, `until_next`, turns that sample, the Cursor Effect remaining delay, and
this pass's predicted frame time into one optional delay. The Run Clock term is the remainder until
the painted glyphs would change, plus that predicted frame time, so egui's early-wake subtraction
lands on the second boundary instead of starting a burst of extra Render Frames. Publish still
wakes immediately when `PlaybackObservation` changes. The function never reads BPM and never starts
a Tick period from this Render Frame.

## User Stories

1. As a performer, I want the Run Clock to advance once a second while Playing, so that `mm:ss` is wall-clock of this run rather than a count of Ticks.
2. As a performer playing at 1 BPM, I want the Run Clock to move during the 15-second Tick, so that a slow tempo does not freeze the clock.
3. As a performer playing at 120 BPM, I want the Run Clock to stay honest between Ticks, so that a fast tempo does not wait on Cursor Effect animation to show the next second.
4. As a performer, I want Tick to change when the engine publishes, so that the integer on the Panel is the engine's Tick rather than a timer the console invented.
5. As a performer, I want a quiet Render Frame not to start a Tick period from now, so that `**` and Tick cannot land late or twice.
6. As a performer, I want Tick to still jump to the new value as soon as a Tick publishes, so that the Panel does not wait out a remainder to show it.
7. As a performer, I want Run Clock to hold when I stop, so that a stopped run does not keep spending wall-clock.
8. As a performer, I want Run Clock to reset when I play again, so that a new run starts at `00:00`.
9. As a performer, I want Run Clock to stay `00:00` before the first run, so that rest is not mistaken for a run that has begun.
10. As a performer, I want an Overrun not to invent skipped time on Run Clock, so that a stall the engine declined does not look like it played.
11. As a performer, I want Run Clock to pass `59:59` into `h:mm:ss` on the next whole second, so that a long run keeps one-second honesty after the format widens.
12. As a performer, I want the Cursor Effect to keep moving on its own cadence, so that presentation time stays independent of Playback.
13. As a performer using reduced motion, I want Run Clock to keep advancing while Playing, so that turning the Cursor Effect off does not freeze the clock.
14. As a performer with the Cursor off-screen, I want Run Clock to keep advancing while Playing, so that a panned-away Cursor does not freeze the clock.
15. As a performer at 60 BPM, I want the next Run Clock Paint not to be "one second from this Render Frame", so that a 250 ms Tick is not secretly driven by a 1 s metronome started here.
16. As a performer, I want BPM, destination, and Tick Readouts to keep their current behaviour, so that this work only makes Run Clock honest.
17. As a performer on WASM, I want the same Run Clock honesty as on native, so that the browser console is not a second clock.
18. As a performer, I want the second the Panel paints to be the second it schedules from, so that `00:01` cannot vanish between two clock reads in one Render Frame.
19. As a performer, I want one Render Frame when the second changes, so that egui waking a frame early does not redraw back to back until the glyphs move.
20. As a maintainer, I want the Cursor Effect branch to return its delay rather than request a Render Frame itself, so that a second timed request cannot remain beside `until_next`.
21. As a maintainer, I want `until_next` to run after CentralPanel, so that both remainders exist before any timed request.
22. As a maintainer, I want that function to sit in the console, so that the toolkit-free crate does not take a Paint wake.
23. As a maintainer, I want tests to pin remainder arithmetic on the shipped function with chosen Durations, so that CI does not sleep a Tick and does not test a sibling of what ships.
24. As a maintainer, I want the existing publish-ZERO test to keep passing, so that Tick honesty stays a watch, not a delay.
25. As a maintainer, I want a Playing pass with Cursor Effect off and a Tick period longer than one second to request a delay of at most one second after egui's subtraction, so that the failing-first run cannot hide behind Cursor Effect wakes.
26. As a maintainer, I want `until_next(Some(900ms), None, 0)` to answer `Some(100ms)`, so that remainder is not one second from now.
27. As a maintainer, I want `until_next(None, …, …)` to contribute no Run Clock delay, so that a frozen Readout, a missing origin, or rest does not schedule a moving-clock wake.
28. As a maintainer, I want `until_next(Some(1s), None, 0)` to answer `Some(1s)`, so that this Render Frame — which already shows the new glyphs — does not spin at zero delay.
29. As a maintainer, I want the Run Clock term to add this pass's predicted frame time and the Cursor Effect term not to, so that nobody later strips the compensation as double-counting.
30. As a maintainer, I want a Stopped pass with Cursor Effect off, after settling, to request no timed wake, so that rest does not wake every second.
31. As a maintainer, I want Cursor Effect remaining delay to stay an input, so that Cursor Effect cadence is not reimplemented inside the wake function.
32. As a maintainer, I want no BPM on the wake interface, so that a Tick Grid period cannot become the delay source.
33. As a maintainer, I want no new wake port until tests leave the console pass, so that one egui Context does not become a hypothetical seam.

## Implementation Decisions

- The timed seam is a free function in the console. Publish wake stays the existing watch loop at
  construction. `Context` is already shared with that task; this work does not change that. There
  is no unit struct: associated functions would only have been a namespace.
- Shape. `until_next` takes no egui types. The call site reads `predicted_dt` and passes it as a
  Duration:

  ```rust
  fn until_next(
      run_clock: Option<Duration>,
      cursor_effect: Option<Duration>,
      predicted_dt: Duration,
  ) -> Option<Duration>
  ```

- The console samples Run Clock once per Render Frame and uses that value for both the Readout
  text and `until_next`. A second `run_clock()` call later in the same pass is forbidden: it can
  paint `00:00` from 0.9995 s and then schedule from 1.0002 s, skipping `00:01`. The predicted-frame
  offset is applied to that same sample, so the compensated wake is computed against the glyphs the
  Panel drew.
- `run_clock` is `Some` only while the clock is moving: Playing with `run_started_at` set. Stopped,
  rest, and Playing with no origin pass `None`. Playing with no origin must not schedule a
  one-second wake against the frozen value `run_clock()` would fall back to.
- `until_next` never reads `PlaybackObservation`, BPM, Tick Grid, or this Render Frame's age.
  Overrun has no branch: the engine does not publish, and a moving clock is still the sampled
  Duration.
- Remainder is display honesty, not a metronome started from this Render Frame. `Some(run)` answers
  time until the next whole second of that Duration. At an exact whole second the remainder is one
  second, not zero: this Render Frame already shows the new glyphs. `None` means no timed wake from
  that argument.
- egui 0.36.1 subtracts predicted frame time from every `request_repaint_after` delay (default
  1/60 s; native eframe uses the monitor refresh). A remainder aimed at the next whole second
  therefore wakes about one frame early, while C still shows the old second; that pass then asks
  for the leftover, egui subtracts again, and the console redraws back to back until the glyphs
  change. The Run Clock term adds `predicted_dt` so that subtraction lands on the boundary. The
  Cursor Effect term is passed through unchanged. Do not remove this offset later as redundant:
  it is what preserves one Render Frame per second.
- Cursor Effect remaining delay is computed inside CentralPanel. The visibility check
  (`effect_bounds` intersects the console area) stays there and decides whether that `Option` is
  `Some`. That branch does not request a Render Frame. `until_next` runs once after CentralPanel.
  The console then makes one `request_repaint_after` from `until_next`'s `Option`. It does not also
  request one second from now, and it does not request a Tick period from now. Diagnostics is a
  window in the root viewport, so that root delay covers it.
- `until_next` may return the sooner of the compensated Run Clock term and the Cursor Effect term
  so the call site has one timed request. That is ownership of the call, not what makes the wake
  honest: `request_repaint_after` already keeps the soonest delay if both were requested separately.
- No `Wake` port. Production and the console-pass tests share the real context. WASM needs no
  extra branch: eframe's web runner reads the same delay.
- Ticket 01's Answer still stands as intent (`mm:ss` moves while Playing). The literal "one second
  from this Render Frame" is refused because at 60 BPM that duration is a Tick Grid coincidence
  waiting to happen. Accepting egui's early-wake burst is refused: at an uncapped rate it is a
  busy loop.

## Testing Decisions

- Exact remainders belong in `until_next` unit tests, with an explicit `predicted_dt`.
  `until_next(Some(900ms), None, 0)` answers `Some(100ms)`. `until_next(Some(1s), None, 0)`
  answers `Some(1s)`. `until_next(None, None, …)` answers `None`. A non-zero `predicted_dt` is
  added only to the Run Clock term: `until_next(Some(900ms), None, 16ms)` answers `Some(116ms)`,
  and `until_next(None, Some(50ms), 16ms)` still answers `Some(50ms)`. There is no test-only sibling.
- Tests that drive the Console observe `repaint_delay` after egui's subtraction (`RawInput`'s
  default `predicted_dt` is 1/60 s). They assert bounds, or they set `predicted_dt` on the test
  input. They do not expect the raw remainder.
- Failing-first proof of the bug: `reduced_motion = true`, BPM 1, Playing, a quiet pass, delay at
  most one second. Today that pass returns `Duration::MAX` (no timed wake). Cursor Effect must be
  off; its 45–190 ms wakes would already satisfy "at most one second".
- Idle guard: Stopped plus reduced motion requests no timed wake (`Duration::MAX`). The console's
  first passes may request repaints of their own (focus, MIDI auto-select); run a settling pass
  before asserting, and confirm that need when the test is written.
- Prior art: `a_playing_console_repaints_as_soon_as_the_published_tick_advances` (publish → zero
  delay) stays and still carries Tick honesty. `a_playing_console_does_not_schedule_a_tick_period_from_this_frame`
  stays; egui's subtraction means the observed delay is never exactly `Bpm::delay_ms()` anyway, so
  that check proves even less once the delay is a remainder. The Cursor-Effect-off, at-most-one-second
  pass is the real guard.
- Cursor Effect `repaint_after` tests remain prior art for the remaining-delay input; this work
  does not re-pin Cursor Effect cadence.
- Tests do not sleep a musical second or a Tick period to prove honesty. Wall-clock jitter loops
  are not the proof. A live `mise run inspect` at 1 BPM is optional confirmation by eye and adds
  no proof beyond the tests above.

## Out of Scope

- When `**` / `//` light relative to the Tick that just played.
- Paint-group comments, bloom wording, or restoring a deleted Cursor Effect fill test.
- Unused cursor delay on options.
- The wall-clock Tick-gap Playback test.
- Showing Scan on the closed Panel.
- Diagnostics frame-rate wakes, extra Readout grains, or a fold of honesty rules.
- Changing how Run Clock is published or frozen.
- Folding the publish-watch loop into `until_next`.
- A recording wake adapter.
- Compensating the Cursor Effect delay for predicted frame time.

## Further Notes

A Render Frame is independent of a Tick. Cursor Effect presentation time is independent of Playback.
`until_next` is where those two facts meet a third: Run Clock glyphs change on wall-clock seconds of
the run, which are neither Ticks nor Cursor Effect frames, and they change from the Duration this
Render Frame already painted. egui 0.36.1's predicted-frame subtraction is why that remainder is
not handed to `request_repaint_after` raw.

This effort finishes ticket 01's unanswered wake. It does not reopen Panel, Readout, Tick, or Run Clock.
