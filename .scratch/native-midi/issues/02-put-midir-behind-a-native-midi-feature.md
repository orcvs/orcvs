# 02 — Put midir behind a native-midi feature

**What to build:** A `native-midi` feature on `orcvs` that gates the `midir` dependency and the native MIDI backend, on by default so every build that exists today resolves identically. Turning it off gives a running Orcvs that composes, executes, and renders Source exactly as before but produces no MIDI output, and whose dependency tree contains no `midir` and no system audio library. The shipped console keeps its MIDI: it enables the feature explicitly for its native targets.

Feature name is `native-midi`, not `midi`: the target-agnostic MIDI vocabulary — destinations, backends, output adapters, Play Commands — stays present either way. What the feature gates is the one backend that talks to the platform.

**Blocked by:** 01 — Name the native MIDI decision once.

**Status:** ready-for-agent

- [ ] `orcvs` declares a `native-midi` feature, on by default, and `midir` is an optional dependency reached only through it.
- [ ] With the feature enabled, dependency resolution, behaviour, and the public surface are unchanged from today.
- [ ] With the feature disabled, `orcvs` builds with neither `midir` nor a system audio library anywhere in its dependency tree, and a running Orcvs uses an output adapter that emits nothing.
- [ ] The console enables the feature for its native targets, so the shipped application still lists and selects MIDI destinations.
- [ ] The WASM target and its browser regressions are unaffected in either feature state.
- [ ] Verification exercises the feature disabled, enabled, and crossed with `persistence`, rather than assuming the two compose.
- [ ] Tooling checks and documentation record the feature, its default, and what disabling it gives up.
- [ ] `mise run check`, `mise run test_persistence`, `mise run check_wasm`, and `mise run audit_deps` pass.

## Comments

### Release freeze

Recorded during the `release/v1` issue alignment on 2026-09-04. This issue carries no
`release/v1` tag and is not release work, but it must not land inside the release window: from the
moment `v1-release/03` cuts the candidate SHA until `v1-release/01` records the GO decision.

The reason is evidence, not behaviour. `v1-release/03` records a `cargo deny --locked check` result
that describes one dependency tree. Making `midir` optional changes that tree, so a merge inside the
window leaves the recorded result describing a build that is no longer the candidate. Land it before
the SHA is cut, or after GO.
