# Source Playback Engine

## Notes

Implementation issues are tracked under `issues/`.

## Decisions-so-far

- Native MIDI delivery is isolated behind Playback Engine destination configuration and a target-gated `midir` backend; raw delivery behavior is hardware-independently regression tested. [Issue 06](issues/06-deliver-play-commands-to-native-midi-output.md)

## Fog
