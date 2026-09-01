# Define the release evidence contract

Type: grilling

Blocked by: 01 — Name the shipped language inventory.

Status: resolved

## Question

What exact reproducible evidence is necessary and sufficient to accept the First Release Candidate
for canonical Source parsing and generation, native and WASM behavior, persistence, MIDI output,
parser and protocol boundaries, and performance regression? Distinguish exhaustive tests,
properties, target builds, rendered or hardware evidence, benchmark history, and human go/no-go
review so the proof gate cannot be closed by an ambiguous “tests pass”.

## Answer

First Release Candidate acceptance is a conjunction of semantic, target, product, performance, and
human evidence. No single command, screenshot, hardware session, or review can substitute for the
other classes. Every artifact is tied to the exact candidate commit, and every shipped inventory
member maps to specific evidence.

### Candidate identity and automated gate

- Record the candidate commit SHA, the pinned toolchain from `rust-toolchain.toml`, the clean-checkout
  procedure, exact commands, exit status, and authoritative CI links.
- `mise run check` must pass from a clean checkout of that SHA. The same SHA must have green native,
  persistence, WASM build, and browser jobs before the go/no-go decision; ordinary post-merge WASM
  scheduling cannot leave candidate evidence pending.
- Native compilation and tests cover Linux and macOS. WASM proof covers both declared feature
  builds and the headless Firefox browser suite. Windows and additional browsers are not FRC claims.
- Target builds prove portability and integration; they do not replace semantic tests.

### Language and protocol semantics

- Every shipped value, Function, spatial behavior, Tick behavior, and terminal output has at least
  one positive behavior proof plus every applicable type, range, boundary, ordering, zero, failure,
  and diagnostic proof. The release record links this inventory-to-evidence matrix.
- Finite domains use exhaustive tests rather than sampling: all 256 Numbers and canonical encodings,
  all 128 Notes and canonical encodings, every defined conversion case, and all 65,536 input pairs
  for each byte-binary arithmetic law whose domain has that size.
- Canonical generated Source round-trips through parsing with the same typed value or Function. All
  declared Function spellings and signatures are unique and compiler-checked.
- Large structured spaces use shrinking property tests. The release campaign runs at least 256
  cases per property for parser totality, Grid/Position laws, Language Map row partitioning and
  recovery, and other accepted structured boundaries. Every discovered counterexample becomes a
  committed regression before acceptance.
- Native semantic proof is sufficient for platform-independent pure language laws; those tests need
  not be duplicated mechanically on every target. Fuzzing and stateful edit-sequence models are not
  required unless implementation exposure produces a concrete parser/protocol risk that warrants
  them.

### Persistence

The FRC makes a product-level persistence claim, not merely a serde claim. Automated model tests
must prove that Grid and character Source are the stored authority, malformed dimensions are
rejected, and all derived language state is rebuilt after round-trip. In addition, the evidence
bundle records save/restart/reload behavior through each shipped native and WASM storage path; use
an automated end-to-end test where feasible and a repeatable recorded smoke where the host storage
integration cannot be automated reliably.

### MIDI

Deterministic fake-adapter tests are authoritative for software behavior. They assert the exact MIDI
wire bytes, Tick Plan order, zero cases, Timed and Mono expiry/replacement, channel independence,
Control Change, Pitch Bend byte order, selection, failure, reconnection, destination change,
all-notes-off, and ownership cleanup without requiring hardware.

The candidate also requires one recorded physical-device smoke test of the native adapter. Record
the candidate SHA, OS, MIDI device, connection and playback procedure, expected observation, actual
result, and reviewer. This manual evidence proves OS/port/device interoperability; it does not
replace the deterministic fake tests.

### Rendered product evidence

Record four captures from the candidate: native wide, native tall, WASM wide, and WASM tall. Each
artifact records SHA, OS or browser, viewport dimensions, capture procedure, and reviewer. A human
checklist verifies square Cell geometry, complete wide/tall Grid presentation, semantic palette,
occupied and empty Cells, Function/Number/Note/Bang/Activation/diagnostic distinctions, and Cursor
treatment. Existing unversioned WASM-only captures do not satisfy this contract.

### Performance

Run `mise run bench` for the candidate and compare it with a named stable baseline using the same
toolchain, benchmark profile, workloads, and measurement environment. Archive or link the Criterion
output and inspect the benchmark series for cumulative drift. The CI `300%` previous-point limit is
only a catastrophic regression guard; passing it is not sufficient acceptance evidence. A human
reviewer records whether observed movement is acceptable and why. The FRC performance claim covers
only the benchmarked parse and interpret paths unless a release-critical path receives its own
reproducible benchmark.

### Human go/no-go record

The final record contains the candidate identity, target/feature matrix, exact command results and
CI links, exhaustive/property summaries, inventory traceability, persistence artifacts, MIDI fake
and hardware evidence, rendered captures, benchmark comparison and history review, known-defect
review, and the accepted deferrals. A named reviewer and date conclude with explicit `GO` or
`NO-GO`.

There are no informal waivers. Any missing or failing required evidence means `NO-GO`. Changing a
requirement requires a recorded roadmap decision rather than a checkbox exception.
