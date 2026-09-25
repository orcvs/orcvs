# 12 — Move console.rs inline tests into sibling test modules

**What to build:** The console module's source file holds only shipped code; its roughly six thousand lines of inline tests (`run_clock_tests`, `tests`, `storage_tests`) move into sibling test modules, as the kittest tests already are. A prefactor with no behaviour change. Other console modules keep their inline `mod tests` by crate convention and are out of scope.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The console module's source file contains no test module bodies.
- [ ] Every moved test still runs and passes; the test count is unchanged (106 test functions in `console.rs` at `199c3331`).
- [ ] No shipped item's visibility widens to serve the move beyond what the existing sibling test module already requires.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid. `console.rs` is 9039 lines, about 5990 of them inline tests. Baseline test count recorded.
