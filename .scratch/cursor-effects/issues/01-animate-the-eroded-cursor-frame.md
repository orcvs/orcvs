# 01 — Animate the eroded Cursor frame

**What to build:** Replace the selected Cell's plain Cursor stroke with the eroded off-white frame developed in the cursor prototypes. The frame keeps changing while stationary: bright patches, gaps, short horizontal tears, and fine connecting strands form and disappear independently along all four edges. The selected Position remains exact and its Glyph remains readable.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The frame follows the selected Cell at every supported zoom and device scale, including empty and occupied Cells. Neither noise nor displacement moves the selected Position, hit target, or Glyph. Evidence: `cursor_effect_shapes` anchors on the passed `cursor` rect; `console::tests::a_zoomed_row_leaves_cursor_effects_out_of_cell_fills` keeps effects beneath the Grid at max zoom without shifting selection geometry.
- [x] Noise evolves while the Cursor is stationary, while it moves, and while Playback is stopped. It is not merely a movement-triggered burst or a change of opacity around a fixed outline. Evidence: `CursorEffectAnimation::advance` advances on elapsed presentation time, not cursor movement; `animation_changes_only_at_deadlines_and_does_not_replay_a_backlog`.
- [x] All four edges have independently varying concentrations, gaps, and fragments. No permanent top-left/bottom-right emphasis, fixed bright corners, or regularly arranged radial strands remain. Evidence: `frame_fragments_cover_all_four_edges_and_evolve` asserts coverage on all four sides and shape change between samples.
- [x] The default frame is off-white, matching the latest “Changing edges + field” prototype. Variation uses irregular intervals around the prototype's roughly 100–220 ms cadence, not a synchronized whole-frame flash. Confirm the final defaults by viewing the native render at normal size. Evidence: `defaults_are_the_approved_colours_and_an_irregular_live_cadence` (RGB 234, 235, 229; `interval(0) != interval(255)`); native visual check on branch at normal size.
- [x] Randomness is console presentation state, independent of Source revisions, evaluation, MIDI, and Playback timing. Ordinary extra repaints do not advance the noise faster; a fixed presentation state produces repeatable geometry for verification. Evidence: animation state lives in `ConsoleApp`; fixed `CursorEffectSample` inputs produce repeatable shapes in geometry tests.
- [x] Repaints are scheduled for the next visual change. Hidden or fully clipped effects do not require continuous animation work; resuming does not replay a backlog of missed noise updates. Evidence: `repaint_after`; `animation_changes_only_at_deadlines_and_does_not_replay_a_backlog`; `a_fully_clipped_effect_builds_no_shapes`.
- [x] The frame is clipped correctly and remains visible on native and WASM. Reduced-motion behaviour retains a clear static Cursor without changing selection behaviour. Evidence: `a_fully_clipped_effect_builds_no_shapes`; `reduced_motion_uses_a_clear_static_cursor`; WASM seed fix in `animation_seed` (no `SystemTime::now()` panic on `wasm32-unknown-unknown`).
- [x] Focused tests cover all-edge evolution, bounded geometry, unchanged Cell selection, clipping, and animation timing. Test inputs are built below shipped entry points, without test-only parameters or branches in production code. Evidence: `console/src/cursor_effects.rs` tests and `console::tests::a_zoomed_row_leaves_cursor_effects_out_of_cell_fills`.
- [x] Compare several stationary realizations and movement over occupied Cells against the latest prototype at normal size and enlarged detail. Preserve that prototype as design evidence before cleanup. Its visual decisions are the reference; rewrite its throwaway implementation for egui. Evidence: `console/prototypes/cursor-comparison/cursor-evolving-frame.html` retained; native visual comparison on branch.
- [x] Run the applicable repository-scoped checks and record visual verification and deferred CI checks. Extend the console painting benchmark to cover the animated Cursor path; the comparison run belongs to CI. Evidence: `console/benches/paint.rs` `cursor_effects` bench; crate-scoped fmt/clippy/nextest on branch; merge-tier browser and benchmark comparison deferred to CI.

## Design decisions

This ticket supersedes the earlier static Cursor/no-animation direction only for Cursor effects. Update the corresponding appearance documentation when implementing; do not relax unrelated widget or panel styling rules. Keep presentation ownership in the console and respect the existing separation between Paint values and geometry.

The existing area bloom can remain during this slice. Ticket 02 replaces it, so this ticket is independently demoable without coupling the frame implementation to a new area renderer.

## Answer

The console now owns a randomized, deadline-driven eroded frame whose four edges evolve independently. It follows the exact selected Cell, clips with the viewport, avoids missed-update backlogs, and becomes a clean static frame at zero glitch amount.
