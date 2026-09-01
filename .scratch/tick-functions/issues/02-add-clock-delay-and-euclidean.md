# 02 — Add Clock, Delay, and Euclidean Functions

**What to build:** Implement `~.`, `~*`, and `~%` from ADR 0012 using explicit absolute Tick and
Number operands.

**Blocked by:** 01 — Thread Tick and Position into interpretation; orcvs-language-migration/01; orcvs-language-migration/02; lang-foundations/06.

**Status:** ready-for-agent

- [ ] Clock returns `floor(Tick / rate) % modulus`.
- [ ] Delay Bangs exactly when `Tick % (rate * modulus) == 0`, including Tick `0`.
- [ ] Zero rate or modulus diagnoses and the cycle product cannot byte-wrap.
- [ ] Euclidean follows ADR 0012's formula and phase exactly.
- [ ] Euclidean handles zero hits, full hits, zero steps, and hits greater than steps.
- [ ] Note operands diagnose in Clock, Delay, and Euclidean tests rather than converting implicitly.
- [ ] Sequence operands follow the ordinary broadcasting rules once available.
- [ ] Tick-by-Tick tests use explicit Source Grids and diagnostics.
