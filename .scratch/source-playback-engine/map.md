# Source Playback Engine

## Notes

Implementation issues are tracked under `issues/`.

## Decisions-so-far

- Expressions remain horizontal within one Grid row and retain current diagnostics throughout Live Editing. [Issue 02](issues/02-keep-expressions-horizontal-and-diagnosable.md)
- A Tick Plan deterministically commits the complete result of one pre-Tick Source snapshot. [Issue 03](issues/03-commit-atomic-cell-results-through-tick-plans.md)
- Root Play Functions emit ordered Play Commands without Cell writes or inferred timing. [Issue 04](issues/04-interpret-terminal-play-functions-into-play-commands.md)
- The Playback Engine owns lifecycle, musical time, Tick orchestration, and exact output dispatch. [Issue 05](issues/05-run-live-editing-through-the-playback-engine.md)
- Native MIDI delivery is isolated behind Playback Engine destination configuration and a target-gated `midir` backend; raw delivery behavior is hardware-independently regression tested. [Issue 06](issues/06-deliver-play-commands-to-native-midi-output.md)
- Playback Engine owns its lifecycle concurrency behind a cloneable handle and transfers ordered diagnostics through one atomic observation. [Issue 20](issues/20-deepen-playback-lifecycle-module.md)
- Source rebuilds all derived Cell classifications from each accepted revision, including row-confined operand hints and raw occupied Cells. [Issue 11](issues/11-invalidate-operand-slot-hints-on-edit.md)
- Source rejects Expressions beyond parser capacity atomically, while parsing remains panic-free for non-edit ingress. [Issue 12](issues/12-bound-expression-length-instead-of-panicking.md)
- Self-overwriting results are unreachable because Expressions remain row-confined and results target the row below. [Issue 13](issues/13-discard-tick-results-that-overwrite-their-own-expression.md)
- Occupied Cells always publish a Source-derived Glyph before the renderer observes them. [Issue 15](issues/15-render-a-cell-that-has-no-glyph.md)
- Background Marker, Highlight, and Space classifications remain distinct through rendering. [Issue 16](issues/16-construct-the-highlight-and-space-glyphs.md)
- Marker spacing is a validated positive whole-Cell value shared by placement and block geometry. [Issue 17](issues/17-measure-marker-spacing-in-whole-cells.md)
- Source persistence stores only canonical Grid dimensions and Cells, validates them at ingress, and rebuilds derived state. [Issue 18](issues/18-repair-the-persistence-feature.md)
- Positions carry allocation-free Grid identity, and Grid queries refuse positions minted elsewhere. [Issue 19](issues/19-give-a-position-its-grid-identity.md)

## Fog
