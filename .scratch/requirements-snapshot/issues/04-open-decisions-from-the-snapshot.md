# 04 — Decisions the snapshot left open

**Status:** needs-triage

Each is listed under "Open decisions" in its requirement and is not binding there.

- Value roots and ADR 0006's "inert until Bang" (decisions D5, `docs/requirements/evaluation.md`).
  Asked of Toby on 2026-11-14.
- Accepting ADRs 0078 and 0079, which are built (D6, `docs/requirements/console.md`). Asked of
  Toby on 2026-11-14.
- Standalone operand literals: a 2026-10-16 ruling reverses ADR 0021 for Number and Note pairs and
  needs an ADR (D4, `docs/requirements/language.md`).
- ADR 0016's teardown after a refused delivery against the code's retry (D9,
  `docs/requirements/playback.md`).
- The network destination reaching the Playback Engine outside ADR 0056's request slots
  (`docs/requirements/playback.md`).
- Site details resting on undated spec text: the 16 KiB link limit and refusal forms (SITE-022),
  service workers (SITE-023), deploy cache replacement (SITE-082).
- Glossary terms the requirements use without entries: Advance, Emit, Write Refusal, Portal Error,
  Input and Output Span, Declared and Selected, Current and Retained (ADRs 0060, 0077), Optional
  Operand, and safety action. For `/domain-modeling`.
