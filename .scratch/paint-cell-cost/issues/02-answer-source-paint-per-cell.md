# 02 — Answer Source Paint facts per Cell in the Language Map

**What to build:** Implement ADR 0050. The Language Map derives every distinction Paint needs once
per Source revision and stores the answers per Cell. A Render Frame Cell copies those answers rather
than carrying the parser's shared claim, and `Paint::derive_with_colours` maps them to colours without
walking a Span or hashing a claim pointer.

**Blocked by:** 01 — Apportion the per-Cell regression across the four facts the paint gained.

**Status:** resolved

- [x] The Language Map answers Function, Pending Operand, Valid Operand, Invalid Operand, Bang,
      Comment, and Unclaimed per Cell. Every operand answer independently carries its declared
      Number, Note, Char, Atom, or Sequence Token.
- [x] Pending Operand is answered while the Language Map has the row's Source bytes and the claim's
      Span, once per Source revision rather than once per drawn Cell.
- [x] The Output Portal fact moves beside the other per-Cell Paint facts in the Language Map. It is
      no longer allocated and recomputed for every Render Frame derivation.
- [x] `RenderCell` carries the finished per-Cell facts and no shared `Claim`. The claim `Arc` and the
      console's claim-shaped Cell-range interpretation do not cross the boundary.
- [x] `Paint::derive_with_colours` contains no `HashMap<*const Claim, bool>`, pointer-keyed lookup, or
      call that walks `claim.cells`.
- [x] Existing Paint answers remain identical, including Token tint on pending and invalid operands,
      Output Portal precedence, comments, Bangs, Functions, and unclaimed Cells.
- [x] Boundary tests cover multi-Cell claims and prove every covered Cell receives the same operand
      state and declared Token; tests also cover empty pending slots and written invalid slots.
- [x] The scoped `orcvs` and `console` gates pass. The benchmark comparison is read from the pull
      request action; a local benchmark is not used as acceptance evidence.

## Superseded proposal

The unpublished `paint-cell-cost-tickets` branch proposed ticket 02 as “Answer written on the Render
Frame instead of hashing it per Cell.” ADR 0050 rejects that partial boundary: moving only `written`
would remove the dominant cache cost while leaving the console to interpret the rest of a
language-shaped claim per Cell. This ticket replaces that proposal; the branch must not be merged.

## Then

`theming/06` follows this ticket and must resolve its fact-to-channel mapping once per frame into a
flat lookup. `benchmarks/07` should pin the recovered `paint_derive` floor after the implementation
lands.

## Comments

**2026-09-20.** ADRs 0050 and 0051 were accepted after pull request #114 merged. This ticket records
the implementation that pull request deliberately did not include.

**2026-09-20 — resolved.** `LanguageMap` now stores `SourcePaint` and the fitted Output Portal
highlight per Cell. `RenderCell` copies those values; the shared parser claim, its `Arc`, the
pointer-keyed `HashMap`, and the console's Span walk are gone. The unpublished partial ticket is
superseded by this implementation. Pull request #113 remained open during the work, so its fitted
Output Portal behavior was incorporated at Language Map cadence rather than discarded; the branch
still needs its ordinary merge/rebase coordination before this work is opened as a pull request.
