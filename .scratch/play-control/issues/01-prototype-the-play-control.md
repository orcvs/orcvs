# 01 — Prototype the Play control

Status: needs-triage
Type: prototype
Blocked by: None — can start immediately

**What to build:**

Variants of the Play control drawn by the real console (its font, Themes and Panel), switched by
`?variant=` on the web build, captured at desktop and phone widths in the dark and light Themes, so
the user can choose placement, glyphs and whether Rewind joins it. See [`../spec.md`](../spec.md).

The prototype lives on the throwaway branch `prototype/play-control`.

**Acceptance criteria:**

- [ ] Each variant is drawn by the console itself and captured at desktop and phone widths, dark and
      light.
- [ ] The user has chosen a variant, and the choice is recorded here.
