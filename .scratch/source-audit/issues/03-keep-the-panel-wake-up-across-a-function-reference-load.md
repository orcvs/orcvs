# 03 — Prove the panel's Playback wake-up survives an Open

**What to build:** A test that the Tick readout and beat marker still repaint when Playback publishes after the console installs a new Orcvs. The defect itself is fixed: every install path — Help → Function Reference, File → New and File → Open — now goes through `Console::open`, which builds the Orcvs, wires the Playback wake-up and builds the MIDI selection through one `environment()` constructor. No test observes the repaint, so a regression would go unnoticed.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [x] Every path that installs a new Orcvs in the console wires the Playback wake-up to it, through one constructor rather than per call site.
- [ ] A test opens a Source (the Function reference, or any Open), publishes a Playback observation, and observes a repaint request. Kittest tests do not run Playback, so it sits with the existing `app_pass_repaint_delay` tests.
- [ ] After the old Playback observation sender closes, its wake-up task eventually exits. A deterministic test observes that completion; dropping or replacing Orcvs need not terminate the task synchronously.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The bug was fixed by 3d77975b ("Open a Source into the running console through one operation"). The existing tests for an Open assert on settings and the Source, never on a repaint, so the ticket narrows to the tests. It no longer conflicts with 13.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Save As calls `save_picked` → `save_to` and changes file bookkeeping without installing Orcvs. Observer cleanup is asynchronous after sender closure.
