# 27 — Link to an example from the site

**What to build:** A link to the site that names an example opens it, so a reader can send
someone `orcvs.com/?example=orca-tracker`. `/` frames `/play/` with its own query and fragment
(ADR 0094).

Toby's decision (2026-10-29): `orcvs.com/?example=<link-name>`, over a page per example and over
the console writing the link into the address bar.

**Acceptance:**

- `/?example=orca-tracker` frames `/play/?example=orca-tracker`, the console starts, and no
  service worker is registered (`site/tests/smoke.spec.ts`).
- `/#source=<text>` frames `/play/#source=<text>` the same way.
- Plain `/` still frames `/play/`.

**Status:** resolved
