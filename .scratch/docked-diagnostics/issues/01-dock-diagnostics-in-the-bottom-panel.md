# 01: Dock Diagnostics beneath the bottom Panel's playback controls

**What to build:** View → Diagnostics expands the fixed bottom Panel to display rendering diagnostics beneath the existing playback controls, separated by a horizontal rule. Diagnostics occupy their own space rather than covering the Source View. Turning Diagnostics off restores the compact Panel.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Diagnostics are enabled in a fresh console and remain toggleable through View → Diagnostics.
- [x] Enabling Diagnostics makes the bottom Panel taller, preserves its playback controls above a horizontal separator, and displays label/value rows beneath it.
- [x] All six readouts remain available: FPS, Frame time, CPU time, Cell size, Visible Source region, and Pixels per point.
- [x] The diagnostic table uses the Panel's full width, with a generous label column and the remaining space for values. Labels display their full text; long values in narrow windows can truncate with the full value available on hover.
- [x] Diagnostics are fixed within the Panel, with no floating window, drag interaction, or resize handle.
- [x] The expanded Panel consumes layout space and reduces the Source View's height without overlaying it.
- [x] Visible Source region reflects the current frame's Source View geometry, including after toggling Diagnostics, resizing, panning, or changing whole-UI zoom.
- [x] Turning Diagnostics off hides the readouts and restores the compact Panel height.
- [x] The embedded egui inspection section is removed from the everyday Diagnostics presentation; development inspection tooling remains available.
- [x] Focused regression coverage drives the shipped console to verify toggling, Panel expansion and restoration, readout placement below the playback controls, and separation from the Source View.
- [x] Existing playback controls, keyboard ownership, and appearance behaviour remain correct with Diagnostics enabled and disabled.

## Comments

The agreed layout is a playback-control row, a horizontal separator, and six diagnostic label/value rows. This is a reversible presentation change and does not warrant an ADR. Diagnostics here means rendering readouts, not language or Playback failure messages.


Implementation uses the Panel's full width: the label column has a minimum of one sixth of the available width, labels retain their full text, and values use the remaining space. The Panel reserves the Diagnostics area before Source layout and fills it from the current frame's geometry afterward.

Validation:

- Updated the shipped-console toggle regression to verify Panel expansion and restoration, Source space, readout placement, the separator, and removal of the embedded inspector.
- Added current-frame Source region coverage for toggling, resizing, panning, and zooming.
- Added painted-text coverage for complete labels at default and narrow widths, generous label-column spacing, and an untruncated Source region value at the default width.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package console --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo test --package console --locked` — passed; 525 native tests, plus the console doctest target.
- `cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings` — passed.
- `node --test scripts/tests/roadmap.test.ts` — passed.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.

Not run:

- `cargo nextest run --package console --locked` — cargo-nextest is unavailable; the complete console suite ran through cargo test instead.
- `mise run check_wasm` — mise and trunk are unavailable; its WASM clippy command passed, but the two trunk application builds did not run.
- Live egui inspection — mise, egui-mcp, and a GUI session are unavailable; behaviour and geometry were verified through the shipped-console harness.
- Headless browser suite, macOS checks, and full-count proptest — deferred to CI.

Risks: presentation and layout only. No public API, unsafe, dependency, feature, concurrency, or performance-claim changes. No ADR is needed.

Follow-up: Diagnostics open by default. The label column is half its initial width (one sixth of the Panel width). Release captures read the Source area from actual UI geometry so their coverage excludes the expanded Panel.

Follow-up validation: `PROPTEST_CASES=32 cargo test --package console --locked` passed (525 tests). `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, and `cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings` passed. `cargo clippy --package console --lib --tests --features release-capture --locked -- -D warnings` passed, compiling the updated capture helper. Both roadmap checks passed. The previously recorded unavailable checks and CI deferrals still apply.
