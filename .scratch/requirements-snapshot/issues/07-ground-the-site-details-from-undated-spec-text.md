# 07 — Ground the site details that rest on undated spec text

**Status:** needs-triage

**What to decide:** three site rules are built but rest only on undated `.scratch/orcvs-site/` spec text or a ticket's implementation choice: the 16 KiB link
limit and its refusal forms (SITE-022), service workers, and replacing a cached console on deploy. `docs/requirements/site.md` marks them.

**Done when:** each is ruled, in an ADR or a dated ruling, and the requirement says which.

**Progress, 2026-11-15:** all three proposed as
[ADR 0113](../../../docs/adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md).
Waiting on Toby to accept or change it; its open questions list where the code and the spec differ.
