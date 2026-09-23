# 10 — Decide whether a benchmark floor breach blocks merge

**What to build:** A decision, and then its enforcement, on whether a failed Benchmark job, a floor breach in particular, stops a pull request merging. Today it does not.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] Decide: is the floor check a merge gate, or advisory?
- [ ] If it is a gate, make it a required status that works with `bench.yml`'s path filters. A required check that is skipped by a path filter never reports, and blocks every pull request that does not touch those paths.
- [ ] Record the decision in `docs/tooling.md`, and in ADR or `.scratch/benchmarks/spec.md` form if it changes the tier contract.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** `main`'s required status contexts are `full-gate`, `macos` and `wasm`; Benchmark is not among them. Two breaches reached `main` this way:
- PR #121's Benchmark `pull-request` job (run 35800814469) failed all four `paint_derive` floors (`fitted/16x16` 4880 ns against 4200), and the PR merged. `theming/08` said the floor "is read on the pull request as usual".
- `main`'s Benchmark run 35805579171, after #123, failed `fitted/256x256` at 1,560,683 ns against 1,400,000.

#125 and #126 recovered the cost, and `main` is under every floor as of run 35821771051. The cause of #121's breach was not confirmed; translucent tints taking `blend_channel`'s blend path is the likely candidate.
