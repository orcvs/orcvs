# 04 — Measure the warm gate and decide on a fast lang job

**What to build:** Measure the second green CI run, when `Swatinem/rust-cache` serves the
dependency artefacts. Report the wall-clock time of the pull-request tier. Then decide whether a
separate `lang` job is worth adding.

**Blocked by:** 03 — Move the slow gates to the merge trigger.

**Status:** ready-for-human

- [ ] The wall-clock time of the warm pull-request tier is recorded in this issue.
- [ ] The cache restore and save times are recorded separately from the compile time.
- [ ] A decision on the fast `lang` job is recorded, with the number that justifies it.

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
