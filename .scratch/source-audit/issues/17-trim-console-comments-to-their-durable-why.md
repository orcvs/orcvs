# 17 — Correct console comments the source-audit changes leave behind

**What to build:** After source-comments/04 holds `console` to `docs/agents/comments.md` (lineage, provenance citations — including the 150 `.scratch/` citations — and the gate carve-outs), what remains is the work that rule does not cover and that depends on the code the console source-audit tickets change: upstream citations that name a file line rather than the behaviour relied on (30 `file.rs:NN` citations in `.rs` files, 21 of them in `console.rs`; the 8 in `console/Cargo.toml` are source-comments/05's), and explanations repeated within a single function. About 24% of console `.rs` lines are comments.

**Blocked by:** source-comments/04; 11 — Decide the fate of the colour-blindness simulation; 13 — Split Console::ui into panel modules; 14 — Drive the panel layout test through Console::ui; 18 — Make TOML the only Theme representation; 22 — Keep an earlier refused Source when a later start also refuses.

**Status:** ready-for-agent

- [ ] Upstream citations name the behaviour relied on, not a file line.
- [ ] Each argument appears once, at the item that enforces it.
- [ ] Rendering-budget and invariant explanations are kept.
- [ ] Comments that contradict the code after the tickets above land are corrected or removed.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The lineage and citation criteria duplicated source-comments/04, and the manifest criterion duplicated source-comments/05 (both filed by 0ce8d2c4). This ticket keeps what those do not cover. The earlier "about 30%" did not reproduce; about 24% then and now.
