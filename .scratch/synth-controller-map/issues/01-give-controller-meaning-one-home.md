# 01 — Give "what a controller does on a channel" one home

**What to build:** One module in `synth` that resolves a Control Change on a channel to what it
means for that channel's Part, with each controller's load value beside it. `Synth::apply` asks it
once, and the holders of the state take the meaning and never see a controller number.

**Status:** claimed

**Why:** Routing is four match sites in four files chained by fall-through: `Synth::apply`
(`lib.rs`), `Controls::take` (`acid.rs`), `Channel::set` (`channel.rs`) and `Sends::on`
(`sends.rs`). Load values sit in `Channel::new`, `Controls::default` and `Tilt::set`. The
musical-terminology stack rewrote this table twice. Routing is proven today by rendering audio
(`controllers_74_and_75_on_the_acid_channel…`).

- [ ] A Control Change resolves once, per Part, to a typed meaning. On the acid channel 74 and 75
      resolve to its cutoff and decay and never reach the channel's tilt or length.
- [ ] All Sound Off (120), Reset All Controllers (121) and All Notes Off (123) are in the map; no
      controller number appears as a bare literal in shipped code.
- [ ] Each controller's load value is stated in the map as a Control Change value, and the holders
      start from it. Starting from it sets a `Smoothing` directly and does not ramp, so a fresh
      channel is at rest.
- [ ] `synth/benches/synth.rs` uses the crate's controller names instead of re-declaring them.
- [ ] Routing and precedence are tested against the map. One rendered test per Part still proves
      the wiring from `apply` to sound.

**Out of scope:** where the acid voice's `Controls` live. They stay on `Synth` and stay threaded
through `Sound`; moving them belongs with giving the voice pool its own module.

## Comments

Decided with Toby on 2026-11-17. ADR 0088 and ADR 0099 fix the table's contents, not where it
lives, so no ADR changes.

PR #413 (`claude/synth-controller-map`) implements this ticket. Its status becomes `resolved` when
that PR merges.
