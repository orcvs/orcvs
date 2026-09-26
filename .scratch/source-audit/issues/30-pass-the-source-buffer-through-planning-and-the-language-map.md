# 30 — Pass the SourceBuffer through planning and the Language Map

**What to build:** Tick planning and the Language Map receive the Cells as a type that carries their invariant — one printable ASCII byte per Cell — instead of `&[u8]`, which does not. Because `&[u8]` loses the invariant, each consumer that needs text re-establishes it: the Language Map's row walk and three sites in Tick execution call `from_utf8(...).expect("ASCII …")` before handing text to the Parser or reading an operand's spelling, and execution's working copy writes a `CellContent` back as a bare byte. After 08 the Source holds its Cells in a `SourceBuffer`; this ticket lets the buffer's borrowed view travel through `tick::plan`, `LanguageMap::build`/`rebuild`, `claims_by_cell` and execution, so the invariant is proven once where it is established.

**Blocked by:** 08.

**Status:** ready-for-agent

- [ ] Tick planning, Language Map build and rebuild, and Claim lookup take the `SourceBuffer`'s borrowed view rather than `&[u8]`.
- [ ] The view answers a row or span as `&str` through one checked conversion, and the `from_utf8(...).expect` sites in the Language Map and Tick execution are gone. No `unsafe` is introduced to provide the view.
- [ ] Execution's working copy of the Cells has the same invariant-carrying type, and its writes take a `CellContent`.
- [ ] 28's Tick series and the Language Map derive and rebuild benchmarks show no regression beyond noise, or the ticket records the measured cost.
- [ ] If the type now appears in signatures outside the Source module's storage, reconsider whether it needs a `CONTEXT.md` entry. The Source glossary entry avoids "buffer", so any entry must not present the Source itself as a buffer.

## Comments

**2026-09-26 — origin.** Deferred from 08's design so the storage change's benchmarks attribute to storage alone. The type-safety gain is here: the four runtime ASCII checks in the Language Map's row walk and Tick execution's three text reads give way to the view's one checked conversion. Two other shipped checks stay outside this ticket: `SourceBuffer::as_str`, which the view's conversion may reuse, and the per-row check in `file::write`, which reads `Source::cells()` rather than planning's view.
