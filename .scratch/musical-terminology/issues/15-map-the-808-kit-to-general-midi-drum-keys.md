# 15 — Map the 808 kit to General MIDI drum keys

**What to build:** A General MIDI drum pattern plays the right instruments on the Built-in Synth's
kit: each instrument answers its GM keys, and other keys are silent.

**Blocked by:** 13 (the kit's channels move there).

**Status:** ready-for-agent

- [ ] Keys: kick 35/36, rim 37, snare 38/40, clap 39, closed hat 42/44, open hat 46, low tom 41/43,
      mid tom 45/47, high tom 48/50, cymbal 49/51/57, cowbell 56, conga 62–64.
- [ ] Every other key plays nothing; the octave-free pitch-class mapping is gone.
- [ ] The kit's doc comments state the key map in the present tense.
- [ ] The ADR that describes the kit is amended.
- [ ] Examples and lessons that play the kit use GM keys.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package synth --all-targets --locked -- -D
      warnings`, `cargo nextest run --package synth --locked` and `cargo nextest run --package
      orcvs --locked` pass, and `mise run site_build` and `mise run site_test` pass. A change to the
      web synth also runs `mise run check_wasm`.

## Comments

Decided in `decisions.md`.
