# 14 — Drive the panel layout test through Console::ui

**What to build:** The test that pins the top panel's height against the default window (`the_top_panel_takes_the_height_the_default_window_holds_back`) drives the real console frame instead of rebuilding the panel layout by hand, so it fails when production layout drifts. It has already drifted without failing: its hand-built top bar has only File and View menus, while production also has Help, the right-aligned mode control and Notices.

**Blocked by:** None — can start immediately.

Related: 13 may move the frame code, but the test can already drive the real `Console::ui`.

**Status:** resolved

- [x] The test renders the real console and asserts on the laid-out panels.
- [x] A deliberate production mutation that changes the asserted panel height or available Source area makes the test fail. A content-only change that preserves those dimensions need not fail.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid, with the drift above as evidence.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Removed the unnecessary decomposition blocker and specified a mutation relevant to the measured layout contract.

**2026-09-25 — implementation (epic PR 14).** Delivered in orcvs/orcvs#157. The test runs a whole `Console::ui` pass and asserts the top and bottom `PanelState` rectangles and the Source widget's rectangle (the one `Sense::CLICK | Sense::DRAG` widget). It failed under each production mutation tried: an inner margin on `source_panel_frame`, a taller top-bar `min_size`, and a 40-point top margin on `bottom_panel_frame`. A 20-point bottom margin fits within the Panel's minimum and leaves the dimensions unchanged, so the test does not fail on that content-only change.
