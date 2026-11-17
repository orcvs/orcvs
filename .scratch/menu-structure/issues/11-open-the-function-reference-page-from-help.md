# 11 — Open the Function reference page from Help

**What to build:** Help → Function Reference opens the site's `/reference/` page, in a new tab on
the web and in the system browser on native, as Help → Learn does, and replaces the separate Help →
Reference item. The runnable Grid that `console/assets/function_reference.orcvs` holds moves under
Help → Examples as `Functions`, and asks before discarding as every example does.
[ADR 0111](../../../docs/adr/0111-the-native-console-edits-one-source-file-and-autosave-only-recovers.md) decides it (CONS-133). Today Function Reference replaces the running Source with
that Grid, unasked on the web, beside a Reference item that opens the page.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] Help lists Learn, Function Reference (the page), Examples and About; no item replaces the
      Source unasked, on native or the web.
- [ ] `Functions` is a shipped example, with a link name (SITE-024), opened and asked about like
      the others.
- [ ] The Function reference Source keeps its checked-in text and its test against the Function
      table.
- [ ] `docs/requirements/console.md` no longer lists CONS-133 as built differently.

## Comments

- 2026-11-18: The web case is the one readers meet first: on the site, Help → Function Reference
  replaces the reader's Grid without asking, where native asks. `docs/first-result.md`'s Next
  steps describes Help → Function Reference as replacing what is on the Grid; when this lands,
  point that line at the page and at Help → Examples → `Functions` instead.
