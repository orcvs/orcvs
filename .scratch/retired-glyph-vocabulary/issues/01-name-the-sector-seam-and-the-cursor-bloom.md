# 01 — Name the Sector Seam and the Cursor Bloom in the glossary

**What to build:** Two new `CONTEXT.md` entries, **Sector Seam** and **Cursor Bloom**, for the two
console concepts that are shipped, tested, publicly typed, and undefined.

**Blocked by:** None — can start immediately, but read the trap below before writing a word.

**Status:** resolved

- [x] `CONTEXT.md` defines **Sector Seam**: the graded registration mark the console draws along
      every sector boundary of the Source Grid, at the configured interval in both axes. It is
      geometry drawn over Cell edges and carries no content, occupies no Cell, and belongs to no
      Expression. Its `_Avoid_` list names the terms it replaces and the ones it must not drift to.
- [x] `CONTEXT.md` defines **Cursor Bloom**: the graded field of four bands the console draws
      outward from the Cursor's Cell, measured in Cells by Chebyshev distance, decided per Cell and
      carried on the Render Frame. It changes a Cell's background and border and never its content
      or its Glyph. Its `_Avoid_` list likewise names what it is not.
- [x] Both entries follow the file's existing shape exactly: `**Term**:` on its own line, the
      definition as prose beneath it, then an `_Avoid_:` line. They sit in the console section
      beside **Cursor**, **Glyph**, **Render Frame** and **Paint**.
- [x] Neither definition states a default value. `8` and `7` are `Opts` defaults
      (`orcvs/src/opts.rs:4`, `:6`), not properties of the concepts, and the glossary does not
      record `Opts` values anywhere else.
- [x] The **Paint** entry at `CONTEXT.md:271-273` is not reworded. It already uses "sector seams";
      this ticket only gives that term a definition to point at.
- [x] `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` pass.

## Comments

**The gap is real and was checked term by term.** `CONTEXT.md` contains no occurrence of "bloom",
in any case, anywhere in the file. "Seam" occurs twice: once at `CONTEXT.md:248` about the ADR 0016
ownership seam, which is a different sense entirely, and once at `CONTEXT.md:272` inside the
**Paint** entry, which lists "sector seams" among what a Paint decides. So the glossary already uses
one of these two terms without defining it, and does not contain the other at all. `docs/adr/` holds
no record of either: no ADR file mentions a bloom, and no ADR mentions a sector seam.

**Both concepts are shipped, not proposed.** `CursorBloom` is a public enum with four variants at
`orcvs/src/render_frame.rs:14-20`, carried on every `RenderCell`
(`orcvs/src/render_frame.rs:27`, `:51-53`) and read by `console/src/style.rs:88-99` and
`:106-113`. The seams are two `Option<u8>` strengths per Cell (`orcvs/src/render_frame.rs:28-29`,
`:55-61`) computed by `sector_seam_strength` (`orcvs/src/render_frame.rs:179-194`) from
`SECTOR_SEAM_STRENGTHS = [100, 72, 34, 13]` (`orcvs/src/render_frame.rs:177`), and attenuated into a
colour by `console/src/style.rs:115-119`. `console/src/theme.md:26-42` describes the visual
behaviour of both at length and is the best existing source for the wording — but it is a palette
record, not the glossary, and `CLAUDE.md` names `CONTEXT.md` the source of truth for vocabulary.

**A second effort touches this and does not cover it.** `typed-source-paint/01` deletes the `Glyph`
vocabulary wholesale, and one of its acceptance lines reads "`CONTEXT.md`'s **Glyph** entry is
removed and no term replaces it. The Grid's background rulings — the sector seams and the Cursor
bloom — are described where they are drawn." Described where they are drawn is `console/src/theme.md`
and the `render_frame.rs` comments, both of which already describe them well. Neither is the
glossary, and `docs/agents/domain.md` binds issue titles, type names and test names to the glossary
specifically. So that line does not satisfy this ticket, and this ticket does not conflict with it.

**Why this is `needs-triage` rather than `ready-for-agent`.** Two reasons, and either alone is
enough. First, a glossary entry is a naming decision, and `docs/agents/domain.md` binds every later
issue title, type name and test name to whatever words land here — `05` renames two public fields to
match. Second, ownership is contested: see the trap.

**Trap — a concurrent effort may claim these same two entries.**
`.scratch/render-frame-responsibility/` is being filed right now by another agent, against the code
that computes both concepts. This effort has not read that directory and must not write to it.
Before writing these entries, read `.scratch/render-frame-responsibility/` and check whether it
already owns them. If it does, drop this ticket to a reference — the entries are written once, in
one change — and leave `02` and `05` blocked on that effort's ticket instead. This is how
`source-paint/01` handled the **Console** entry that `console-testing/02` already owned: it blocked
on the other effort rather than duplicating the definition.

**Resolved 2026-09-13.** Trap checked: `render-frame-responsibility/01` and `04` assign both
entries to this ticket and refuse to edit `CONTEXT.md` themselves. Entries land beside **Cursor**
and **Marker**; **Paint** untouched. **Sector Seam** `_Avoid_` takes Marker plus the Guide /
gridline / ruler-dot list ticket `02` must not lose when it deletes **Marker**. **Cursor Bloom**
`_Avoid_` names Highlight and the glow / radial-light / focus-matrix drift `theme.md` already
contrasts against.
