# 07 — Ground the site details that rest on undated spec text

**Status:** resolved

**What to decide:** three site rules are built but rest only on undated `.scratch/orcvs-site/` spec text or a ticket's implementation choice: the 16 KiB link
limit and its refusal forms (SITE-022), service workers, and replacing a cached console on deploy. `docs/requirements/site.md` marks them.

**Done when:** each is ruled, in an ADR or a dated ruling, and the requirement says which.

## Comments

2026-11-15: all three proposed as
[ADR 0113](../../../docs/adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md).
Waiting to be accepted or changed; its open questions list where the code and the spec differ.
Ticket 05's preview-URL item is still ready-for-human and outside ADR 0113.

2026-11-16: accepted [ADR 0113](../../../docs/adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md), deciding that the service worker follows `opening`'s reading of a link and that every `/play/` file is network-first. SITE-020, SITE-022, SITE-082 and SITE-083 state it; the two gaps are [orcvs-site/31](../../orcvs-site/issues/31-register-the-service-worker-by-the-consoles-own-reading-of-a-link.md) and [orcvs-site/32](../../orcvs-site/issues/32-answer-every-play-file-from-the-network-first.md).
