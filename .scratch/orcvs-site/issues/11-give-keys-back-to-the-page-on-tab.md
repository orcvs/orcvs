# 11 — Give keys back to the page on Tab

**What to build:** In an embedded console, Tab moves focus out of the console so keys reach the
page again, as the spec's keyboard condition asks: "Tab or a click outside gives keys back to the
page." Today a click outside does, and Tab does not: after Tab, arrow keys still move the Cursor.
See `../spec.md` (Keyboard). The change is in the console's input handling, so it follows the
`egui` skill.

**Blocked by:** 04

**Status:** needs-triage

- [ ] `site/tests/lesson.spec.ts`'s "Tab in the console gives keys back to the page" passes with its
      `test.fail()` removed.
- [ ] Tab inside the full console at `/play/` keeps whatever it does there today, unless the triage
      decides otherwise.
