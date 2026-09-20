# 07 — Seed egui's chrome colours from the restored Source colours

**What to build:** The four egui `Visuals` fields the console borrows from Source Paint —
`extreme_bg_color`, `faint_bg_color`, `error_fg_color` and `warn_fg_color` — are seeded from the
restored `SourcePaintSettings` rather than from the compiled-in defaults, so a persisted Source
background does not open against default chrome.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

## The defect

`Console::new` installs the style before it restores anything: `style()` runs at
`console/src/console.rs:800-801`, while the restored `SourcePaintSettings` arrives with
`start.source_paint` at `:860`. So `style()` reads `DEFAULT_SOURCE_BACKGROUND` and `DEFAULT_BANG`
(`console/src/style.rs:296-299`) and cannot do otherwise — the values it would want do not exist
yet.

With `persistence` on, a viewer who retunes Source background to anything but `#000000` and
restarts gets a Source Grid painting the restored colour and chrome still seeded from black. The
two disagree on the first paint of every later session.

## What the existing doc comment already settles

`style()`'s own doc comment (`console/src/style.rs:279-291`) argues that these fields are set once
rather than read per frame, because nothing consults them per Cell, so there is no seam that would
make them track a live `Theme → Source colours` edit the way `show_source` does. **That part
stands and this issue does not reopen it.** A live retune moving the Grid while the chrome baseline
stays is deliberate.

What the same comment then claims — that this is "the same way a restored session would" behave —
is the part to fix. A live edit has no baseline to have been seeded from; a restored session does.
Deciding to leave live edits alone does not decide that startup should ignore a value it holds.

## Acceptance

- [ ] The four fields are seeded from the `SourcePaintSettings` the console starts with, not from
      `DEFAULT_SOURCE_BACKGROUND` and `DEFAULT_BANG`.
- [ ] `style()`'s doc comment keeps the live-edit reasoning and drops the "same way a restored
      session would" equivalence, which this issue disproves.
- [ ] A `persistence` build that stores a non-default Source background opens with chrome seeded
      from it. `mise run test_persistence` passes.
- [ ] A build without `persistence` is unchanged: it starts from the Source Paint defaults, and the
      existing style tests (`console/src/style.rs:674`, `:685`) still pass, adjusted for the
      signature if it changes.
- [ ] A live `Theme → Source colours` edit still leaves the chrome baseline where it was. Pin it,
      so a later change cannot quietly turn this into a per-frame restyle.

## Interaction with 04 and 06

`04` replaces `style()` with `style(theme)` and `install(ctx)`, and `06` removes the
`set_theme(Theme::Dark)` beside it. All three edit the same few lines of `Console::new`. This issue
changes what `style()` reads, not when it is called or which themes it registers, so it composes
with either — but whichever lands second rebases onto the other rather than racing it. No blocking
edge is recorded because neither ordering is wrong.

## Comments

**2026-09-20.** Split out of `syntax-highlighting/07`'s review follow-ups, which listed it as "egui's
own background and error colours are seeded from static defaults while the Grid follows live
settings; seed from the restored settings at startup". It is console chrome and a startup-ordering
question rather than Source Paint, and it sits beside this effort's other two startup issues, so it
belongs here. `syntax-highlighting/07` records the move.
