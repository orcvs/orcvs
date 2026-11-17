# 01 — Remove personal names from documents

**Status:** needs-triage

**Background:** Documents in this repository name no person. About 20 accepted ADRs in `docs/adr/`
(for example "Decided by … on 2026-11-15" in their Status lines), several `docs/research/` notes
and many `.scratch/` issues and traces still name the maintainer. Accepted ADRs are append-only, so
removing a name from one changes a binding record and needs a rule for how that is allowed.

**What to do:**

1. Decide how a name leaves an accepted ADR: rewrite the Status line in place as a metadata fix
   (the date and decision stay), or leave accepted ADRs as they are and stop new ones naming anyone.
2. Remove names from `docs/research/` and `.scratch/` files, keeping dates and decisions.
3. State the rule where agents read it, in `AGENTS.md` or `docs/agents/comments.md`, so new
   documents do not add names.
4. Run `node scripts/check-requirements.ts --marker` and move every marker if any accepted ADR
   changed, since the fingerprint covers ADR text.

Find them with `grep -rIl` for the name, excluding `.git` and `target`.
