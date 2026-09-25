# 21 — Bound the Playback diagnostics queue

**What to build:** An embedder that never drains Playback diagnostics does not grow memory without limit. Today every declined Tick reports an Overrun into an unbounded channel, so a long overloaded run with no drain accumulates one entry per Tick. Repeated invalid starts also report StartFailure, and alternating output errors bypass identical-error suppression; bounding Overruns alone is insufficient.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Record a bound on retained diagnostic memory across all classes, including message payloads. Define retention, coalescing and eviction behavior for each class, and bounded counters for omitted events, with explicit saturation behavior.
- [x] A drain after a long undrained run still reports that overruns happened and their count, or an explicit lower bound if the counter saturated.
- [x] Deterministic tests generate undrained Overruns, repeated invalid starts and alternating output failures, separately and mixed, and assert the retained memory stays bounded.
- [x] A drain preserves evidence of critical failures according to the recorded policy and reports omissions; it does not promise to retain every individual non-Overrun event indefinitely.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.

**2026-09-25 — acceptance-criteria review against `199c3331`.** An unlimited lossless non-Overrun stream conflicts with bounded memory. Extended the policy and tests to every diagnostic class. Command admission is separate work in source-audit/27.

**2026-09-25 — implementation.** Implemented in orcvs/orcvs#146 (epic PR 3): bounded `DiagnosticLog` (32 entries, 256-byte messages, class-tiered eviction, saturating omission counts, `Omitted` summary on drain); policy in ADR 0055. Resolve on merge.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#146 (`897d6814`); every criterion verified on `main`.
