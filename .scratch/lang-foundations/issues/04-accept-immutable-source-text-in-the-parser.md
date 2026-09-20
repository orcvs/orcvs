# 04 — Accept immutable Source text in the Parser

**What to build:** Let parser construction take Source text the caller does not own mutably, so that nothing allocates a string, clones one, or iterates mutably purely to satisfy a signature.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** Improvement

One of the two parser constructors demands mutable Source text and then reborrows it immutably; the parser never writes through it. The other constructor already takes immutable text and is what the one production caller uses. The mutable constructor's cost is paid almost entirely by tests, property tests, and benchmarks, which manufacture owned strings to feed it.

- [ ] Both parser constructors accept immutable Source text.
- [ ] No caller, test, property test, or benchmark allocates, clones, or mutably iterates Source text solely to construct a Parser.
- [ ] Parsing behavior, capacity diagnostics, and native and WASM callers are unchanged.
- [ ] No benchmark is owed. The change moves a signature, not generated code; record that as the reason on the `Not run` line rather than running the series.

## Comments

Rewritten after auditing this ticket against the crate. Two acceptance criteria were dropped because they describe a structure the crate no longer has.

"Strict parsing returns the bounded Atom storage without collecting it into an equivalent bounded buffer again" presumed a bounded Atom buffer. Parsed Atoms are a heap vector, not a bounded one, and strict parsing already returns what the Expression hands it. There is no re-collect on that path; the only identity round-trip left is in a test helper and belongs to whoever next touches it.

"Taking parsed Atoms from an Expression moves the existing buffer without reconstructing it" presumed an Atom buffer sitting inside the Expression that a move could claim. Issue 07 paired each entry's syntax with its value, so Atoms are now interleaved fields inside positioned records and the handoff must collect. Satisfying that criterion would mean undoing 07.

Both came from an audit of the crate at `c889af5`, several hundred commits back, and were invalidated by issues 07 and 08 — which the spec's Delivery section expected to *follow* this ticket, and which shipped ahead of it instead.

The audit also found two things outside this ticket's scope. The per-call duplication of an Expression's values on the Source read path is issue 11. The forced-inlining question the spec scopes out is untouched, but worth noting concretely: the whole main parse loop carries `inline(always)` while the innermost per-token push carries no hint at all. That needs the benchmark evidence the spec asks for, and this ticket does not supply it.
