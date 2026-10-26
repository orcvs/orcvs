# 10 — Find a Function by a spelling made only of symbols

**What to build:** Site search finds a Function whose spelling has no letter or digit, such as
`.+`, `.|` or `!>`, by typing that spelling. Pagefind indexes words and
drops symbols, so today `.v` finds Convert to Number but `.+` finds nothing. Either the reference
gives Pagefind a searchable term per spelling and the search box maps a typed spelling to it, or
the reference page gains its own filter by spelling. See `../spec.md` (Reference content) and
ticket 06.

**Blocked by:** 06

**Status:** needs-triage

- [ ] Typing `.+` in the site's search box offers `/reference/#-add`, and every other spelling in
      `docs/functions.md` likewise offers its entry.
