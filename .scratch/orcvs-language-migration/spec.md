# Migrate the Orcvs implementation onto the audited vocabulary

**Goal:** Bring `lang/` into line with the vocabulary and semantics that `CONTEXT.md` and
`docs/adr/0006`–`0021` now define, so the documented sources of truth and the shipped parser
agree.

## Why

The Orca operator audit settled the Orcvs Function vocabulary in `CONTEXT.md` and ADRs 0006–0021,
and ADR 0019 is the authoritative capability index. The implementation in `lang/` still carries the
pre-audit vocabulary. `AGENTS.md` names `CONTEXT.md` and `docs/adr/` as the sources of truth for
vocabulary, and `docs/agents/domain.md` requires contradictions to be surfaced rather than left
implicit — these issues are that surfacing.

Two of the divergences are collisions rather than renames: `**` and `>>` still parse, as the wrong
thing, so they fail silently rather than diagnosing.

## Issues

- `issues/01-move-arithmetic-onto-the-dot-family.md`
- `issues/02-free-the-bang-and-activation-spellings.md`
- `issues/03-wrap-general-arithmetic-over-bytes.md`
- `issues/04-select-a-disjoint-note-encoding.md`
- `issues/05-add-explicit-numeric-conversions.md`
- `issues/06-retire-the-identity-test-function.md`
- `issues/07-complete-the-numeric-function-family.md`
