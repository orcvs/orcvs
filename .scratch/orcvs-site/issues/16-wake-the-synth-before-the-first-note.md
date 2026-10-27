# 16 — Wake the Built-in Synth before the first note

**What to build:** The first note of a run sounds on time on the web. A page's audio context
starts suspended, and `synth/web/synth-host.js` resumes it on the first `orcvs.midi` message it
receives. Resuming takes a device over 100 ms, measured at about 130 ms in headless Chromium, so
the first note sounds late. The host holds messages while the context starts, and plays the ones
at most 500 ms past their planned instant as soon as it runs.

Toby's direction (2026-10-27): send a zero-volume note before Playback starts, so the context is
running by the time the first Tick's message arrives. Decide with Toby when the console sends it:
when the Built-in Synth is selected, on the reader's first key or click in the console, or when
Playback starts. Also decide whether the first Tick waits for the context. The message has to
reach the host while the reader's gesture still counts as activation, and it must sound nothing
on either the tones or the drums.

**Acceptance:**

- With audio that waits for a gesture, the first Tick's Note On sounds within the 50 ms delay of
  its planned instant. The test is `site/tests/synth.spec.ts`, which already runs Chromium without
  Playwright's autoplay flag.
- The wake message makes no sound and takes no voice.

**Blocked by:** 08

**Status:** needs-triage
