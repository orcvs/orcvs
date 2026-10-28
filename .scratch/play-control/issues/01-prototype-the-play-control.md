# 01 — Prototype the Play control

Status: resolved
Type: prototype
Blocked by: None — can start immediately

**What to build:**

Variants of the Play control drawn by the real console (its font, Themes and Panel), switched by
`?variant=` on the web build, captured at desktop and phone widths in the dark and light Themes, so
the user can choose placement, glyphs and whether Rewind joins it. See [`../spec.md`](../spec.md).

The prototype lives on the throwaway branch `prototype/play-control`.

**Acceptance criteria:**

- [x] Each variant is drawn by the console itself and captured at desktop and phone widths, dark and
      light.
- [x] The user has chosen a variant, and the choice is recorded here.

## Comments

2026-10-27: Built as `?variant=A`–`D` on `prototype/play-control`, with captures in its
`.scratch/play-control/prototype/`. All four fit at 390 px; an emulated touch tap on A plays.
Toby chose **A with ⏮ Rewind**: Rewind and Play/Stop glyphs at the start of the Panel (ADR 0089).
Seeing them, he found them too small and chose the Cells `P` and `R` instead, one Source Cell each
and one Cell apart (`prototype/play-control-cells`, variant E).
