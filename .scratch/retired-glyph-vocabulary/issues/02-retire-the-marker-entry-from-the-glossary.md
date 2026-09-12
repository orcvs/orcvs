# 02 — Retire the Marker entry from the glossary

**What to fix:** `CONTEXT.md:263-265` defines a **Marker** the console does not draw and has not
drawn since the sector seams replaced it.

**Blocked by:** 01 — Name the Sector Seam and the Cursor Bloom in the glossary.

**Status:** ready-for-agent

- [ ] The **Marker** entry at `CONTEXT.md:263-265` is gone from the glossary.
- [ ] The `_Avoid_` list that was on it — "Guide, gridline, ruler dot" — is not silently lost.
      Whichever of those terms still threaten the **Sector Seam** entry `01` writes belong on that
      entry's `_Avoid_` line instead.
- [ ] Nothing in the glossary claims a purely visual Glyph exists.
- [ ] No other glossary entry is reworded, in particular not **Paint** (`CONTEXT.md:271-273`) or
      **Cursor** (`CONTEXT.md:255-257`).
- [ ] **Conditional, and only if `typed-source-paint/01` has not landed:** the **Glyph** entry at
      `CONTEXT.md:259-261` no longer names Marker or Highlight — it says a Cell the Source has not
      parsed carries Space, and nothing else. If `typed-source-paint/01` has landed, that entry is
      already gone and this line is satisfied by its absence. Either way the glossary does not
      finish this ticket still naming two Glyphs that cannot occur.
- [ ] `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` pass.

## Comments

The entry reads, in full:

```text
**Marker**:
A purely visual Glyph the console draws at every marker-spacing interval of Cells in both axes, so
distance across the Source can be read by eye. A Marker carries no content and belongs to no
Expression; it appears only on a Cell the Source gives no Glyph of its own.
_Avoid_: Guide, gridline, ruler dot
```

Every clause of it is now false except the last. Nothing draws it: a Cell's `Glyph` comes from
`orcvs/src/render_frame.rs:92-95`, which reads `language_map().glyph_at(position)` and falls back to
`Glyph::Space`; a Language Map assigns `Glyph::from(entry.token)`
(`orcvs/src/source/language_map.rs:388`) or backfills `Glyph::Char`
(`orcvs/src/source/language_map.rs:404`), and `From<Token>` (`orcvs/src/glyph.rs:53-73`) has no
`Marker` arm. The behaviour the entry describes — a mark at every interval of Cells in both axes, so
distance can be read by eye — is exactly what the sector seams do now, in geometry rather than in
Glyphs. `console/src/theme.md:41-42` says so: they "replace the historical `+` Marker Glyphs,
leaving every empty Cell visually empty while preserving the configured Marker spacing as geometry."

This is why the ticket is blocked by `01` and not merely sequenced after it. Deleting the entry
first leaves the glossary with no name for the thing that took the job over, and
`docs/agents/domain.md` would then have nothing to bind a later issue title to. Define the survivor,
then retire the corpse.

**Why this ticket is narrower than it looks, and what owns the rest.** `typed-source-paint/01` —
`ready-for-agent`, no unmet blockers — has an acceptance line reading "`CONTEXT.md`'s **Glyph**
entry is removed and no term replaces it." It says nothing about the **Marker** entry: a grep across
that entire effort finds the word only in its spec's Token/Glyph comparison table and in one passing
clause of its `02`. So the **Marker** entry is genuinely unowned, and it survives either outcome —
if `Glyph` becomes `Option<Token>`, a glossary entry defining a Glyph variant is more orphaned, not
less. That is what makes this ticket safe to take now, and why the **Glyph** entry is a conditional
line rather than an unconditional one.

**There is no separate Highlight entry to delete.** The term appears in the glossary only inside the
**Glyph** entry's sentence at `CONTEXT.md:260` — "…and Marker, Highlight or Space for a Cell it has
not" — which is the clause the conditional line covers.

**Status is `ready-for-agent`** because the wording is fully determined once `01` settles: one
deletion, one `_Avoid_` line rehomed, and one conditional clause edit whose condition an agent can
check by reading the file.
