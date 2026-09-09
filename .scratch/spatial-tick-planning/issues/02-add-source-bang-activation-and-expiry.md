# 02 — Add Source Bang activation and expiry

**What to verify:** Close the original Bang-lifecycle issue against `CONTEXT.md` and ADR 0032. Implementation is tracked in `tick-execution-order/02`; do not implement the old cleanup-turn design independently.

**Blocked by:** None — tick-execution-order/03 is resolved.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Current-Tick Bang output activates aligned cardinal Expression roots with settled operands.
- [ ] Horizontal root anchors are two columns away; vertical anchors are one row away.
- [ ] Dependencies place a Bang producer before its consumer regardless of Position; each root executes at most once.
- [ ] Prior Bang display is discarded without activation; manually entered `**` is a no-op.
- [ ] A current result remains visible until the next Tick without becoming another activation.
- [ ] Direct activation does not overwrite the target Function.
- [ ] Partial or non-root contact does not activate.
- [ ] Parsing owns contextual Bang validity; rejected typed operands neither activate nor receive display cleanup.

## Comments

2026-09-06: The accepted dependency-ordering design supersedes this ticket's original snapshot-activation and cleanup-turn requirements. See ADR 0032 and `tick-execution-order/01`–`03`. This issue remains an integration acceptance record, not a second implementation workstream.

2026-09-06: ADR 0032's fixed-Portal scheduler now provides current-Tick dependency ordering, transient display, manual no-op, parser-owned validity, and once-only execution. This ticket remains open: current Source grammar cannot author a horizontally adjacent standalone Bang without merging it into the target Expression, and the Directional/Self-Banging Function surface that makes horizontal contact useful is still follow-up work.
