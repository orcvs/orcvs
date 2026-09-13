# Portal geometry verification

Changed: Portal owns complete row fit and remaining-row coverage. Feedback reads,
write admission, and Reservations consume that geometry. Execution relationship
lookup consumes admitted SpanWrite coverage.

Tests added: exact complete fit and row-edge refusal at the Portal interface;
remaining-row coverage at first, interior, and last columns and in a one-column Grid.
Each new interface test first failed compilation because its method did not exist.
Existing Source behavior tests were retained.

Commands run:

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package orcvs --package console --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(source::portal) | test(source::tick)' --status-level fail --final-status-level fail` — passed, 120 tests.
- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.

Not run: full final workspace tests, doctests, platform and feature matrix,
full-case proptest, and benchmark comparisons — deferred to CI at the user's request.

Risks: internal interface refactor only; no public API, unsafe, concurrency,
dependency, or feature changes. No performance claim. General Grid and Span
validation and scheduling semantics remain outside this change.
