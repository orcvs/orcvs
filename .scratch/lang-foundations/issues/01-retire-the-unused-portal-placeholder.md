# 01 — Retire the unused Portal placeholder

**What to build:** Remove the unused Portal module and public export so the current crate does not
present an unimplemented destination abstraction as working language infrastructure. Future Portal
state will be introduced by the complete Sequence-write slice when its Tick Plan requirements are
known.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** Improvement

- [x] The unused Portal type, module, and export are removed.
- [x] No current parser, evaluator, or Source behavior changes.
- [x] The later Sequence Portal ticket still describes the domain behavior without depending on the
      deleted placeholder's shape.
- [x] Native and WASM builds remain green.

## Comments

### Resolution, 2026-09-05

Closed alongside `sequence-values/04`, which is the slice that knows what Portal state a Tick Plan
actually needs. `lang/src/portal.rs` and the `pub use portal::Portal` in `lang/src/lib.rs` are gone;
nothing in the workspace referenced either. The type was dead — it carried an `Atom` and a bare
coordinate pair in the language crate, which is the shape ADR 0009 and CONTEXT.md rule out for a
Portal — so removing it changed no parser, evaluator, or Source behaviour. The real Portal now lives
in `orcvs/src/source/portal.rs` as internal destination state, and owes nothing to the placeholder's
shape.
