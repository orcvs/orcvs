# 04 — Teach "Your first result" with the real console

**What to build:** The first lesson at `/learn/` follows `docs/first-result.md`. An MDX component,
`<Console example="…" />` or `<Console source="…" />`, shows a still preview with "Launch example"
and "Open full console". Launch creates a same-origin iframe of `/play/` with the matching URL, so a
page starts only the consoles its reader opens. If the console fails to load, the lesson text stays
and the component shows a message. See `../spec.md` (The console in the site, Keyboard).

**Blocked by:** 01, 02, 03

**Status:** ready-for-agent

- [ ] Playwright tests: Launch opens the console with the lesson's Source, stopped; a failed load
      leaves the lesson text and shows the message.
- [ ] Keyboard, checked in a real browser and recorded in the PR: before a click, keys scroll the
      page and do not move the Cursor; after one click, arrow keys move the Cursor and the page does
      not scroll; Tab or a click outside gives keys back to the page. A failure becomes its own
      ticket with that condition as its test.
- [ ] On a narrow viewport the lesson stays readable and offers "Open full console" instead of a
      cramped embed.
- [ ] The lesson's steps match `docs/first-result.md` as it stands when the PR opens.
