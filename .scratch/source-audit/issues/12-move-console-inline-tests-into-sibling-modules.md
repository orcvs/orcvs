# 12 — Move console.rs inline tests into sibling test modules

**What to build:** The console module's source file holds only shipped code; its roughly six thousand lines of inline tests (`run_clock_tests`, `tests`, `storage_tests`) move into sibling test modules, as the kittest tests already are. A prefactor with no behaviour change. Other console modules keep their inline `mod tests` by crate convention and are out of scope.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The console module's source file contains no test module bodies.
- [x] Every moved test still runs and passes; the test count is unchanged (106 test functions in `console.rs` at `199c3331`).
- [x] No shipped item's visibility widens to serve the move beyond what the existing sibling test module already requires.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid. `console.rs` is 9039 lines, about 5990 of them inline tests. Baseline test count recorded.

**2026-09-25 — implementation (epic PR 15).** Delivered in orcvs/orcvs#158, stacked on #157. `run_clock_tests`, `tests` and `storage_tests` move to `console/src/console/{run_clock_tests,tests,storage_tests}.rs` beside `kittest_tests.rs`; `console.rs` keeps only the three `mod` declarations (9132 lines to 3063). The move commit is mechanical — one dedent, rustfmt reflow, the `storage_tests` doc becoming the file's inner doc — and changes no cfg or shipped visibility. Count at refactor start (`6ba42397`, PR 14's head): 496 native console tests, 108 of them in the moved modules, and 9 browser tests in `console/tests/wasm.rs`; the nextest list is identical after the move.
