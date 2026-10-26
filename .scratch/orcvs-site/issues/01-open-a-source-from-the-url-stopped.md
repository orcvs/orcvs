# 01 — Open a Source from the URL, stopped

**What to build:** The web console reads its URL once at startup. `?example=<name>` opens the
shipped example of that name from `console/src/examples.rs`. `#source=<percent-encoded text>` opens
that text, read as a Source File through `orcvs::source::file::read`, with a byte limit checked
before decoding. Text that does not decode or parse is rejected whole, as is an unknown example
name. Either way the console opens the empty Source and shows why. A Source opened this way opens
with Playback stopped, so only the reader's action starts it. See `../spec.md` (Loading a Source,
Nothing plays by itself) and `docs/research/orcvs-site.md` section 5.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The byte limit is a named constant, and the ticket's commit message says why that value.
- [x] Boundary tests cover: a known and an unknown example name, a valid Source, malformed
      percent-encoding, text over the limit, text that is not a valid Source File, and both
      parameters at once (the ticket picks and tests which one wins).
- [x] A round-trip test shows that the text the reader accepts opens what
      `orcvs::source::file::write` gives for it: exhaustively, over every byte alone and in lines,
      since the tooling contract keeps proptest out of `console`.
- [x] A test shows a Source opened from the URL is stopped at startup and that frames alone never
      start it. (Playback starts only from an input event, so a selected MIDI destination cannot
      start it; the test drives frames with no destination rather than a fake one.)
- [x] Native builds read no URL, and no shipped native item gains a parameter for this.
- [x] `cargo clippy --package console --all-targets --locked -- -D warnings`,
      `cargo nextest run --package console --locked`, and `mise run check_wasm` pass.

## Comments

### Shipped, 2026-10-26

`console/src/linked_source.rs` reads the URL; `Console::start_at` is the web entry point and
`Console::start` stays the native one. The limit is 16 KiB of fragment, which holds 21 full Grid
rows with every byte percent-encoded. A link that names both an example and a Source is refused
rather than choosing one, so an ambiguous link never opens something its author did not mean.
