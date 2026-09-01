# 04 — Simplify Parser borrowing and Atom handoff

**What to build:** Parse immutable Source text and move the bounded parsed Atom storage directly to
its consumer, without requiring mutable owned Strings or rebuilding the same `ArrayVec` through
intermediate collections.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Parser construction accepts immutable Source text and stores only an immutable borrow.
- [ ] Callers no longer allocate or request mutable text solely to construct a Parser.
- [ ] Strict parsing returns the bounded Atom storage without collecting it into an equivalent
      bounded buffer again.
- [ ] Taking parsed Atoms from an Expression moves the existing buffer without reconstructing it.
- [ ] Parsing behavior, capacity diagnostics, and native/WASM callers remain unchanged.
- [ ] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.
