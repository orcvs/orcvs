# 06 — Render the Function reference from the repo

**What to build:** `/reference/` renders `docs/functions.md` at build time instead of keeping a
copy, so the test that checks it (`orcvs/tests/function_reference_doc.rs`) also checks the site.
Each Function's example has "Launch example" through the ticket 04 component. See `../spec.md`
(Reference content).

**Blocked by:** 03, 04

**Status:** ready-for-agent

- [ ] Changing `docs/functions.md` changes the built reference with no other edit.
- [ ] Site search finds a Function by its spelling.
- [ ] A smoke test launches one Function's example.
