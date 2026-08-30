# 04 — Measure the warm gate and decide on a fast lang job

**What to build:** Measure the second green CI run, when `Swatinem/rust-cache` serves the
dependency artefacts. Report the wall-clock time of the pull-request tier. Then decide whether a
separate `lang` job is worth adding.

**Blocked by:** 03 — Move the slow gates to the merge trigger.

**Status:** resolved

- [x] The wall-clock time of the warm pull-request tier is recorded in this issue.
- [x] The cache restore and save times are recorded separately from the compile time.
- [x] A decision on the fast `lang` job is recorded, with the number that justifies it.

## Comments

`lang` builds and tests from nothing in 9 seconds and produces a 49 MB target directory, because it
never compiles `eframe`. A separate job would give a 10-second signal on the `lang` properties and
would repeat only those 9 seconds.

Two facts argue against adding it before the measurement. The invariants are split roughly evenly:
the parser round trip, the Number and Note conversions, and the arithmetic are in `lang`, while the
Grid round trip, the Language Map partition, and Glyph derivation are in the core crate. A `lang`
job therefore accelerates about half of the properties. And once the crate split lands, the core
crate no longer compiles the toolkit either, so the gap this job closes may already be closed.

Re-measure after `crate-boundaries` lands before deciding.

PR 2 run 33312928171, attempt 2, measured the warm gate. The Linux job took 44 seconds wall-clock,
and the `check_pull_request` step took 12 seconds. Cargo reported 1.94 seconds for clippy's check,
8.27 seconds for the native test build, and 0.15 seconds for doctest compilation. The Linux cache
restore step took 8 seconds. It downloaded 432,143,224 bytes in about 1.6 seconds and completed
extraction about 2.8 seconds later. The save step found the cache up to date and took about 0.12
seconds. For comparison, the warm macOS job took 39 seconds and its pull-request tier took 14
seconds.

Do not add a separate `lang` job. The complete warm Linux pull-request tier is only 12 seconds. A
new job would add runner, toolchain, mise, and cache overhead to accelerate only part of that
12-second step. Revisit the decision only if the crate split changes the measured balance.
