# Apply the decided release roadmap

Type: task

Blocked by: 05 — Design the release proof gate.

Status: resolved

## Question

Apply every resolved wayfinding decision to the local Markdown tracker: correct release tags,
ticket bodies and identities, dependency edges, `.scratch/ROADMAP.md`, the Definition of Done, and
the release proof gate. Regenerate the roadmap and verify that it represents the decided route.
Do not implement production behavior or alter accepted domain semantics during this task.

## Answer

Applied every resolved wayfinding decision to the Markdown tracker and its derived roadmap:

- Tagged 41 required implementation, semantic-proof, product-evidence, and final-gate tickets with
  `release/v1`; tagged the four independent maintenance tickets `Improvement`.
- Corrected Expression-entry pairing, strict/permissive parser properties, Language Map properties,
  complete numeric exhaustive proof, Bang-bearing Sequences, scalar whole-value Equality, the MIDI
  activation seam, Activation movement, and candidate visual evidence.
- Retired the superseded Expression Map spec without creating a competing issue set.
- Added the Activation representation prototype and distinct product-persistence, exact-candidate,
  and physical-MIDI evidence tickets.
- Replaced the Definition of Done with the decided inventory, exact semantic/target/product evidence,
  explicit release decision, and prose accepted-deferral list.
- Made `v1-release/01` the true dependency sink through the exact-candidate and artifact branches.
- Strengthened `scripts/roadmap.ts`: an open declared Gate must depend on every open tagged issue; a
  settled declared Gate is retained as release history only when no tagged work remains open.
- Regenerated the text and HTML roadmap. It reports 41 release issues, a 10-ticket longest critical
  path, and 31 mandatory parallel tickets, all inside the Gate dependency closure.

### Completion evidence

Changed: local Markdown release scope, ticket membership/bodies/dependencies, four new release-route
tickets plus one prototype effort, Definition of Done, release gate, roadmap guidance, and roadmap
Gate validation.

Tests added or updated: three roadmap regressions cover a settled Gate, rejection of a prematurely
settled Gate, and rejection of tagged work outside the open Gate's dependency closure.

Commands run:

- `node --check scripts/roadmap.ts` — passed.
- `node --test scripts/tests/roadmap.test.ts` — passed, 8/8 tests.
- `pnpm roadmap` — passed; 41 release issues, critical path 10, parallel work 31.
- `node scripts/roadmap.ts --html` and `test -s .scratch/roadmap.html` — passed.
- `cargo fmt --all -- --check` — passed.
- `git diff --check` — passed.
- `mise run check` — failed in the existing tooling-contract preflight because
  `lang/Cargo.toml` contains the Criterion dependency merged on `origin/main`, while
  `scripts/check-tooling-contract.sh` asserts that the manifest must contain no Criterion entry.
  Neither conflicting file was changed by this roadmap work.

Not run: downstream Rust crate, WASM, persistence, dependency, or performance gates separately — no
production Rust, manifest, feature, dependency, platform, or performance behavior changed, and the
repository-wide gate stopped on the unrelated pre-existing preflight contradiction.

Risks: no public API, unsafe, dependency, feature, or performance change. Planning risk is guarded
by the generator's enforced Gate closure and focused regression tests.
