# 02 — Add Clock, Delay, and Euclidean Functions

**What to build:** Implement `~.`, `~*`, and `~%` from ADR 0012 using explicit absolute Tick and
Number operands.

**Blocked by:** 01 — Thread Tick and Position into interpretation;
`.scratch/orcvs-language-migration/issues/01-move-arithmetic-onto-the-dot-family.md`; and
`.scratch/orcvs-language-migration/issues/02-free-the-bang-and-activation-spellings.md`.

**Status:** ready-for-agent

- [ ] Clock returns `floor(Tick / rate) % modulus`.
- [ ] Delay Bangs exactly when `Tick % (rate * modulus) == 0`, including Tick `0`.
- [ ] Zero rate or modulus diagnoses and the cycle product cannot byte-wrap.
- [ ] Euclidean follows ADR 0012's formula and phase exactly.
- [ ] Euclidean handles zero hits, full hits, zero steps, and hits greater than steps.
- [ ] Sequence operands follow the ordinary broadcasting rules once available.
- [ ] Tick-by-Tick tests use explicit Source Grids and diagnostics.
