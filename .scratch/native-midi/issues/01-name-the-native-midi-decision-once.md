# 01 — Name the native MIDI decision once

**What to build:** One place in `orcvs` that decides whether a native MIDI backend exists on the target and what a running Orcvs therefore uses for output, so that nothing else — no other `orcvs` module, and no consuming crate — restates that condition. The console still discovers and selects MIDI destinations exactly as it does today; it just asks `orcvs` what backend it has instead of deriving it from the target operating system.

This is a prefactor with no behaviour change. It exists because a consuming crate cannot test an `orcvs` feature: `feature = "native-midi"` inside the shell names the shell's own feature. Without a seam the shell has to mirror the flag and keep the mirror in sync forever.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Whether a native MIDI backend is available is decided in exactly one place in `orcvs`; no other module or crate spells out the equivalent target condition for MIDI.
- [ ] `orcvs` exposes a default output adapter and a MIDI backend type that name a valid type on every supported target, including WASM.
- [ ] The console obtains its running Orcvs and its MIDI device selection through that seam and carries no target condition of its own for MIDI.
- [ ] Native MIDI destination discovery, selection, all-notes-off safety, and delivery-failure reporting behave exactly as before on macOS, Windows, and Linux.
- [ ] On a target with no native MIDI backend the console still starts and plays silently, as it does today.
- [ ] No manifest, dependency, or feature change.
- [ ] `mise run check` passes.
