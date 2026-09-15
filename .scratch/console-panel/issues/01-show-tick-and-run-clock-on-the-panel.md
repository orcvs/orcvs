# 01 — Show Tick and Run Clock on the Panel

**What to build:** A static Panel at the bottom of the console that shows Tick and Run Clock. Play: Tick counts as an integer and Run Clock runs from `00:00`. Stop: both freeze. Play again: both reset with the new run. The Panel cannot invent those numbers — `Orcvs` has to publish Tick and the run's epoch so a Render Frame can read them.

Tempo and MIDI menus stay for this ticket. BPM and destination Readouts are later tickets.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A static Panel sits at the bottom of the console. It does not drag, snap, or change orientation.
- [x] The Panel shows Tick as a whole number (the engine's integer: executed count and next Tick). No fraction, no pad, no `f` suffix.
- [x] The Panel shows Run Clock as wall-clock of the current Playback run, `mm:ss`, then `h:mm:ss` after 59:59.
- [x] Before the first run, Tick is `0` and Run Clock is `00:00`.
- [x] While Playback is stopped, Tick and Run Clock hold the values they had when the run stopped.
- [x] Beginning a new run resets Tick to `0` and Run Clock to `00:00`.
- [x] An Overrun does not move Tick, does not invent skipped time on Run Clock, and does not appear on the Panel.
- [x] File and View remain on the top bar. The Tempo and MIDI menus are still present.
- [x] `CONTEXT.md` already names Panel, Readout, Tick, and Run Clock; this ticket does not reopen those terms.

## Answer

The engine publishes one `PlaybackObservation` (`state`, `tick`, `run_started_at`, `frozen_run_clock`) on a `watch`, the MidiDestinations pattern. `PlaybackEngine::state()` reads `observation.borrow().state`. `PlaybackEngine::observation()` and `Orcvs::playback_observation()` return the latest value without awaiting.

`begin_run` captures `web_time::Instant::now()` as the run origin (not `TickClock.epoch`). `execute_tick` publishes the advanced Tick after a successful Tick; an Overrun returns without publishing. `stop` freezes `run_clock()` into `frozen_run_clock` and clears the origin. While Playing, `PlaybackObservation::run_clock()` is `origin.elapsed()`; while Stopped it is the frozen Duration.

The console draws a non-resizable `egui::Panel::bottom("bottom_panel")` before `CentralPanel`, height `BOTTOM_PANEL_HEIGHT` (32), Readouts `Tick` (integer) and `Run Clock` (`format_run_clock`). `DEFAULT_VIEW_SIZE` includes that height so the default window still presents the default Grid at scale one. While Playing it also `request_repaint_after(1s)` so `mm:ss` moves on Render Frames. Tempo and MIDI menus stay.

Files: `orcvs/src/playback.rs`, `orcvs/src/app.rs`, `console/src/console.rs`.
