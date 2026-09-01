# 01 — Add proptest for native targets

**What to build:** Add `proptest` as a dev-dependency for non-WASM targets in `lang` and `orcvs`.
Wire the case count to the verification tier, and commit the counterexample files.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `proptest` is declared as a workspace dependency and used by `lang` and `orcvs`.
- [ ] The dependency is confined to `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`.
- [ ] Property modules carry the matching `cfg`, so `wasm-pack test` compiles without proptest.
- [ ] The pull-request tier sets `PROPTEST_CASES=32`; the merge tier uses the 256-case default.
- [ ] `.proptest-regressions` is committed and is not ignored by `.gitignore`.
- [ ] `cargo deny --locked check` passes with the new dependency graph.
- [ ] `mise run check_wasm` and `mise run test_wasm` are unaffected.

## Comments

The dependency addition needs a recorded rationale under the repository contract. `AGENTS.md` already
asks for property tests at the parser boundary, so cite that.

An uncommitted counterexample is a failure you can see in CI and cannot reproduce. That is worse than
no property at all, so treat the regression files as source.

Do not add `cargo-fuzz` in this issue. `AGENTS.md` defers fuzzing to "when exposure warrants it", and
that is a separate decision with its own cost.
