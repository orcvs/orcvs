# 04 — Add deterministic Random

**What to build:** Implement `~? seed minimum maximum` with ADR 0013's ChaCha8 seed layout and
inclusive byte-range mapping.

**Blocked by:** 01 — Thread Tick and Position into interpretation;
`.scratch/sequence-values/issues/02-broadcast-atomic-functions-over-sequences.md`.

**Status:** ready-for-agent

- [ ] Reversed bounds normalize and equal bounds return that value.
- [ ] Seed, absolute Tick, signed Position coordinates, and Sequence index occupy the specified bytes.
- [ ] A fresh ChaCha8 stream supplies the first `u64` for each scalar result.
- [ ] Golden vectors pin seed bytes, stream output, and range mapping.
- [ ] Moving a Function changes its stream; identical inputs reproduce it.
- [ ] Note seed or bound operands diagnose in Random tests rather than converting implicitly.
- [ ] Sequence index distinguishes broadcast elements.
- [ ] `rand_chacha` is added only to `lang`, with default features disabled and dependency audit.
- [ ] Native and `wasm32-unknown-unknown` results match.

## Comments

This ticket must use the Rust dependency-change workflow in addition to ordinary Rust verification.
