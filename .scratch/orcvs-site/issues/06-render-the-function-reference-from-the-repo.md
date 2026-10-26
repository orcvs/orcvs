# 06 — Render the Function reference from the repo

**What to build:** `/reference/` renders `docs/functions.md` at build time instead of keeping a
copy, so the test that checks it (`orcvs/tests/function_reference_doc.rs`) also checks the site.
Each Function's example has "Launch example" through the ticket 04 component. See `../spec.md`
(Reference content).

**Blocked by:** 03, 04

**Status:** resolved

- [x] Changing `docs/functions.md` changes the built reference with no other edit.
- [ ] Site search finds a Function by its spelling. Met for names and for spellings with a
      letter, such as `.v`; a spelling of symbols alone moved to ticket 10.
- [x] A smoke test launches one Function's example.

## Comments

### Shipped, 2026-10-26

`site/src/pages/reference/index.astro` imports `docs/functions.md` and frames it in a Starlight
page; nothing is copied. `site/src/console/function-reference.ts` is a Sätteri mdast plugin, the
Markdown processor Astro 7 uses, that runs on that file alone: it drops the file's `#` heading
(Starlight renders the page title), points `first-result.md` at `/learn/first-result/` and other
relative links at the repository, and adds "Launch example" after every `orcvs` block with no
`tick`, the starting Source `orcvs/tests/function_reference_doc.rs` runs. Using the plugin API
needs `@astrojs/markdown-satteri` as a direct dependency, pinned to the 0.4.5 Astro already
installs. `site/tests/reference.spec.ts` checks the page's headings and launch links against the
file, launches `.+` Add, and searches. The component's markup, launch script and styles moved to
`site/src/console/` so the lesson and the reference share them.

Pagefind indexes words, so `.+` finds nothing; ticket 10 holds that.
