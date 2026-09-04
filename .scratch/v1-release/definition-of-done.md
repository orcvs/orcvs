# First Release Candidate Definition of Done

The First Release Candidate proves the inventory decided in
`v1-roadmap-wayfinding/issues/01-name-the-shipped-language-inventory.md`. That issue is the
inventory of record: it names every member and gives the state of each one, so no count of shipped
or unshipped members is copied into this document. ADRs and `CONTEXT.md` remain design inputs until
implementation and this evidence agree; glossary presence alone does not make a capability shipped.

## Language and Source

- [ ] Canonical two-Cell syntax and generation cover every shipped Function, Bang, Number, Note,
      and Comment; Number and Note identity is contextual and Comments begin with `##`.
- [ ] Every successfully parsed evaluable Expression entry pairs one syntax expectation with one
      real runtime value. Incomplete and invalid Live Edits remain editable, diagnose without
      panicking, and never acquire placeholder values.
- [ ] The Language Map is the sole Source-derived interface for Language Units, Expressions, roots,
      Positions, Spans, Glyphs, and diagnostics; Source remains the only stored program state.
- [ ] Every shipped spelling and fixed signature is compiler-checked, unique, and round-trips
      through canonical Source.

## Shipped Function inventory

- [ ] Numeric `.+`, `.-`, `.|`, `.x`, `./`, `.%`, `.<`, `.>`, `.=`, `.v`, and `.^` implement the
      complete byte, diagnostic, Bang, conversion, and identity laws.
- [ ] Flat Sequences permit Bang Atoms, broadcast compatible Atomic Functions, and implement `:-`,
      `:#`, `:<`, `:&`, `:?`, and `:=` deterministically without nesting or partial results.
- [ ] Tick Functions `~.`, `~*`, `~+`, `~?`, `~%`, and `~>` use explicit Tick/Position inputs,
      deterministic randomness, and visible Source feedback without hidden cross-Tick state.
- [ ] Spatial behavior covers Bang activation and expiry, all four Directional Bang forms and
      Self-Banging Function movements, Jump chains `&^`, `&v`, `&<`, `&>`, Halt `*!`, Source-order
      turns, atomic writes, conflicts, later-root activation, and boundary diagnostics.
- [ ] MIDI terminal output covers Raw `!>`, Timed `!~`, Monophonic `!%`, Control Change `!c`, and
      Pitch Bend `!b` with explicit operand types, protocol ranges, ordering, scheduling, ownership,
      device lifecycle, and exact wire bytes.

## Reproducible semantic proof

- [ ] Every inventory member links to positive behavior and every applicable type, range, zero,
      boundary, ordering, failure, and diagnostic proof.
- [ ] All 256 Numbers, 128 Notes, canonical encodings, conversions, and finite binary numeric domains
      run exhaustively rather than through samples.
- [ ] Parser totality, Grid/Position laws, Language Map partition/recovery, and other structured
      domains run at least 256 shrinking property cases, with every discovered counterexample
      committed as a regression.
- [ ] The exact candidate SHA passes a clean-checkout `mise run check` and authoritative Linux,
      macOS, persistence, both-feature WASM, Firefox browser, rustdoc, and dependency-policy gates.

## Product evidence

- [ ] Native and WASM persistence store Grid and character Source as authority, reject malformed
      state, rebuild derived state, and restore through the shipped save–restart–reload paths.
- [ ] Four candidate-bound captures—native/WASM × wide/tall—record SHA, platform, viewport,
      procedure, and reviewer and pass the decided geometry, palette, semantic-state, diagnostic,
      and Cursor checklist.
- [ ] Deterministic fake MIDI tests prove exact bytes and lifecycle; one recorded physical-device
      smoke proves the exact candidate's native adapter and OS/port/device integration.
- [ ] Criterion output compares the candidate with a named stable baseline under the same toolchain,
      profile, workloads, and environment; archived output and history review rule out unacceptable
      point or cumulative regression in the claimed parse/interpret paths.

## Release decision

- [ ] Known correctness defects and open Improvement-only work are reviewed; any work required for
      correctness, safe implementation, or decided evidence has joined `release/v1`.
- [ ] User-facing documentation, `CONTEXT.md`, implemented behavior, and the shipped inventory agree.
- [ ] Candidate documentation and evidence use exactly the accepted deferrals below and introduce no
      implicit additional exclusions.
- [ ] A named reviewer and date record explicit `GO` only when every required item passes; any
      missing or failing evidence records `NO-GO`, with no informal waiver.

## Accepted deferrals

- Concrete Source Read `@<` and Source Write `@>` addressing beyond directional Jump.
- UDP `!u`, OSC `!o`, and their text/message values and transports.
- Application Command `!$` and its command value encoding.
- Cross-version Source, persistence-format, and Rust-interface compatibility while Orcvs remains
  pre-release.
- Tickets tagged only `Improvement`, unless correctness, safe implementation, or measured release
  evidence later requires one to join `release/v1` through a recorded roadmap decision.
