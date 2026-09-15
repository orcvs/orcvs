# 01 — Animate the eroded Cursor frame

**What to build:** Replace the selected Cell's plain Cursor stroke with the eroded off-white frame developed in the cursor prototypes. The frame keeps changing while stationary: bright patches, gaps, short horizontal tears, and fine connecting strands form and disappear independently along all four edges. The selected Position remains exact and its Glyph remains readable.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [ ] The frame follows the selected Cell at every supported zoom and device scale, including empty and occupied Cells. Neither noise nor displacement moves the selected Position, hit target, or Glyph.
- [ ] Noise evolves while the Cursor is stationary, while it moves, and while Playback is stopped. It is not merely a movement-triggered burst or a change of opacity around a fixed outline.
- [ ] All four edges have independently varying concentrations, gaps, and fragments. No permanent top-left/bottom-right emphasis, fixed bright corners, or regularly arranged radial strands remain.
- [ ] The default frame is off-white, matching the latest “Changing edges + field” prototype. Variation uses irregular intervals around the prototype's roughly 100–220 ms cadence, not a synchronized whole-frame flash. Confirm the final defaults by viewing the native render at normal size.
- [ ] Randomness is console presentation state, independent of Source revisions, evaluation, MIDI, and Playback timing. Ordinary extra repaints do not advance the noise faster; a fixed presentation state produces repeatable geometry for verification.
- [ ] Repaints are scheduled for the next visual change. Hidden or fully clipped effects do not require continuous animation work; resuming does not replay a backlog of missed noise updates.
- [ ] The frame is clipped correctly and remains visible on native and WASM. Reduced-motion behaviour retains a clear static Cursor without changing selection behaviour.
- [ ] Focused tests cover all-edge evolution, bounded geometry, unchanged Cell selection, clipping, and animation timing. Test inputs are built below shipped entry points, without test-only parameters or branches in production code.
- [ ] Compare several stationary realizations and movement over occupied Cells against the latest prototype at normal size and enlarged detail. Preserve that prototype as design evidence before cleanup. Its visual decisions are the reference; rewrite its throwaway implementation for egui.
- [ ] Run the applicable repository-scoped checks and record visual verification and deferred CI checks. Extend the console painting benchmark to cover the animated Cursor path; the comparison run belongs to CI.

## Design decisions

This ticket supersedes the earlier static Cursor/no-animation direction only for Cursor effects. Update the corresponding appearance documentation when implementing; do not relax unrelated widget or panel styling rules. Keep presentation ownership in the console and respect the existing separation between Paint values and geometry.

The existing area bloom can remain during this slice. Ticket 02 replaces it, so this ticket is independently demoable without coupling the frame implementation to a new area renderer.

## Answer

The console now owns a randomized, deadline-driven eroded frame whose four edges evolve independently. It follows the exact selected Cell, clips with the viewport, avoids missed-update backlogs, and becomes a clean static frame at zero glitch amount.
