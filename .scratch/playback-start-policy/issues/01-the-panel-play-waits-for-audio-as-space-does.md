# 01 — The Panel's Play waits for audio as Space does

Status: ready-for-agent

**Defect:** On a page whose audio is asleep, the Panel's `P` starts Playback on the Built-in Synth at once, so Tick 0 sounds late and the Ticks after it on time. ADR 0087 says a console holds a start on the Built-in Synth until the audio runs, for at most 500 ms, and ADR 0089 says `P` does what Space does. Only Space crosses the hold: `Console::route_keys` passes the frame's events through `hold_start_for_audio` (`console/src/console/input.rs:284`), which drops `KeyPressed(Space)` from the vector (`console/src/built_in_synth.rs:162`), while `P` calls `Orcvs::toggle_playback` directly (`console/src/console/panel.rs:441`). A tap on `P` resumes the page's audio (ADR 0089), but resuming takes a device over 100 ms (ADR 0087).

**What to build:** One console Playback module owns the start policy, and both controls make the same request of it.

- Space and `P` are both Playback requests. The console takes Space out of the frame's events before `Orcvs::event_handler`, calls `Orcvs::disarm_fill` when it does (every event disarms a command-Enter fill, `orcvs/src/app.rs:541`), and makes a request. `P` makes the same request through one `Console::request_playback(ctx)`.
- A granted toggle runs after `event_handler` has handled the frame's other events.
- `console/src/console/playback.rs` holds a plain state machine, `request(now, early)` and `poll(now, awake)`, that owns the held start, release, cancel, `AUDIO_WAIT` and the 20 ms repaint. It builds on native and the web.
- One `Console` method answers `early` (Playback stopped, the page's audio asleep, the Built-in Synth selected) and `awake`. It is `cfg`-gated to never early on native, which has no `SynthPage`.
- One per-frame poll releases a held start, so a `P` tap held during the Panel pass releases on the next frame as a held Space does.
- `SynthAnnouncements` keeps `observe` and loses `hold_start` and `held_since`.

The rules are the same whichever control asks:

- A start is held only while Playback is stopped, the page's audio is asleep and the Built-in Synth is selected.
- A second request while one is held cancels both, whether it comes from Space or `P`.
- A request while Playback plays stops it at once.
- A held start is released when the audio wakes or once `AUDIO_WAIT` has passed.

No new ADR: this implements ADR 0087 and 0089. Preserve ADR 0041's ownership of Playback state by the Playback Engine.

## Acceptance criteria

- [ ] Native unit tests on the state machine cover every case `hold_start`'s tests cover today (hold, release on wake, release at `AUDIO_WAIT`, cancel on a second request, no hold while playing, no hold when another destination is selected), plus a cancel across the two controls.
- [ ] A Playwright case in `site/tests/synth.spec.ts` taps `P` on a page whose audio is asleep and shows Tick 0 sounds on time; it fails before the change.
- [ ] Space still disarms a command-Enter fill.
- [ ] Space and `P` on native behave as before.
- [ ] `hold_start_for_audio` and `SynthAnnouncements::hold_start` are gone; no test-only input reaches shipped code.

## Comments

### 2026-11-03 — Opened

From the architecture review of HEAD `6b124537` (candidate 01), settled in a grilling session. Rejected: having `P` inject a synthetic `KeyPressed(Space)`. It makes a pointer control act as a key, and `route_keys`' `keyboard_elsewhere` early return would drop the tap while a control holds focus. Native kittest has no synth, so the Playwright case is the evidence for the defect; it runs in the merge tier's browser suite. Land this before `example-discard/01`.
