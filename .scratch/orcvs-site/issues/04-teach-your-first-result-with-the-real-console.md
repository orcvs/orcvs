# 04 — Teach "Your first result" with the real console

**What to build:** The first lesson at `/learn/` follows `docs/first-result.md`. An MDX component,
`<Console example="…" />` or `<Console source="…" />`, shows a still preview with "Launch example"
and "Open full console". Launch creates a same-origin iframe of `/play/` with the matching URL, so a
page starts only the consoles its reader opens. If the console fails to load, the lesson text stays
and the component shows a message. See `../spec.md` (The console in the site, Keyboard).

**Blocked by:** 01, 02, 03

**Status:** resolved

- [x] Playwright tests: Launch opens the console with the lesson's Source, stopped; a failed load
      leaves the lesson text and shows the message.
- [x] Keyboard, checked in a real browser and recorded in the PR: before a click, keys scroll the
      page and do not move the Cursor; after one click, arrow keys move the Cursor and the page does
      not scroll; Tab or a click outside gives keys back to the page. A failure becomes its own
      ticket with that condition as its test.
- [x] On a narrow viewport the lesson stays readable and offers "Open full console" instead of a
      cramped embed.
- [x] The lesson's steps match `docs/first-result.md` as it stands when the PR opens.

## Comments

### Shipped, 2026-10-26

`site/src/components/Console.astro` is the component and `site/src/content/docs/learn/first-result.mdx`
the lesson; `site/tests/lesson.spec.ts` checks every condition above in Chromium. Pressing
**Launch example** is the reader's click: the console takes the keyboard as it starts, so after
that one click arrow keys move the Cursor and the page does not scroll. Before it, and after a
click outside, keys scroll the page and leave the Cursor where it was. Tab does not give keys
back: the console keeps them. That failure is ticket 11, and its condition is a test in
`lesson.spec.ts` marked `test.fail()` until the ticket is done. The
console has no DOM signal for "started" except the `width` attribute eframe sets on the canvas,
so the component and the tests read that. A test compares the lesson's `text` blocks with
`docs/first-result.md`'s, so the two cannot drift apart unnoticed.
