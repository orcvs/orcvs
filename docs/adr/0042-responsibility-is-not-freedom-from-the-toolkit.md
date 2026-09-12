# Responsibility is not freedom from the toolkit

Status: accepted. Refines [ADR 0022](0022-keep-the-core-crate-free-of-the-ui-toolkit.md). The dependency
rule ADR 0022 states stands unchanged — including "no module moves into `orcvs` until it is free of
[`egui`, `eframe`, and `winit`]" — and this decision does not reverse it. It records the half ADR
0022 left unstated: freedom from those crates is necessary for a module to live in `orcvs`, and it
is not sufficient.

A fact about a Source belongs to `orcvs`. A decision about how this console chooses to show a Source
belongs to `console`, whether or not it can be spelled in `u8`. The vocabulary already names both
sides — Source, Grid, Cell, Glyph, Render Frame, Paint — and this decision does not invent a new
pair of words for them.

## Why the half that was written was not enough

ADR 0022 itself says the older separations were conventions one crate could not enforce and that the
compiler enforces the toolkit cut. That is true of the dependency half and of nothing else. Every
borderline case that compiled therefore resolved the same way: into `orcvs`.

Three shipped examples are the evidence, not taste:

- `SECTOR_SEAM_STRENGTHS: [u8; 4] = [100, 72, 34, 13]` in `orcvs/src/render_frame.rs` is an array of
  alpha percentages in the core crate, consumed only by `console/src/style.rs`, whose body is
  `base_alpha * strength / 100`.
- `CursorBloom` is a public enum of the core crate naming four bands of a glow, with no `CONTEXT.md`
  entry and no ADR behind it.
- `DEFAULT_FONT_SIZE` in `orcvs/src/opts.rs` is a font size in the core crate that only `console`
  reads.

Each of those is free of `egui`, so each of those landed. The drift was one-directional because the
compiler enforces the dependency half and nothing enforces the responsibility half. What enforces
the new half is a reader, at review — this ADR does not pretend otherwise.

## What the criterion does and does not decide

`Glyph` stays in `orcvs`. Which Language Unit a Cell belongs to is a fact about the Source that
`CONTEXT.md` already defines, even though its only consumer paints with it. An ADR that cannot say
which side that one falls on is useless as a criterion.

[ADR 0040](0040-the-console-paints-from-a-value.md) is the decision that gave the console a value
layer — Paint — able to receive presentation that moves out of `orcvs`. This ADR cites that layer; it
does not restate how Paint is derived or how Shapes are built, and it does not edit ADR 0040.

## Rejected alternative

Leave ADR 0022 as the whole rule and treat each drift as a one-off review comment. Refused because
three independent reviews found the same class of drift, which makes it a criterion fault rather than
a sequence of lapses.

## Numbering

This file takes 0042: [ADR 0040](0040-the-console-paints-from-a-value.md) and
[ADR 0041](0041-the-playback-engine-owns-its-state-in-one-task.md) already hold the next two numbers
on `main`. The ticket that proposed this decision named 0041 before playback took that number; the
README's rule — take the next number above the highest file — yields 0042 here. `0027` is not taken.
