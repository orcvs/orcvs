# 08 — Play sound through the built-in synth

**What to build:** The Built-in Synth as ADR 0087 decides it, on the web. A Rust synth core crate
with no audio I/O, compiled to its own WASM module that runs in an AudioWorklet, plays Pilot-style
tones on channels `00`–`07` and the 808-style kit on `08`–`0F`, sixteen voices, at each message's
planned time plus a fixed delay. A page that frames the console announces it with
`orcvs.synth`; the console then lists "Built-in Synth" after the MIDI ports, a Linked or small
console picks it, and its connection posts `orcvs.midi` with each message's planned Tick time. The
page's sound module passes those to the worklet and resumes the audio context on the first message
of a run. The core's WASM and worklet script ship in `console-web` beside the console. See
`../spec.md` (Built-in synth). The timing prototype on `claude/project-thread-wd8e80`
(`console/src/builtin_synth.rs`) holds a working 808 kit and the timing measurements.

**Blocked by:** 02, 07

**Status:** ready-for-human

- [x] Tests through the `MidiBackend` seam: the destination is listed after the MIDI ports only
      while a page announces it; a Linked console picks it; each message is posted with its Tick's
      planned time; an `available: false` announcement answers its reason on connect.
- [x] Tests on the synth core: each channel and pitch class becomes the voice ADR 0087 names, at the
      frame its time names; a seventeenth Note On takes the oldest voice; All Notes Off releases
      tones and leaves drums ringing; a closed hat chokes an open hat. No shipped item takes a
      parameter only a test supplies.
- [x] A browser test shows the audio context is running after the Playback gesture, and that a
      steady 16th pattern sounds at a constant offset from its Ticks within the 50 ms target.
- [ ] Any `web-sys` feature or dependency added is recorded with its rationale, and
      `mise run audit_deps` passes.
- [x] `mise run check_wasm` passes, and the site's build still compiles no Rust.

## Comments

- The core is the `synth` crate; `synth/src/web.rs` exports it to the AudioWorklet as three C-ABI
  functions, so the module imports nothing and needs no generated glue. `synth/web/` holds the
  worklet and the page's host script; `scripts/stage-synth.sh`, Trunk's post-build hook, stages
  them and `synth.wasm` into `console-web`.
- Playback hands each destination how late its Tick is submitted (`OutputAdapter::submit`'s
  `since_due`, `MidiConnection::send_due`), and the synth's connection turns that back into the planned instant
  on the page's clock. The host plays each message 50 ms after it.
- The site loads the host from `site/src/console/launch.ts`, beside the consoles it launches.
  `site/tests/synth.spec.ts` frames a steady sixteenth pattern and finds the Ticks on a 125 ms grid,
  each sounding 50 ms after its Tick and none reaching the worklet late.
- While Web MIDI access is still pending, discovery answers that it is waiting, so automatic
  selection never picks the synth over ports it has not listed yet; a Linked console still picks
  the synth, since connecting to it needs no listing.
- `check_wasm`'s steps were run by hand: workspace clippy for `wasm32-unknown-unknown`, then
  `trunk build --locked` with and without default features, which staged `synth.wasm` and both
  scripts beside the console.
- `web-sys` gains `MessageEvent` and `Performance`, recorded in `console/Cargo.toml`. The `synth`
  crate has no dependencies. `cargo deny` was not available where this was built, so
  `mise run audit_deps` has not run yet; its box stays open until CI or a local run passes it.
