# 02 — Bind a positioned entry by one enum

Status: needs-triage

Type: task

Blocked by: None — can start immediately

**What to build:** replace `PositionedEntry`'s `atom: Option<Atom>` and `unwritten: bool` with one
enum for how the entry is bound: an Atom, unwritten (two blank Cells), or unbound. Today only a doc
comment says an unwritten entry has no Atom, and `push(.., unwritten: bool)` passes the flag by
position, so the compiler cannot hold the invariant.

Raised in review of PR #373 and deferred there, because the change reaches every reader of
`entry.atom` across `lang` and `orcvs`: the render frame, the Language Map, the Tick's inputs and the
Parser's property tests.

## Acceptance criteria

- [ ] `PositionedEntry` cannot represent an unwritten entry that holds an Atom.
- [ ] Every reader of the entry's binding matches on the enum, with no behaviour change.
- [ ] Parsing and Ticks still allocate nothing where the allocation fixtures say so.
