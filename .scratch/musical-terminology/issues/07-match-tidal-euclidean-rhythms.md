# 07 — Match Tidal's Euclidean rhythms

**What to build:** A pattern copied from Tidal sounds the same in Orcvs.

**Blocked by:** 01 (the reference implementation is settled).

**Status:** resolved

- [x] For every 1 ≤ `hits` ≤ `steps` ≤ 32, `~% hits steps` equals Tidal's `bjorklund`, checked
      against a reference implementation written in test code only.
- [x] Zero hits is silence and zero steps is refused, as today, each with its own test.
- [x] Shipped uses whose pattern changes, `~%0304` among them, are updated with their tests.
- [x] ADR 0012 is amended: the patterns match Tidal's, E(3,4) becomes `xxx.`, and Orca parity, the
      reason it gave for the bucket formula, is given up.
- [x] The reference and glossary describe the Euclidean pattern as Tidal's, with an example, and the
      research notes stop calling the old formula Bjorklund's.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 14 November 2026. `lang`'s Euclidean body places its onsets as Tidal's `bjorklund`
does, for any `steps` up to `FF`, by walking the pairing's levels as counts and lengths, so it
copies no step of any group. A
transcription of Tidal's `Sound.Tidal.Bjorklund`, in test code only and itself checked against
published patterns, is compared with `~%` for every 1 ≤ `hits` ≤ `steps` ≤ 32 over two cycles. Zero
hits is silence and zero steps is refused, each with its own test, and sampled patterns up to `FF`
steps are compared with the reference too. `~%0304` appeared only in `orcvs`
tests, which now expect `XXX.`; the shipped examples and the console's Function Reference use
`~%0104` and `~%0308`, whose patterns do not change. `docs/functions.md` and `CONTEXT.md` describe
the patterns as Tidal's with `~%0304` as a runnable example, `docs/research/timing-operators.md`
stops calling the old formula Bjorklund's, and a dated amendment to ADR 0012 records the change,
E(3,4) as `XXX.`, and the loss of Orca parity.
