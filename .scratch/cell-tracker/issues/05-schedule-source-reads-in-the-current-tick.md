# 05 — Integrate Source reads with current-Tick execution

Status: needs-triage
Blocked by: 04

## Work

Implement 03's dependency contract through the production Source Tick path.
Connect the reader to its address/index operands and possible Source suppliers,
then bind the current characters only when those dependencies have settled.
Reuse Select when the accepted read returns a Sequence.

## Acceptance

- [ ] A Clock-driven pattern selects the expected first step at Tick 0, holds
      it for the requested rate, wraps at the declared length, and keeps cycling
      past Tick 255 without byte-truncating absolute time.
- [ ] The actual Source executes through at least two cycles with no diagnostics
      for valid data; no test substitutes an evaluator result for the read.
- [ ] A writer changes the selected step in the same Tick and MIDI receives the
      new Note, with the writer placed both before and after the reader in Grid order.
- [ ] Tests exercise changing index/geometry operands, competing and partial
      writes, failed writers, read/output overlap and dependency cycles as decided.
- [ ] Output obeys complete-fit and current-width rules, and preserves the
      distinction between typed nested values and contextual spatial encodings.
- [ ] Atomic Tick publication and bounded execution remain intact; Source contains
      all persistent musical data. No second parser reconstructs ownership.

Changes touch the scheduling and execution paths named in 03. Add a benchmark
only for a claimed cost change; a local comparison run is deferred to CI under
the repository contract.
