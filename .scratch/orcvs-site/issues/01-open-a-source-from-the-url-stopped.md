# 01 — Open a Source from the URL, stopped

**What to build:** The web console reads its URL once at startup. `?example=<name>` opens the
shipped example of that name from `console/src/examples.rs`. `#source=<percent-encoded text>` opens
that text, read as a Source File through `orcvs::source::file::read`, with a byte limit checked
before decoding. Text that does not decode or parse is rejected whole, as is an unknown example
name. Either way the console opens the empty Source and shows why. A Source opened this way opens
with Playback stopped, so only the reader's action starts it. See `../spec.md` (Loading a Source,
Nothing plays by itself) and `docs/research/orcvs-site.md` section 5.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The byte limit is a named constant, and the ticket's commit message says why that value.
- [ ] Boundary tests cover: a known and an unknown example name, a valid Source, malformed
      percent-encoding, text over the limit, text that is not a valid Source File, and both
      parameters at once (the ticket picks and tests which one wins).
- [ ] A property test shows that any text the reader accepts round-trips through
      `orcvs::source::file::write`. `PROPTEST_CASES=32` locally.
- [ ] A test shows a Source opened from the URL is stopped at startup, even with a MIDI destination
      selected.
- [ ] Native builds read no URL, and no shipped native item gains a parameter for this.
- [ ] `cargo clippy --package console --all-targets --locked -- -D warnings`,
      `cargo nextest run --package console --locked`, and `mise run check_wasm` pass.
