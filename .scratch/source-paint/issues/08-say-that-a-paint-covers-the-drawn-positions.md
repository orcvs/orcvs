# 08 — Say that a Paint covers the drawn Positions

**What to build:** `07` scopes a `Paint` to the Positions the console draws. Three records describe a Paint as covering every Cell of a Render Frame, and each has to say the narrower thing instead.

**Blocked by:** 07

**Status:** resolved

### ADR 0040 is written, not amended

ADR 0040 is added by the last commit of this branch and is not on `main`, so there is no accepted record to amend and no former number to carry. Write it saying what the code does. It keeps its number: `docs/adr/README.md`'s renumbering rule is about two ADRs colliding, which this is not.

- [ ] The decision paragraph says `Paint::derive` takes a Render Frame and the Position range the console draws, and `Paint::cursor()` answers an `Option<Position>`.
- [ ] "**A `Paint` carries no geometry, and that is the whole of what makes it a value.** No `Rect` appears in it" stands as written — it is still true and still the point. Add the distinction the code now rests on: index ranges cross into the value layer and a `Rect` does not, because the reason for the rule is that an assertion about a colour must not have to acquire a viewport, and a `Range<usize>` costs no viewport. `BackgroundRun`'s `columns` was already this decision.
- [ ] Under Consequences, "A second consumer of the paint decision is now possible without a Render Frame being drawn" gains its qualifier: such a consumer states which Positions it wants.
- [ ] Under Consequences, "This ADR claims no performance effect in either direction: the fold, the allocations and the per-Cell work are the same work in a different place, and no benchmark was run to claim otherwise" is no longer true and must not survive. The per-Cell work and the allocation now follow the viewport. Say what is claimed, and cite `09` for how it is measured rather than claiming a number this branch did not measure.
- [ ] Record why the Cursor became optional, because it reads like a weakening and is not one: `RenderFrame::derive` still asserts the Grid owns exactly one selected Position, and a Paint that does not cover that Position is a Paint of a viewport the Cursor is outside. The alternative — an accessor on `RenderFrame` answering the selected Position — was refused as `orcvs` API growth bought for nothing the console needs.

### CONTEXT.md has already landed and does need amending

The **Paint** entry was added by `852255f`, which is earlier on this branch, so unlike the ADR it exists and is wrong.

- [ ] "The per-Cell decision of how one Render Frame is drawn" becomes the decision of how the Positions the console draws are drawn.
- [ ] "A Paint is derived from a Render Frame and carries no geometry" keeps the second half and gains the range.
- [ ] The _Avoid_ list is unchanged.

### The effort spec

`.scratch/source-paint/spec.md` states two decisions `07` overturns and one claim it falsifies. It is the document an agent picking this effort up reads first, so it cannot be left contradicting the code.

- [ ] Under "The value layer": "`Paint::cursor() -> Position`, not a `bool` on every Cell" and "`Paint::derive(&RenderFrame)` takes nothing else" are both revised. The second one's *reason* — no `Orcvs`, no `egui::Context`, because the derive needing a `Context` is what the whole effort turns on — is untouched by `07` and stays exactly as it is. A `VisiblePositions` is neither.
- [ ] Under "Verification": "No benchmark is owed. The shape sequence handed to `painter.extend` is unchanged, so no path a Source reaches changes cost" is false from `07` onward. Point it at `09`.

## Verification

Documentation only; no Rust changes.

```sh
node --test scripts/tests/roadmap.test.ts
node scripts/roadmap.ts > /dev/null
```

`bash scripts/check-tooling-contract.sh` because `docs/adr/` gains a file and the script fails when two files there share a leading number.
