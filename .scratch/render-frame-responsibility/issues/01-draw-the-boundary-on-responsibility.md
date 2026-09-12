# 01 — Record where the model/presentation boundary sits

**What to build:** An accepted ADR that states the criterion for what may live in `orcvs`, and
refines ADR 0022's dependency rule rather than reversing it.

ADR 0022 says `orcvs` must not depend on `egui`, `eframe` or `winit`. That is true, it stays true,
and this ADR does not touch it. What this ADR adds is the condition ADR 0022 left unstated: **not
depending on the toolkit is necessary for a module to live in `orcvs`, and it is not sufficient.**
The question a module has to answer as well is whose decision it is. A fact about a Source belongs
to `orcvs`. A decision about how this console chooses to show a Source belongs to `console`, whether
or not it can be spelled in `u8`.

The reason to write it down is that the missing half was not merely unstated — it was actively
misleading, because the compiler enforces the half that was written and nothing enforces the half
that was not. ADR 0022 says so itself:

> Both separations were conventions that one crate could not enforce; the compiler enforces this
> one.

So every drift went the same way. `SECTOR_SEAM_STRENGTHS: [u8; 4] = [100, 72, 34, 13]`
(`orcvs/src/render_frame.rs:177`) is an array of alpha percentages in the core crate, consumed only
by `console/src/style.rs:115-119`, whose entire body is `base_alpha * strength / 100`. `CursorBloom`
(`orcvs/src/render_frame.rs:15`) is a public enum of the core crate naming four bands of a glow, with
no `CONTEXT.md` entry and no ADR behind it. `DEFAULT_FONT_SIZE` (`orcvs/src/opts.rs:3`) is a font
size in the core crate that only `console` reads. Each of those compiled, so each of those landed.

**The ADR takes number 0042.**

When this ticket was written, the highest file in `docs/adr/` was
`0039-pulse-functions-refuse-a-sequence-operand.md`, so the README's rule yielded 0040, and
`source-paint/01` claimed 0040. Paint landed as ADR 0040; playback then took ADR 0041. This ticket
therefore takes 0042 — the next number above the highest file — rather than the 0041 the draft named.

**Do not take 0027.** `docs/adr/README.md` names it: "A gap is not a free number: `0027` is missing
here because a branch that has not landed claims it, and reusing it would land the collision this
directory has already had once."

## How this relates to `source-paint/01`, and why it does not supersede it

Both tickets write an ADR about the console's rendering, so the relationship has to be stated rather
than left for a reader to work out.

`source-paint/01` records a decision about the **inside of one crate**: the console derives a
per-Cell paint description as a value carrying no geometry, and a separate step converts it to
Shapes. Its rejected alternative is publishing `RenderFrame::derive` so a test can build a Render
Frame directly. None of that is about where the crate seam sits.

This ADR records a decision about the **seam between two crates**: which of them owns a presentation
decision, and on what criterion. Its subject is ADR 0022, not ADR 0038.

**`source-paint/01` is not superseded and is not to be edited by this ticket.** Folding the two into
one file was considered and refused twice over. It would re-open a ticket that is `ready-for-agent`
and specified down to its checkboxes, for no gain; and it would put a decision about the internals of
`console` and a decision about the boundary between `orcvs` and `console` in one ADR, which is the
conflation this whole effort exists to undo. One decision per file is the directory's own rule.

So: **0040 is `source-paint`'s, 0041 is playback's, 0042 is this one's**, and 0042 cites 0040 as the
decision that established the value layer the aesthetic can now move into.

**Blocked by:** source-paint/01 — resolved (ADR 0040 on `main`).

**Status:** resolved

- [x] `docs/adr/0042-responsibility-is-not-freedom-from-the-toolkit.md` exists, status accepted, in
      the house format: a `Status:` line that says what it revises and what it does not, then prose.
- [x] It states the criterion: freedom from `egui`, `eframe` and `winit` is necessary for a module to
      live in `orcvs` and is not sufficient; the further condition is that the module decides a fact
      about a Source rather than a fact about how a Source is shown.
- [x] It cites [ADR 0022](0022-keep-the-core-crate-free-of-the-ui-toolkit.md) and is explicit that it
      **refines** rather than reverses it: the dependency rule stands unchanged, including "no module
      moves into `orcvs` until it is free of them".
- [x] It records the evidence that the criterion was needed, with at least the seam strengths, the
      undocumented `CursorBloom`, and `DEFAULT_FONT_SIZE` — because an ADR asserting a criterion
      without the drift that produced it reads as taste.
- [x] It records why the drift was one-directional: the compiler enforces the dependency half and
      nothing enforces the responsibility half, so every borderline case resolved the same way. It
      states what does enforce the new half — a reader, at review — and does not pretend otherwise.
- [x] It cites [ADR 0040](0040-the-console-paints-from-a-value.md) as the decision that gave the
      console a value layer to receive what moves, and does not restate it.
- [x] It names the boundary in the vocabulary `CONTEXT.md` already uses — Source, Grid, Cell, Glyph,
      Render Frame, Paint — rather than inventing a new pair of words for the two sides.
- [x] It records the alternative that was refused: leaving ADR 0022 as the whole rule and treating
      each drift as a one-off review comment. Refused because three independent reviews found the
      same class of drift, which makes it a criterion fault rather than a sequence of lapses.
- [x] It states what the criterion does **not** decide: `Glyph` stays in `orcvs`, because which
      Language Unit a Cell belongs to is a fact about the Source that `CONTEXT.md` already defines,
      even though its only consumer paints with it. The ADR is useless if it cannot say which side
      that one falls on.
- [x] No ADR is renumbered, `0027` is not taken, and `.scratch/adr-numbering/` is not touched.
- [x] `CONTEXT.md` is not edited by this ticket, and neither is it edited by any other ticket in
      this effort. `retired-glyph-vocabulary/01` owns the **Sector Seam** and **Cursor Bloom**
      entries; ticket `04` blocks on it rather than duplicating them.

## Verification

Documentation only. `node --test scripts/tests/roadmap.test.ts` and
`node scripts/roadmap.ts > /dev/null` for the `.scratch/` change.

## Comments

`needs-triage` rather than `ready-for-agent`, deliberately. Every other ticket in this effort is a
code change an agent can carry out against a bar; this one proposes revising the criterion of an
accepted ADR, and that is a maintainer's decision. It is also the premise the rest of the effort
rests on: if the criterion is refused, tickets `03` and `04` go with it, and `02`, `05`, `06`, `07`
survive on their own merits because each is a defect independent of where the seam is drawn.

> *This was generated by AI during triage.*

**Triage 2026-09-12 — accepted → `ready-for-agent`.**

The responsibility criterion is accepted as written. ADR 0042 (draft had said 0041; playback took
that number) refines ADR 0022; it does not reverse the dependency rule.

**Spacing settlement for ticket `04`:** option (1). After the bloom and seam code leave `orcvs`,
`RenderFrame` answers the seam period and the bloom radius (under whatever names
`retired-glyph-vocabulary/05` has left them). Presentation fields of `Opts` stay in `orcvs` for
this effort; moving them into `console` remains the separate `Opts` drift named in the spec's out of
scope, not part of `04`.

Landed as `docs/adr/0042-responsibility-is-not-freedom-from-the-toolkit.md`.
