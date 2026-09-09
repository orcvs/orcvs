# 01 — Repair the WASM browser test against the Output Command seam

**What to build:** The browser regression suite compiles and runs again, so a push to `main` reports green. The suite asserts against the Output Command an adapter records, not the Play Command the Playback Engine resolves before an adapter ever sees one.

**Blocked by:** None — can start immediately. `main` is red until it lands.

**Status:** resolved

- [x] `mise run test_wasm` compiles.
- [x] The Raw Play regression asserts on the Output Command the adapter records.
- [x] A push to `main` reports the WASM job green.

## Comments

The seam moved when Timed Play's Note Off scheduling landed: the in-memory adapter began recording Output Commands, and the browser suite had last been touched three commits earlier. Nothing carried it across, and nothing compiled it, so the break sat on `main`.

Raw Play maps one-to-one onto a Note On carrying the same three operands, so the assertion keeps its meaning rather than being weakened to fit.

An implementation exists uncommitted in the working tree. It compiles for `wasm32-unknown-unknown` and passes the repaired compile gate, but the browser run itself has not executed locally — geckodriver is killed under the agent sandbox — so CI is the first place the assertion actually runs.

### Evidence for the green WASM job on `main`

The repair to `shell/tests/wasm.rs` landed as `f3d3fec` on the `01-close-verification-gaps` branch and merged to `main` as `13ed044` (pull request #21). The `Rust` workflow run for that push reports the `wasm` job green: <https://github.com/orcvs/orcvs/actions/runs/33962851971>.

That run is the red-to-green transition rather than an unrelated pass. The push immediately before it, `2f3ed99` (pull request #19), ran the same `wasm` job red: <https://github.com/orcvs/orcvs/actions/runs/33949241036>.

The assertion itself executed, not only the compile. On a push to `main` the `wasm` job in `.github/workflows/test.yml` runs `mise run check_wasm` and then `mise run check_merge` with `ORCVS_MERGE_COMPONENT=wasm`, which is the headless browser suite.

The job has stayed green on every later push to `main`, most recently `72bb2cc`: <https://github.com/orcvs/orcvs/actions/runs/34298038844>.
