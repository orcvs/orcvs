# 07 — Skip the compile when no Rust input changed

**What to build:** A pull request that touches only documentation or `.scratch/` passes the required `ci` check without compiling the workspace.

**Blocked by:** 06 (stacked on it; independent in substance)

**Status:** resolved

- [x] A pull request whose diff is Markdown alone skips the `full-gate` and `wasm` compiles. The rule is narrower than "no Rust input": a console test includes a `.toml` from `.scratch/`, so only `.md` paths are exempt.
- [x] The `ci` aggregate still reports on every pull request and passes for a skipped compile only when no Rust input changed.
- [x] The merge queue and pushes to `main` always run the full tier.
- [x] The tooling contract and `scripts/tests/check-ci-results.sh` cover the new skipped-is-success case and reject it when Rust inputs changed.

## Comments

The input set must be generous: a file that can change what the compile or a gate sees belongs in
it. `docs/tooling.md` is read by no gate. `.scratch/` is read by the roadmap planner, which runs in
`check_pull_request`, so a `.scratch/`-only change still needs the planner's two node lines even
when it skips cargo.
