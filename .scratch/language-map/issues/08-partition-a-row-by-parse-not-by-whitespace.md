# 08 — Partition a row by parse, not by whitespace

**What to build:** Give the Expression boundary one owner. `walk_row` and `Parser::analyze` both
recognize the same characters today, and neither owns where an Expression ends. Drive the row
partition from the parse, so a row is a sequence of Expressions whose widths come from their root
Functions' arities, and the whitespace group — the "run" — stops being a boundary.

The whitespace scan is old and this issue does not fault it for existing. It was always a way to
choose which characters to hand the Parser. What changed is that it became authoritative; see
**Regression history** below.

**Blocked by:** 07.

**Status:** wontfix

**Tags:** release/v1

- [ ] `walk_row` advances by what the Parser consumed, not by a whitespace group.
- [ ] `RunBoundary`, `run_boundary`, and the run concept are deleted, not renamed.
- [ ] `unit_kind` no longer calls `Function::try_from`, `to_atom_num`, or `to_atom_note`. Unit kinds
      come from the parse that established the Expression.
- [ ] A unit kind still records the spelling and nothing else. An operand unit stays
      `OperandLiteral`; no typed Atom is written back into `LanguageMap::units`.
- [ ] `standalone_run` and `units_tile_span` are deleted. A Bang-only Span parses by the same path as
      every other Span.
- [ ] The Parser owns the `##` Comment rule that `run_boundary` states today.
- [ ] **Scanning rule.** Between Expressions, the Parser skips empty Cells to find the next anchor.
      `.=0101 !>007FC4` is two Expressions and the space between them is not diagnosed.
- [ ] **Claimed-extent rule.** A space inside an Expression's arity-determined extent is an unfilled
      operand, not a terminator. `.+01 02` is one Expression whose second operand is ` 0` and fails
      to bind, plus a leftover `2`. This is a deliberate language change — see **The space rules**.
- [ ] `parse_span` emits one `ExpressionEntry` per Expression and resumes at the Cell after each one.
      Its `span` is exactly the width the parse consumed.
- [ ] `.+0102Z` is a valid Expression `.+0102` and one invalid character `Z`, not one invalid
      Expression.
- [ ] `***` is Bang `**` and one invalid `*`, as ADR 0018 states, and the Bang Expression executes.
- [ ] `.+0102.+0304` is two Expressions, each with its own anchor and its own ordinary result
      destination.
- [ ] `07`'s temporary trailing-content restoration in `parse_span` is removed.
- [ ] `check_expression_capacity` (`orcvs/src/source/model.rs:294`) still refuses an edit that would
      exceed parser capacity.
- [ ] The row-local property holds: the Parser reads one row slice, so no Language Unit, Comment, or
      Expression crosses the row edge.
- [ ] The incremental path (`LanguageMap::rebuild`, `orcvs/src/source/language_map.rs:218`) still
      rebuilds only the changed rows.
- [ ] CONTEXT.md's **Expression** entry no longer defines an Expression as a whitespace-delimited
      span.
- [ ] A new ADR supersedes ADR 0024's two-stage mechanism, records what replaces it, and states both
      space rules above — the one ADR 0018 left out, and the one this issue changes.

## The defect

Two stages recognize the same characters, and the disagreement between them is structural.

`unit_kind` (`orcvs/src/source/language_map.rs:592`) calls, in order: `**`, `Activation::try_from`,
`Function::try_from`, then `to_atom_num` and `to_atom_note`.

`take_language_unit` (`lang/src/parser.rs:118`) calls, in order: `**`, `Activation::try_from`,
`Function::try_from`, then `take_token`, which calls `to_atom_num` and `to_atom_note`.

The same four `lang` entry points, in the same order, over the same bytes. `walk_row` recognizes
every two-Cell pair in the row. `parse_span` (`orcvs/src/source/language_map.rs:455`) then hands the
raw bytes back to `analyze`, which recognizes them a second time.

The two stages differ in what they can know:

- `unit_kind` is context-free. It reads a two-Cell spelling in isolation. It has no arity and no
  operand slots, so it cannot know where an Expression ends.
- The parse is context-sensitive. It reads the root Function, takes the arity, and types each operand
  by the slot it lands in.

ADR 0024 makes the first stage the single owner of a unit's *kind*, and that holds: nothing writes
back into `units`. ADR 0024 also holds operand kinds untyped on purpose, because a typed kind would
record the type provenance ADR 0021 keeps out of Source. Both decisions stand and this issue keeps
them.

What ADR 0024 does not assign is *extent*. Both stages compute where things start and stop, and
nothing reconciles the two answers. The context-free stage needs an extent it cannot derive, so
`walk_row` substituted the only boundary a context-free scan can see:

```rust
walk.spans.push(Span::new(grid, cell(start), cell(column - 1)));
```

One Span per whitespace group (`orcvs/src/source/language_map.rs:659`). That is the run. It appears
nowhere in CONTEXT.md's vocabulary and nowhere in ADR 0018. ADR 0018's rule is per Position and has
no grouping step at all: recognize a complete encoding or report one invalid character, then resume.

## Regression history

The whitespace scan predates the Language Map and is not itself the defect.

`row_extents`, in the `language_map.rs` that `741308d` replaced, splits a row on `SPACE_BYTE` and its
own comment calls it "The one rule for Expression extent". `a7fe3ef` carried it into `walk_row`
unchanged. It reaches further back through `d93f802`, `0471d7f`, and `9809903`, and the 2024 console
app grew extents across adjacent non-space Cells with `start_exp` and `join_exp`.

What changed is which layer's answer counts. In that 2024 app the recorded width came from the parse:
`Parsed { inner: atoms, len: parsed.len() }` (`d2392d7:console/src/app.rs:184`), and `len_at`
returned that length, not the extent's. The whitespace scan chose the characters; the parse decided
the width.

`a8f544c` (2026-08-31, "Keep expressions bounded and diagnosable") inverted that. It made the
whitespace extent authoritative by converting the parse's leftover into
`SyntaxError::UnexpectedTrailingContent`. `8e32100` (2026-09-02) carried the same choice into
`analyze`, which is the path the Language Map uses.

So the run is not newly invented. It is newly load-bearing. This issue returns the width decision to
the parse, which is where it sat before `a8f544c`.

## The space rules

`run_boundary` states one space rule. It is doing two jobs, and only one of them survives unchanged.

**Scanning between Expressions.** After an Expression completes, the Parser resumes and must skip
empty Cells to find the next anchor. `.=0101 !>007FC4` needs this, and a skipped space is not a
diagnosed character. ADR 0018 never stated it — its rule is "recognize a complete encoding or report
that character as invalid", and reporting every empty Cell of a mostly-empty Grid as invalid is not
what it means. This issue states the rule ADR 0018 left out. Record it in the superseding ADR.

**A space inside a claimed extent.** `.+` claims six Cells from its anchor: Function at 0–1, operand
at 2–3, operand at 4–5. Under a fixed-width partition a space at Cell 4 does not end the Expression.
It is an operand that fails to bind, the root takes no turn, and the Cells after the sixth are a
separate matter.

This is a change, not a restoration. Today `.+01 02` is two runs: an `Incomplete` `.+01` and a
standalone Operand Literal `02`. After this issue it is one Expression whose second operand is ` 0`,
plus a leftover `2`. No earlier revision of this repo read a space as an operand Cell.

Three reasons to take the change anyway:

- Ports are spatial. A value in an operator's port is that port's value, not an adjacent thing.
- The scheduler already assumes it. `root_layout` maps layout offsets to Grid Positions so other
  Expressions write results *into* those slots; `check_expression_capacity` and `adjacent_root` exist
  to police those spatial claims.
- Execution already does it. `bind_source` (`lang/src/expression.rs:81`) reads
  `source.get(offset..offset + token.len())` from the anchor through the stored layout, against the
  whole rest of the row. It never stops at a space. Only edit-time partitioning does.

State both rules in the superseding ADR, and add a test for each.

## Evidence

`standalone_run` (`orcvs/src/source/language_map.rs:776`) states the duplication in its own doc
comment — it assembles an Expression from established kinds "rather than by reading the same
characters a second time." It is the exception, so the general path is the second read. It returns
`None` "when any of them is a Function or an Operand Literal", so `****` parses as two Bangs and
works, while anything holding a Function does not.

`a_span_its_units_do_not_tile_is_not_a_standalone_run` (`orcvs/src/source/language_map.rs:1047`)
encodes the wrong behaviour against ADR 0018's own worked example. Its comment gives the mechanism:
for `***`, "the third `*` is diagnosed rather than named, and the Span reaches the Parser, which
takes the Bang and calls the rest trailing content." The test then asserts `atoms().is_none()`. The
Bang the ADR says is there does not execute.

The word is also overloaded. `row_runs` (`orcvs/src/source/language_map.rs:670`) uses "run" for a
contiguous index range of row-major items, unrelated to whitespace. Two concepts, one word, one file.

## Scheduler consequences

`.+0102.+0304` becomes two Producers where it is one invalid Expression today. That is a real ADR
0032 change: two roots, two anchors, two ordinary result destinations, and two nodes in the
dependency graph. The scheduler needs no new rule, because each is an ordinary root. Issue 09 states
it in the Tick tests and removes the rules that compensate for the current behaviour.

Sequencing: this issue lands on branch `06-optimize-tick-scheduling` as one of its first two
commits, ahead of the scheduler work. `orcvs/src/source/tick.rs` on that branch depends on the
current one-Expression-per-run spans, so the scheduler commits rebase onto this one rather than the
other way round. `10` records the rebase; `09` settles the one question it cannot answer alone.

## Comments

Absorbed by `cell-indexed-parse/01 — Partition a row by parse, not by whitespace`, which is this
change plus the reason the whitespace scan became authoritative: the Map slices the Source per run
and hands the Parser a copy, so the Parser's positions are relative to a fragment and every consumer
re-bases them. `cell-indexed-parse/02` then stamps the parse into a Cell-indexed array, which is
where this issue's "one owner of the Expression boundary" actually lands. Dropped here rather than
worked twice.
