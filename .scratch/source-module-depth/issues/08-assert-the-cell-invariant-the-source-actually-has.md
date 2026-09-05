# 08 — Assert the Cell invariant the Source actually has, on the Tick path

**What to build:** The one assertion standing between an interpreted result and the Source states
the rule a Cell really has. Every other statement of that rule is the printable ASCII range
`0x20..=0x7e`; the Tick path asserts only `is_ascii`.

**Blocked by:** None.

**Status:** resolved

**Tags:** release/v1

- [x] A Tick's committed result is held to the same rule a typed Cell is: one printable
      single-byte ASCII character.
- [x] The rule is stated once, where a Cell's content is defined, rather than again on the Tick
      path and again in the Portal.
- [x] The existing suite passes unchanged; no Source that ticks today ticks differently.

## Comments

`Source::check_content` accepts `0x20..=0x7e`, the persistence `Deserialize` validates
`0x20..=0x7e`, and `LanguageMap::derive` refuses anything outside `0x20..=0x7e`. The Tick path
asserts `encoded.is_ascii()` in `tick::emit_result`, and `Portal::admit` asserts it again over the
same encoding; both admit control bytes.

Nothing produces one today — every Interpreter result is a hexadecimal Number, a Note name, or a
Char echoing a Cell that was already validated — so this is a latent gap, not a live defect, and
it is not a soundness gap either: `set_source`'s unsafe block needs UTF-8 validity, and any ASCII
byte gives it that. What a control byte would break is the Source's own round trip, since
persistence refuses to read back what the Tick was allowed to write.

Found while rewriting that SAFETY comment to name the assertion it actually rests on. The comment
now records the discrepancy; this ticket removes it.

Worth pairing with the observation that `CellWrite.content` is a `char` and `commit_tick` narrows
it with `as u8`. The field is wider than the invariant, and the narrowing is lossy for anything
outside ASCII. Unreachable today — `execute` builds the only plans `commit_tick` sees — but the
type says less than the code assumes.

### Restated by `sequence-values/04`, 2026-09-05

Two things moved. The Tick-path assert is now in `tick::emit_result`, the seam that turns one
evaluation's answer into effects; `emit_expression_root` still exists but delegates to it. And
`Portal::admit` (`orcvs/src/source/portal.rs`) now asserts `is_ascii` over the same encoding a few
lines later, because it is the sole constructor of a `SpanWrite` and the `set_source` SAFETY
comment in `model.rs` names it as the guarantor. So this ticket has two Tick-path sites to narrow,
not one, and the Portal's is the one the safety argument actually rests on.


### Cell-content design and implementation, 2026-09-05

Confirmed during the branch architecture review: carry validated Cell content through planned
writes, and treat invalid generated content as an internal assertion rather than a language
diagnostic. `source::CellContent` now owns the printable-ASCII rule. Live Editing, persistence,
and Language Map derivation use its constructor; Portal admission retains those validated values
in `SpanWrite`, conflict resolution preserves them in `CellWrite`, and commit accepts the same
type without narrowing an unrestricted character. The Tick's duplicate assertion is removed.
The Cell glossary now explicitly says printable ASCII.

The new control-character regression failed before the change and passes afterwards. Exhaustive
byte classification and editing/derivation agreement tests cover the content domain; all printable
characters pass through Portal commit and, with persistence enabled, serialization and restoration.
Existing write assertions now read `CellContent::as_char()` or construct validated expected content.
No language spellings or normal Source behavior change.


Verification:

- `cargo test --package orcvs --locked generated_control_characters_are_internal_defects` —
  failed before implementation (no panic), passed afterwards.
- `cargo fmt --all -- --check` — passed.
- `cargo check --package orcvs --all-targets --locked` — passed.
- `mise run check_pull_request` — passed: workspace Clippy with warnings denied, 363 tests,
  and workspace doctests. These workspace checks cover the affected crate's scoped gate.
- `mise run check` — stopped at the pre-existing tooling contract mismatch: the check requires
  an actions/checkout v4 comment but `.github/workflows/test.yml` uses v7.0.1.
- `mise run check_merge` — persistence Clippy, 383 tests, doctests, rustdoc with warnings denied,
  dependency checks, WASM Clippy, and both WASM application builds passed. The final browser-test
  compilation failed in unchanged `shell/tests/wasm.rs:63`, comparing OutputCommand to PlayCommand;
  browser tests could not execute. This task invokes `mise run test_persistence` and
  `mise run check_wasm` directly before the failing `mise run test_wasm` stage.
- `cargo +nightly miri test --package orcvs --lib --features persistence --locked every_printable_character_survives_portal_commit`
  — passed (one focused test). Installed the missing nightly Miri component for this run.
- `git diff --check` — passed; complete diff reviewed.

Risk: `CellWrite.content` now uses the validated public CellContent type in this internal,
unpublished application; existing consumers were checked by workspace compilation. The existing
unsafe write now accepts proof of its content precondition. No dependencies, features, concurrency,
or language spellings changed, and no performance claim is made. No new unsafe block was added.
