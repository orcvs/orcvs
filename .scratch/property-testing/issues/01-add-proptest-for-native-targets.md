# 01 — Add proptest for native targets

**What to build:** Add `proptest` as a dev-dependency for non-WASM targets in `lang` and `orcvs`.
Wire the case count to the verification tier, and commit the counterexample files.

**Status:** resolved

**Tags:** release/v1

- [x] `proptest` is declared as a workspace dependency and used by `lang` and `orcvs`.
- [x] The dependency is confined to `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`.
- [x] Property modules carry the matching `cfg`, so `wasm-pack test` compiles without proptest.
- [x] The pull-request tier sets `PROPTEST_CASES=32`; the merge tier uses the 256-case default.
- [x] Git does not ignore the representative `proptest-regressions/` counterexample path for
      each crate: `lang/proptest-regressions/parser.txt` and `orcvs/proptest-regressions/grid.txt`.
- [x] `cargo deny --locked check` passes with the new dependency graph.
- [x] `mise run check_wasm` and `mise run test_wasm` are unaffected.

## Downstream integration

- [ ] The full Grid and Position suite is owned by `issues/02-grid-position-round-trip.md`.
- [ ] The full parser totality suite is owned by `issues/03-parser-totality-on-ascii-input.md`.

## Comments

The dependency addition needs a recorded rationale under the repository contract. `AGENTS.md` already
asks for property tests at the parser boundary, so cite that.

An uncommitted counterexample is a failure you can see in CI and cannot reproduce. That is worse than
no property at all, so treat the regression files as source.

Do not add `cargo-fuzz` in this issue. `AGENTS.md` defers fuzzing to "when exposure warrants it", and
that is a separate decision with its own cost.

### Correction: the counterexample path

This ticket originally required that "`.proptest-regressions` is committed and is not ignored by
`.gitignore`". That filename is wrong, and as written the requirement was unsatisfiable. proptest
1.11's default persistence is `SourceParallel("proptest-regressions")`: a **directory** without a
leading dot, sibling to each crate's `src`, holding one `.txt` per module — for example
`lang/proptest-regressions/parser.txt`. Confirmed against the vendored source for the version in
`Cargo.lock` and empirically, by breaking a property and observing the artefact appear untracked.

Nothing is committed today because no property fails, so there is no counterexample to commit. The
half of the rule that can be enforced now is enforced: `scripts/check-tooling-contract.sh` asks
`git check-ignore` whether a representative path in each crate is ignored, which catches a broad
glob or a nested ignore file as well as a literal rule. The checkbox above is reworded to the
requirement that can actually hold.

### What landed

`proptest` is declared once in `[workspace.dependencies]` with default features off and only `std`
enabled, and consumed by `lang` and `orcvs` under the non-WASM dev-dependency table. `docs/tooling.md`
carries the rationale, the tier split, and the counterexample rule.

The confinement is mechanical rather than conventional. `scripts/check-tooling-contract.sh` pins the
target table in both crates, the absence of proptest from every shipped `[dependencies]` table, from
the plain `[dev-dependencies]` tables that also compile for WASM, and from `shell`; that
`PROPTEST_CASES` is set exactly once in `mise.toml`, in the pull-request tier; and that git does not
ignore the counterexample files. Every one of those was negative tested by mutating the fact it pins.

Each crate carries one narrow seed property, because a declared but unused dev-dependency proves
nothing about the `cfg` gating. `lang` checks that strict parsing is total over printable ASCII;
`orcvs` checks that `position_at` inverts `index` for a generated Position. Both are deliberately
smaller than the suites in the Downstream section, which own their full acceptance lists.

### Verification

Every criterion was re-checked against the code rather than taken from the checkbox, on commit
`6f61049`:

- Workspace declaration and use: `proptest = { version = "1.11.0", default-features = false,
  features = ["std"] }` in the root `[workspace.dependencies]`; `proptest.workspace = true` in both
  `lang/Cargo.toml:33` and `orcvs/Cargo.toml:50`. Absent from `shell/Cargo.toml`.
- Confinement: both crates declare it only under
  `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`. Checked mechanically as well as by
  reading — `cargo tree -e normal,dev` finds one `proptest` node per crate for
  `x86_64-apple-darwin` and zero for `wasm32-unknown-unknown`.
- Module `cfg`: `#[cfg(all(test, not(target_arch = "wasm32")))]` on `lang/src/parser.rs:599` and
  `orcvs/src/grid.rs:533`.
- Tier split: `PROPTEST_CASES` appears exactly once in `mise.toml`, as task-level env on
  `[tasks.check_pull_request]`. The merge tier sets nothing, so it takes proptest's 256-case default.
- Counterexample paths: `git check-ignore` exits 1 for both representative paths. No
  `proptest-regressions` directory exists yet, which is expected while no property fails.
- `cargo deny --locked check` — exit 0; advisories, bans, licences, and sources all ok.
- `mise run check_wasm` — exit 0. `mise run test_wasm` — exit 0, 5 tests passed in headless Firefox.
- The two seed properties actually execute:
  `PROPTEST_CASES=32 cargo nextest run --package lang --package orcvs -E 'test(property)'` runs 2
  tests, both passing.

`docs/tooling.md` claimed the contract script pins the absence of any ignore rule. It pins the two
representative paths, the same overstatement this ticket's checkbox carried; the paragraph now
states what the script checks and why it asks git rather than reading `.gitignore`.

The Downstream integration boxes stay unchecked. `issues/02` and `issues/03` are both still
`ready-for-agent` and own those suites.
