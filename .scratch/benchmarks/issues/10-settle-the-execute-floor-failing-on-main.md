# 10 — Settle the `execute` floor failing on main

**What to build:** The `lang` `execute` benchmark (`lang/benches/lang.rs`, `Interpreter::execute`) passes its floor in `benches/floors.toml` reliably on `main`, or the floor is revised with evidence. The Benchmark workflow's push run on `main` at `0dd4fe9d` (#136, theme TOML) failed `check-bench-floors` at 114 ns against the 100 ns floor; the preceding push runs at `274ac046` and `199c3331` passed. Pull requests that do not touch `lang` then measured 113 ns (#146) and 101 ns (#147), while other runs passed, so every pull request's Benchmark check can fail independently of its change.

**Blocked by:** None — can start immediately.

**Status:** wontfix

- [ ] Establish whether `0dd4fe9d` introduced a regression in `execute` or the floor sits inside CI runner variance, using reproducible measurements of the commits either side.
- [ ] If a regression: fix it, or record why the cost is accepted, and keep the floor.
- [ ] If runner variance: revise the floor or the measurement with the recorded evidence ticket 07 requires for a floor.
- [x] The Benchmark push run on `main` passes.

## Comments

**2026-09-25 — filed during source-audit PRs 1–5.** Observed on #146 and #147; not caused by either. Until settled, a failure of this floor alone is treated as known.

### Audit at cad296df — 2026-09-29

Wontfix: the subject is gone. `26167b95` deleted `Interpreter::execute` and its benchmark; `lang/benches/lang.rs` has no `execute` bench, and `benches/floors.toml:47-49` now floors `execute_function` at 60 ns. The failures described were real (run 36221358068: "execute: measured 108 ns, exceeds the 100 ns floor"). The regression-or-variance question for `0dd4fe9d` was never answered and no longer needs to be. The `main` push run on `cad296df` (36508316780) passed with `execute_function` at 39 ns. PR run 36225367117 once measured it at 62 ns against the 60 ns floor; if that floor needs settling, it is a new ticket.
