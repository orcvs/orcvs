# 01 — Repair the WASM browser test against the Output Command seam

**What to build:** The browser regression suite compiles and runs again, so a push to `main` reports green. The suite asserts against the Output Command an adapter records, not the Play Command the Playback Engine resolves before an adapter ever sees one.

**Blocked by:** None — can start immediately. `main` is red until it lands.

**Status:** in-review

- [x] `mise run test_wasm` compiles.
- [x] The Raw Play regression asserts on the Output Command the adapter records.
- [ ] A push to `main` reports the WASM job green.

## Comments

The seam moved when Timed Play's Note Off scheduling landed: the in-memory adapter began recording Output Commands, and the browser suite had last been touched three commits earlier. Nothing carried it across, and nothing compiled it, so the break sat on `main`.

Raw Play maps one-to-one onto a Note On carrying the same three operands, so the assertion keeps its meaning rather than being weakened to fit.

An implementation exists uncommitted in the working tree. It compiles for `wasm32-unknown-unknown` and passes the repaired compile gate, but the browser run itself has not executed locally — geckodriver is killed under the agent sandbox — so CI is the first place the assertion actually runs.
