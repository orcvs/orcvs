# 14 — Move Built-in Synth effects to General MIDI controllers

**What to build:** A musician's GM controller knowledge works on the Built-in Synth: reverb is
CC91, chorus CC93, and the send effects are set per channel, as on a GM module.

**Blocked by:** 13 (both change the synth's channel and controller handling).

**Status:** ready-for-agent

- [ ] Controllers: `MD` vibrato CC1, `OF` offset CC73 (Attack Time), `LN` length CC72 (Release
      Time), `EQ` tilt CC74 (Brightness), `R1` small reverb CC91, `R2` large reverb CC92, `CH`
      chorus CC93, `DL` echo CC94.
- [ ] Reverb, chorus and echo are per-channel sends: a controller on one channel sets that
      channel's send. `MD`, `OF`, `LN` and `EQ` stay global.
- [ ] On the acid voice's channel, its own sound controllers keep their meanings; CC74 is the
      cutoff there.
- [ ] CC2–CC8 no longer set effects.
- [ ] The Sound Module shows the effects and their new controllers.
- [ ] The synth's internal parameter smoother is renamed `Smoothing`, so the code no longer
      calls it Glide.
- [ ] ADR 0088 is amended with the new map and the per-channel sends.
- [ ] Examples and lessons that send effect controllers use the new numbers.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package synth --all-targets --locked -- -D
      warnings`, `cargo nextest run --package synth --locked` and `cargo nextest run --package
      orcvs --locked` pass, and `mise run site_build` and `mise run site_test` pass. A change to the
      web synth also runs `mise run check_wasm`.

## Comments

Decided in `decisions.md` (rounds one and two: GM controllers, per-channel sends, the map, and
the `Smoothing` rename).
