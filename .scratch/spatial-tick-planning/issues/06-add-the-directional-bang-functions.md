# 06 — Add the Directional Bang Functions

**What to build:** Implement `*^`, `*v`, `*<`, and `*>`. Each is inert until Bang activation. Each
activation writes the matching Self-Banging Function into the two Cells immediately outside its own
two-Cell Span, in the direction its Portal offset declares.

**Stage:** 2 of 2. Issue 03 builds every mechanism this needs except the activation source. These
four differ from the Self-Banging Functions in exactly one declared property: they take ordinary Bang
activation where those take intrinsic. ADR 0029 calls that asymmetry "the whole reason both forms
exist" and refuses to collapse it.

**Blocked by:** 03 — Add the Self-Banging Functions.

**Status:** ready-for-agent

**Tags:** release/v1

**Sources of truth:** `CONTEXT.md` defines Directional Bang Function; ADR 0006 states the emission
geometry as Cell offsets; ADR 0029 states the activation asymmetry; ADRs 0004 and 0009 define
validated Portal effect bundles; `activation-representation/01` (Revised answer only) is the design
of record.

- [ ] `*^`, `*v`, `*<`, and `*>` are rows in `define_functions!`. Each declares no operand, an effect
      kind, ordinary Bang activation, and its Portal offset: `*^` declares `(0, -1)`, `*v` declares
      `(0, 1)` — the default — `*<` declares `(-2, 0)`, and `*>` declares `(2, 0)`.
- [ ] The horizontal offsets are two columns and not one. A Directional Bang Function emits outside
      its own Span, while a Self-Banging Function moves one Cell. A direction name would have hidden
      that difference; the offsets state it.
- [ ] An active Directional Bang Function writes its matching Self-Banging Function at that offset.
      It reuses issue 03's write, including the precondition that the destination Cells are empty and
      inside the Grid.
- [ ] A refused destination diagnoses and emits nothing. This is where the two Function groups
      differ in what a refusal costs: a Self-Banging Function replaces its own Span with Bang.
- [ ] A generated Self-Banging Function first receives a turn from the next Source Snapshot. It does
      not move during the Tick that wrote it.
- [ ] These four Functions answer an effect and not a value, and ADR 0025's single construction point
      refuses each by its declared kind.
- [ ] Tick-by-Tick Source Grid tests cover emission in all four directions, Grid edges, and the
      refused destination. One test drives a full cycle: a Bang activates `*>`, `*>` writes `>>`, and
      `>>` moves on the following Tick.

