# Design the release proof gate

Type: grilling

Blocked by: 03 — Define the release evidence contract; 04 — Reconcile roadmap membership and
dependencies.

Status: resolved

## Question

What final gate topology, Definition of Done, and go/no-go record prove the decided inventory after
the reconciled implementation graph and evidence contract are satisfied? Decide how critical-path
work, parallel release work, accepted deferrals, target-specific evidence, and remaining
Improvement-only work appear without allowing the gate to close while required release work remains.

## Answer

`v1-release/01` remains the one release-closing sink declared by `.scratch/ROADMAP.md`. Its
direct/transitive `Blocked by` closure must contain every open issue tagged `release/v1`; resolving
the gate therefore proves that no required implementation, semantic-proof, or candidate-evidence
branch remains open.

The gate directly names only terminal branches: implementation or proof work that has no later
release ticket to carry it into the closure, plus distinct candidate-bound artifact work. Real
implementation prerequisites remain on their owning tickets. No artificial dependency serializes
independent work merely to make the displayed route look linear.

The roadmap generator's critical subgraph contains every zero-slack issue on any unit-weight
longest blocker path to the gate. Tied prerequisite branches are all critical even when they can
run in parallel; delaying any one delays the gate. Tagged issues with positive slack appear as
parallel work but remain mandatory because they are still in the gate's blocker closure. The
rendered view must not describe parallel release work as optional.

The Definition of Done is the human-readable acceptance index. It names the decided shipped
inventory rather than treating speculative ADR text as shipped, links each requirement to the
conjunctive evidence contract, and separates accepted non-goals from completion checkboxes. One
reconciliation check confirms that candidate documentation and evidence use exactly those
deferrals; UDP, OSC, Source addressing, Application Command encoding, compatibility, and unrelated
Improvements do not appear as unchecked implementation tasks.

The final gate record contains:

- candidate SHA, pinned toolchain, clean-checkout commands, exit statuses, and authoritative CI;
- Linux/macOS native and Firefox WASM target/feature results;
- the complete inventory-to-evidence matrix, exhaustive results, and 256-case property summaries;
- product persistence, four rendered captures, fake MIDI evidence, and physical MIDI smoke;
- named-baseline Criterion output and series-history judgment;
- known-defect review, accepted deferrals, Improvement-only visibility, and documentation agreement;
- named reviewer, date, and explicit unwaived `GO` or `NO-GO`.

Any missing or failing required item forces `NO-GO`. Changing a requirement requires another
recorded planning decision, not a waiver inside the gate.

After `v1-release/01` resolves, `.scratch/ROADMAP.md` retains `Gate: v1-release/01` as the historical
closure identity. The roadmap generator must accept a settled tagged declared gate and render the
release as complete instead of requiring the gate to remain falsely open or deleting its identity.
