# 01 — Expose Source colours as Theme settings

**What to build:** A Theme → Source colours section where each Source Paint role has a colour picker, the defaults are the Okabe–Ito assignment below, a reset restores every default together, and the choices survive restart when persistence is enabled. The Source Grid paints each Token's glyph from these settings rather than from a fixed palette.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Theme → Source colours exposes one opaque colour control per role: Source background, Ordinary (also Char and Atom), Comment, Function, Bang, Number, Note, Sequence, Diagnostic, Result. Diagnostic and Result have no painter until `04` and `06`; they are exposed now so the settings value is complete once.
- [ ] Defaults are exactly:

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

- [ ] The Cell grid line keeps its current colour and is not a Source colour setting.
- [ ] Changes preview immediately in the Source Grid. "Reset to theme defaults" restores every Source colour together and leaves Cursor effects untouched.
- [ ] With persistence, the Source colours are stored under their own key, independently of the Source and of Cursor effects. Missing or malformed stored values fall back to the defaults. Without persistence, defaults apply and edits last for the run.
- [ ] Atom and Sequence stop sharing the Ordinary colour: Atom follows Ordinary and Sequence has its own.
- [ ] The console theme documentation records the new defaults and their source, replacing the glyph entries of the previously decided palette, in the same change as the tests that pin them.
- [ ] The contrast rule is restated rather than silently relaxed: Sequence `#0072B2` measures 4.05:1 on `#000000`, below the current 4.5:1 floor, and the rule and its test name that exception. The "Comment is the dimmest glyph" rule is restated the same way, since Sequence is now dimmer.
- [ ] Focused tests cover defaults, reset, the persistence round trip and malformed fallback, and the feature-off path.

## Comments

**Source.** Decided from the Source Paint prototype (`console/prototypes/syntax-highlighting/source-paint-prototype.html`, `?variant=A&palette=okabe`). Okabe–Ito hex values are as published in R `grDevices` `palette.colors("Okabe-Ito")` (Masataka Okabe & Kei Ito). No overlays: will-execute and diagnostic-coverage overlays were explored and are out of scope.

**Precedent.** Theme → Cursor effects is the shape to follow: one console-owned settings value feeds both rendering and the UI, colour pickers use the opaque colour-edit convention, and persistence has its own key with a default fallback.

**Overlap with open work.** This supersedes the palette-selection part of `restyle-egui-console/04` for Source glyph colours, and makes `console-testing/03`'s pinned glyph values stale. The light palette in `restyle-egui-console/05` is out of scope. Those tickets are not edited here; reconcile them when this lands.
