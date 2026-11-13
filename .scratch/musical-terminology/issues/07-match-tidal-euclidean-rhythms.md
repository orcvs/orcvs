# 07 — Match Tidal's Euclidean rhythms

**What to build:** A pattern copied from Tidal sounds the same in Orcvs.

**Blocked by:** 01 (the reference implementation is settled).

**Status:** ready-for-agent

- [ ] For every 1 ≤ `hits` ≤ `steps` ≤ 32, `~% hits steps` equals Tidal's `bjorklund`, checked
      against a reference implementation written in test code only.
- [ ] Zero hits is silence and zero steps is refused, as today, each with its own test.
- [ ] Shipped uses whose pattern changes, `~%0304` among them, are updated with their tests.
- [ ] ADR 0012 is amended: the patterns match Tidal's, E(3,4) becomes `xxx.`, and Orca parity, the
      reason it gave for the bucket formula, is given up.
- [ ] The reference and glossary describe the Euclidean pattern as Tidal's, with an example, and the
      research notes stop calling the old formula Bjorklund's.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
