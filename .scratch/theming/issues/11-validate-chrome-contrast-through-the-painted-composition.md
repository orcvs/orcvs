# 11 — Validate chrome contrast through the painted composition

**What to build:** The contrast validator measures console chrome the way egui paints it, using the same composition the chrome's painting uses, and covers every chrome text-on-surface state a Theme can produce. `03` required that "painting and contrast validation must use the same composited backgrounds" and deferred it to `08`; `08` resolved having covered the Source only.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Chrome contrast states are composited by a function the chrome's painting also uses, not by the hand-written window → panel → input layering at `console/src/contrast.rs:539-548`. The comment there, which says chrome has no reused composition "yet" because `03` has not landed, goes.
- [ ] The report covers text on `selection.background`, `text.active` on the hovered, active and open widget fills, and `error`, `warning` and `link` on the panel, in addition to the existing `text` and `text.muted` on panel and input. Any state left out is named in the report's scope, with its reason.
- [ ] `shipped_theme_gate` holds Okabe–Ito and Orcvs Light to the same floor over the added states, or each failure is a pair-keyed accepted exception, recorded in `console/src/theme.md`.
- [ ] Tests include a Theme built to fail on a chrome state the old report did not measure.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** It owns `03`'s chrome half of the composited-contrast line. `07` is blocked by this issue: `07` shows the validator's report when a Theme loads, so the report should cover chrome first. `theming-04-switching` exposes Orcvs Light before this issue lands, so the light Theme's chrome states go unmeasured until it does.
