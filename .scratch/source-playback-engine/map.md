# Source Playback Engine

## Notes

Implementation issues are tracked under `issues/`.

## Decisions-so-far

- Expressions remain horizontal within one Grid row and retain current diagnostics throughout Live Editing. [Issue 02](issues/02-keep-expressions-horizontal-and-diagnosable.md)
- A Tick Plan deterministically commits the complete result of one pre-Tick Source snapshot. [Issue 03](issues/03-commit-atomic-cell-results-through-tick-plans.md)
- Root Play Functions emit ordered Play Commands without Cell writes or inferred timing. [Issue 04](issues/04-interpret-terminal-play-functions-into-play-commands.md)
- The Playback Engine owns lifecycle, musical time, Tick orchestration, and exact output dispatch. [Issue 05](issues/05-run-live-editing-through-the-playback-engine.md)
- Native MIDI delivery is isolated behind Playback Engine destination configuration and a target-gated `midir` backend; raw delivery behavior is hardware-independently regression tested. [Issue 06](issues/06-deliver-play-commands-to-native-midi-output.md)

## Fog
