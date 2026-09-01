# 01 — Retire the unused Portal placeholder

**What to build:** Remove the unused Portal module and public export so the current crate does not
present an unimplemented destination abstraction as working language infrastructure. Future Portal
state will be introduced by the complete Sequence-write slice when its Tick Plan requirements are
known.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The unused Portal type, module, and export are removed.
- [ ] No current parser, evaluator, or Source behavior changes.
- [ ] The later Sequence Portal ticket still describes the domain behavior without depending on the
      deleted placeholder's shape.
- [ ] Native and WASM builds remain green.
