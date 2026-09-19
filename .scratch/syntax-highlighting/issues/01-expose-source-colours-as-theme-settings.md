# 01 — Expose Source colours as Theme settings

**What to build:** A Theme → Source colours section where each Source Paint role has a colour picker, the defaults are the Okabe–Ito assignment below, a reset restores every default together, and the choices survive restart when persistence is enabled. The Source Grid paints each Token's glyph from these settings rather than from a fixed palette.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Theme → Source colours exposes one opaque colour control per role: Source background, Ordinary (also Char and Atom), Comment, Function, Bang, Number, Note, Sequence, Diagnostic, Result. Diagnostic and Result have no painter until `04` and `06`; they are exposed now so the settings value is complete once.
- [x] Defaults are exactly:

      | Role | Default | Okabe–Ito name |
      |---|---|---|
      | Source background | `#000000` | black |
      | Ordinary, Char, Atom | `#FFFFFF` | — (prototype pick) |
      | Comment | `#999999` | gray |
      | Function | `#009E73` | bluish green |
      | Bang | `#CC79A7` | reddish purple |
      | Number | `#56B4E9` | sky blue |
      | Note | `#F0E442` | yellow |
      | Sequence | `#0072B2` | blue |
      | Diagnostic | `#D55E00` | vermillion |
      | Result | `#E69F00` | orange |

- [x] The Cell grid line keeps its current colour and is not a Source colour setting.
- [x] Changes preview immediately in the Source Grid. "Reset to theme defaults" restores every Source colour together and leaves Cursor effects untouched.
- [x] With persistence, the Source colours are stored under their own key, independently of the Source and of Cursor effects. Missing or malformed stored values fall back to the defaults. Without persistence, defaults apply and edits last for the run.
- [x] Atom and Sequence stop sharing the Ordinary colour: Atom follows Ordinary and Sequence has its own.
- [x] The console theme documentation records the new defaults and their source, replacing the glyph entries of the previously decided palette, in the same change as the tests that pin them.
- [x] The contrast rule is restated rather than silently relaxed: Sequence `#0072B2` measures 4.05:1 on `#000000`, below the current 4.5:1 floor, and the rule and its test name that exception. The "Comment is the dimmest glyph" rule is restated the same way, since Sequence is now dimmer.
- [x] Focused tests cover defaults, reset, the persistence round trip and malformed fallback, and the feature-off path.

## Comments

**Source.** Decided from the Source Paint prototype (`console/prototypes/syntax-highlighting/source-paint-prototype.html`, `?variant=A&palette=okabe`). Okabe–Ito hex values are as published in R `grDevices` `palette.colors("Okabe-Ito")` (Masataka Okabe & Kei Ito). No overlays: will-execute and diagnostic-coverage overlays were explored and are out of scope.

**Precedent.** Theme → Cursor effects is the shape to follow: one console-owned settings value feeds both rendering and the UI, colour pickers use the opaque colour-edit convention, and persistence has its own key with a default fallback.

**Overlap with open work.** This supersedes the palette-selection part of `restyle-egui-console/04` for Source glyph colours, and makes `console-testing/03`'s pinned glyph values stale. The light palette in `restyle-egui-console/05` is out of scope. Those tickets are not edited here; reconcile them when this lands.

**"Comment is the dimmest glyph" does not hold outright under the exact defaults above.** Measured against `#000000`, the ratios are: Ordinary 21.0, Note 15.88, Result 9.32, Number 9.10, Comment 7.37, Bang 6.86, Function 6.14, Diagnostic 5.43, Sequence 4.05. Sequence is the named floor exception the ticket calls out, but Diagnostic, Function and Bang are *also* dimmer than Comment while still clearing 4.5:1 — the fixed Okabe–Ito hex values do not preserve the old hand-picked palette's "Comment is dimmest among the legible colours" property beyond the floor itself. Rather than asserting a false ordering to match the ticket's narrative aside, `style::tests::comment_reads_dimmer_than_ordinary_and_every_non_sequence_colour_clears_the_floor` and `theme.md` state the true relationship: Comment still reads dimmer than Ordinary (that half of the old rule holds), every non-Sequence colour clears the 4.5:1 floor, and Comment is not required to be the dimmest among them. This is the "restate rather than silently relax" instruction applied to a fact the ticket's own exact hex table changes, not a deviation from it.
