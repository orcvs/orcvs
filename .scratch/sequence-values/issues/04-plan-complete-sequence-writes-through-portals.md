# 04 — Plan complete Sequence writes through Portals

**What to build:** Route ordinary Atom and Sequence results through one Portal destination into
atomic Tick Plan Cell writes, following ADR 0009.

**Blocked by:** 01 — Add the Sequence language value;
`.scratch/language-map/issues/03-move-source-consumers-behind-the-language-map.md`.

**Status:** ready-for-agent

- [ ] A non-empty value validates its entire encoded Footprint before planning writes.
- [ ] Out-of-Grid or non-fitting output diagnoses and plans no partial write.
- [ ] Empty Sequence plans no writes.
- [ ] Current encoding never clears a stale tail from an earlier result.
- [ ] Planned Cells participate independently in ADR 0020 conflict ordering.
- [ ] Generated content is ordinary Source on the next Tick.
- [ ] Portal remains internal destination state, not a language value or persisted object.

## Comments

Concrete Source Read/Write addressing remains deferred; ordinary result Portals are implementable.
