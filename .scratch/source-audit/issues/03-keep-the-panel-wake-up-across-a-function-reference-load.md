# 03 — Prove the panel's Playback wake-up survives an Open

**What to build:** A test that the Tick readout and beat marker still repaint when Playback publishes after the console installs a new Orcvs. The defect itself is fixed: every install path — Help → Function Reference, File → New and File → Open — now goes through `Console::open`, which builds the Orcvs, wires the Playback wake-up and builds the MIDI selection through one `environment()` constructor. No test observes the repaint, so a regression would go unnoticed.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Every path that installs a new Orcvs in the console wires the Playback wake-up to it, through one constructor rather than per call site.
- [x] A test opens a Source (the Function reference, or any Open), publishes a Playback observation, and observes a repaint request. Kittest tests do not run Playback, so it sits with the existing `app_pass_repaint_delay` tests.
- [x] After the old Playback observation sender closes, its wake-up task eventually exits. A deterministic test observes that completion; dropping or replacing Orcvs need not terminate the task synchronously.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The bug was fixed by 3d77975b ("Open a Source into the running console through one operation"). The existing tests for an Open assert on settings and the Source, never on a repaint, so the ticket narrows to the tests. It no longer conflicts with 13.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Save As calls `save_picked` → `save_to` and changes file bookkeeping without installing Orcvs. Observer cleanup is asynchronous after sender closure.

**2026-09-25 — implementation (epic PR 14).** Delivered in orcvs/orcvs#157. `an_opened_console_repaints_as_soon_as_its_new_playback_publishes` opens the Function Reference, plays, and asserts a zero repaint delay once the new Orcvs publishes; it fails when the wake-up is wired in `Console::new` only. `the_wake_up_over_a_replaced_orcvs_ends_once_its_playback_is_gone` spawns and awaits `panel_wake` (split out of `wake_panel_when_playback_publishes`) over the replaced Orcvs; it fails when the loop ignores a closed sender. The settling passes after an Open or a Space press requested a zero delay by themselves, so `settle_repaint` now runs before the Tick is awaited, in the existing Tick repaint test as well.
