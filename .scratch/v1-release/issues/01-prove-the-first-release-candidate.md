# 01 — Prove the first release candidate

**What to build:** Close the release definition with the complete candidate-bound evidence bundle
and an explicit, unwaived human GO/NO-GO decision.

**Blocked by:** v1-release/04; restyle-egui-console/03; v1-roadmap-wayfinding/07.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Every item in `v1-release/definition-of-done.md` links to executable evidence or the one
      accepted-deferral reconciliation record.
- [ ] The record names candidate SHA, pinned toolchain, clean-checkout procedure, exact commands,
      exit statuses, and authoritative Linux, macOS, persistence, WASM, Firefox, rustdoc, and
      dependency-policy results.
- [ ] The inventory-to-evidence matrix covers every shipped value, Function, spatial/Tick behavior,
      and terminal output with positive and applicable boundary/failure evidence.
- [ ] Exhaustive finite-domain results and at-least-256-case property summaries link every committed
      regression.
- [ ] Product persistence evidence covers model authority/rebuild and native/WASM
      save–restart–reload behavior on the candidate.
- [ ] The eight workflow captures, the two macOS spot-check captures, and physical MIDI evidence carry the required candidate metadata and
      human checklists; fake MIDI results prove exact deterministic software behavior.
- [ ] The benchmark record links the candidate's `.github/workflows/bench.yml` run and the
      archived `gh-pages` series, reviews the series history for cumulative drift, and records the
      reviewer's performance judgment against the nominated baseline. `mise run bench` measures but
      does not compare — the comparison lives in the action, as `docs/tooling.md` and
      `.scratch/benchmarks/spec.md` state — so a local run is not the evidence.
- [ ] The Criterion baseline commit is nominated, with its toolchain pinned, before `v1-release/03`
      cuts the candidate SHA — not chosen afterwards from whatever the comparison favours. Nominate
      it from the series `benchmarks/03` publishes to `gh-pages`, and record the nomination here
      with its date.
- [ ] Known defects, Improvement-only work, accepted deferrals, `CONTEXT.md`, and user-facing
      documentation are reconciled against implemented behavior rather than speculative scope. The
      review names `verification-gaps/09` (advisory benchmark gate and branch-protection bypasses)
      explicitly, and records a decision to
      accept each or to move it into `release/v1`.
- [ ] Every Benchmark floor breach on the series since the last clean run is cleared by a later
      run or accepted by the reviewer with its reason. Known at nomination time: `execute` measured
      108 ns against its 100 ns floor on `fbc1344c` (run 35855592973).
- [ ] A named reviewer and date conclude with `GO` only when every requirement passes; otherwise the
      ticket records `NO-GO`. Missing evidence cannot be waived inside this ticket.

## Candidate record

Append the completed evidence index, known-defect and deferral review, reviewer, date, and explicit
`GO` or `NO-GO` here. Resolving this ticket closes the dependency sink declared by
`.scratch/ROADMAP.md`; the roadmap retains that settled Gate as release history.

## Comments

**2026-09-24 — benchmark decisions recorded.**
- `verification-gaps/09`: **accepted as advisory for v1.** The benchmark comparison and floor check stay advisory, and branch protection is unchanged. This ticket's series and floor review stands in for enforcement. Candidate evidence stays trustworthy because it is bound to the exact SHA and re-verified by CI.
- `benchmarks/07`: joined `release/v1` and resolved (`benches/floors.toml`, #119 and #124), so the known-defect line no longer names it.
- `verification-gaps/15`: out of the release. `v1-release/03` requires the candidate to carry its own push-triggered Benchmark run instead.
- The `execute` floor breach after #129 is tracked by the new floor-breach line above.
