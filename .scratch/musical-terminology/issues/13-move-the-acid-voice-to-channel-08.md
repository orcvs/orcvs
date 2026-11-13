# 13 — Move the acid voice to channel 08

**What to build:** A General MIDI drum pattern sent to MIDI channel 10 plays drums on the Built-in
Synth: the acid voice moves to `08` and the 808-style kit takes `09`–`0F`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The acid voice plays on channel `08`; the kit plays on `09`–`0F`; tones stay on `00`–`07`.
- [ ] The Sound Module shows the new channel assignment.
- [ ] ADR 0099 is amended, and the synth's doc comments state the new channels.
- [ ] Examples and lessons that play the acid voice or the kit use the new channels.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package synth --all-targets --locked -- -D
      warnings`, `cargo nextest run --package synth --locked` and `cargo nextest run --package
      orcvs --locked` pass, and `mise run site_build` and `mise run site_test` pass. A change to the
      web synth also runs `mise run check_wasm`.

## Comments

Decided in `decisions.md`.
