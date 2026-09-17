# 01 — Wake Run Clock on a quiet Playing pass

**What to build:** At 1 BPM with Cursor Effect off, a quiet Playing Render Frame requests a
delay of at most one second, so the Run Clock Readout can move between Ticks. Today that
pass requests no timed wake.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] One Run Clock sample per Render Frame feeds both the Readout text and `until_next`. The term is `Some` only while Playing with a run origin.
- [x] `until_next` is a shipped free function. Shape from the spec:

  ```rust
  fn until_next(
      run_clock: Option<Duration>,
      cursor_effect: Option<Duration>,
      predicted_dt: Duration,
  ) -> Option<Duration>
  ```

  It takes no toolkit types and never reads Playback observation or BPM.
- [x] Run Clock remainder is time until the next whole second of that sample, plus predicted frame time. The Cursor Effect term is not offset. An exact whole second waits one second plus predicted frame time.
- [x] CentralPanel returns Cursor Effect remaining delay (visibility check stays there) and does not request a Render Frame. `until_next` runs after CentralPanel. One delayed-repaint request.
- [x] Publish still requests an immediate Render Frame. The published-Tick zero-delay test still holds.
- [x] Unit tests: 900 ms with no Cursor Effect and zero predicted frame time answers 100 ms; exact 1 s answers 1 s; a non-zero predicted frame time adds only to the Run Clock term.
- [x] Console pass with reduced motion, BPM 1, Playing: delay at most 1 s (today no timed wake).

## Comments

Do not fold Cursor Effect cadence into `until_next`. Do not add a wake port. Do not change
how Run Clock is published. Do not strip the Run Clock predicted-frame offset later: the
toolkit subtracts it from every delayed repaint, and without it the console bursts until
the second changes. Combining the two remainders is call-site ownership; it is not what
makes the wake honest. Spec: [../spec.md](../spec.md).

## Answer

`until_next` is a `pub(crate)` free function in `console/src/readout_deadline.rs`. Each
Render Frame samples Run Clock once; that Duration paints the Readout, and `Some` of it
reaches `until_next` only while Playing with a run origin. CentralPanel returns Cursor
Effect remaining delay. After CentralPanel, one `request_repaint_after` from `until_next`.
Publish still wakes immediately. Quiet Playing at 1 BPM with Cursor Effect off requests
a delay of at most one second after egui subtraction.

Files: `console/src/readout_deadline.rs`, `console/src/console.rs`, `console/src/lib.rs`.
