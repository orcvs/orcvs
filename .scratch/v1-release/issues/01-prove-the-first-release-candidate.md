# 01 — Prove the first release candidate

**What to build:** Close the release definition with the complete candidate-bound evidence bundle
and an explicit, unwaived human GO/NO-GO decision.

**Blocked by:** 04 — Record physical MIDI evidence; restyle-egui-console/03 — Native/WASM visual
verification.

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
- [ ] Four visual captures and physical MIDI evidence carry the required candidate metadata and
      human checklists; fake MIDI results prove exact deterministic software behavior.
- [ ] `mise run bench` output compares the candidate with a named stable baseline, links archived
      output, reviews series history for cumulative drift, and records the reviewer's performance
      judgment.
- [ ] Known defects, Improvement-only work, accepted deferrals, `CONTEXT.md`, and user-facing
      documentation are reconciled against implemented behavior rather than speculative scope.
- [ ] A named reviewer and date conclude with `GO` only when every requirement passes; otherwise the
      ticket records `NO-GO`. Missing evidence cannot be waived inside this ticket.

## Candidate record

Append the completed evidence index, known-defect and deferral review, reviewer, date, and explicit
`GO` or `NO-GO` here. Resolving this ticket closes the dependency sink declared by
`.scratch/ROADMAP.md`; the roadmap retains that settled Gate as release history.
