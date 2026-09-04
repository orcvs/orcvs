# 04 — Add deterministic Random

**What to build:** Implement `~? seed minimum maximum` with ADR 0013's ChaCha8 seed layout and
inclusive byte-range mapping.

**Blocked by:** 01 — Thread Tick and Position into interpretation; sequence-values/02; lang-foundations/06.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Reversed bounds normalize and equal bounds return that value.
- [ ] Seed, absolute Tick, signed Position coordinates, and Sequence index occupy the specified bytes.
- [ ] A fresh ChaCha8 stream supplies the first `u64` for each scalar result.
- [ ] Golden vectors pin seed bytes, stream output, and range mapping.
- [ ] Moving a Function changes its stream; identical inputs reproduce it.
- [ ] Note seed or bound operands diagnose in Random tests rather than converting implicitly.
- [ ] Sequence index distinguishes broadcast elements.
- [ ] `rand_chacha` is added only to `lang`, with default features disabled and dependency audit.
- [ ] Native and `wasm32-unknown-unknown` results match.
- [ ] `CONTEXT.md` gains a glossary entry for the Random Function `~?`, naming its spelling, its
      seed/minimum/maximum operands, and the determinism rule that its stream is a function of seed,
      absolute Tick, Position, and Sequence index. Glossary text lands with the issue that builds the
      behaviour, as `spatial-tick-planning/01` did for `Turn`, `Producer`, and `Effect`.

## Comments

This ticket must use the Rust dependency-change workflow in addition to ordinary Rust verification.
