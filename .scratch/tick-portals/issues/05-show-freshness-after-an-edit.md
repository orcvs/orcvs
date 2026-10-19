# 05: Show Tick Portals' freshness after an edit

**What to build:** After an edit the last record is kept but marked retained,
and it paints only the entries whose owner Function still stands at its anchor,
with no Cell of its recorded subtree edited since the commit; an entry whose
operands selected its place also needs every read recorded by a descendant of
its owner to have read no Cell an edit has written since.
Editing a paused tracker keeps the other voices' playheads lit, and a Function
whose subtree the edit touched stops claiming accesses until the next Tick.
See `../spec.md` sections 5 and 6.

**Blocked by:** 02

**Status:** resolved

- [x] A Source revision answers its record as current, or as retained when an
      edit has landed since the commit.
- [x] An older Source revision keeps the record it was read with.
- [x] A newly opened or loaded Source has no record until its first Tick.
- [x] A retained record paints only entries whose owner still stands, whose
      recorded subtree no edit has written since the commit, and, for a
      selected place, whose owner's descendants' reads read no edited Cell.
- [x] ADR 0077 states the freshness contract.
- [x] Tests for each of the above.

## Comments

2026-10-19 — `SourceRevision::tick_portals()` answers `Option<RevisionTickPortals>`,
a `Copy` value carrying `&TickPortals` and a `Freshness` (`Current` or
`Retained`). The Source keeps the last record with the revision its commit left
(ticket 02); `Source::committed_tick_portals` now answers it at any revision with
`Current` when the Source is still there and `Retained` otherwise, and the
revision captures both as it is read, so a later Tick or edit leaves an older
revision's answer as it was. `Source::new`, the persistence `Deserialize` (which
builds through `Source::new`) and `file::read` already started with no record,
so the opening tests passed before the change; they pin the contract rather than
drive it. `SourceCommander::with_source` carries whatever its Source holds, and
every shipped path into it hands a Source no Tick has run on. The Render Frame
asks whether a retained entry's owner still stands from the claims it already
holds (`LanguageMap::claims_by_cell`): the anchor's claim begins there and parsed
as the entry's Function, as `ScheduledEntry::function` anchors a computation.
So the fold allocates nothing more, and the allocation test now derives a
retained record over the same Cells too. The record is kept across an operand
edit, so a voice whose operand changed still paints where its last Tick read
until the next Tick.

2026-10-19 — The comment above is superseded: an operand edit no longer leaves a
voice painting where its last Tick read. Each revision carries the Cells edits
have written since the commit (`EditedCells`, one bit per Cell), and a retained
entry paints only while its owner stands and none of those Cells fall in the
subtree the entry recorded, so an edit elsewhere in the Expression leaves it
lit. Writing back what stood there still counts as an edit. A Read nested in an
operand answers a value from Cells outside the Expression, so a `Selected` entry
also goes out when a read recorded by a strict descendant of its owner read an
edited Cell. Each `PortalUse` carries the Cells its owner's subtree holds, and a
descendant is an entry anchored in them other than the owner: that covers a
Function between two Reads that made no access, which a recorded parent anchor
would not. A `Declared` entry and a sibling Read in the same Expression stay
lit, as does a Read whose own Cells were edited. The Render Frame marks the
edited reads' anchors in the storage of the Output Portal highlight it has
already folded, so the fold still allocates nothing. ADR 0077, spec section 6
and `CONTEXT.md` state the rule.
