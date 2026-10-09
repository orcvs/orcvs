# 07 — Decide the blank-operand edge cases ticket 03 exposed

Status: wontfix
Blocked by: None

**What to decide:**

The ticket 03 review (branch `cell-tracker/blank-results`, commit `8f65a8d7`) found two language behaviours that follow ADR 0062 literally but were not chosen deliberately. Both are active language design and need a human ruling before ticket 09 applies them. Track (ticket 04, branch `cell-tracker/track`) inherits whichever answer is chosen.

## Questions

1. **Every operand blank.** A freshly typed `.+` (both operands two spaces) is currently a valid root that answers blank and writes two spaces at its Output Portal every Tick. Before ticket 03 it was an incomplete Expression with a Language Map diagnostic and no write. Consequences of the current behaviour: typing a value Function immediately clears the Cells south of it; the "incomplete" diagnostic is gone; the half-typed Function reserves its Output Portal and can create competing writes or cycles with neighbours mid-edit. Options:
   - Keep it: every blank operand is still a blank operand. Record the consequences in ADR 0062.
   - Require at least one non-blank operand: an Expression whose operands are all blank stays incomplete, diagnoses and writes nothing. Partly blank operands remain malformed as now.
2. **A Jump copying two spaces.** Ticket 03 made a Jump whose input Portal holds two spaces give the Blank Answer, so a nested Jump returns blank rather than nothing. ADR 0062 as amended records this, but ticket 03 says the explicit encoding "applies only when an inline operand is blank", and the original ADR said "Only an all-empty inline operand, or a blank Return or Portal write that leaves one, produces a blank answer." Options:
   - Keep it: a Jump's copy is a deliberate two-space delivery, matching spec decision 10's rule for Track copying a blank Item.
   - Narrow it: a Jump reading blanks answers the Absence Marker (no write; a nested parent diagnoses "returned nothing"), and the ADR 0062 amendment is removed.

## Answer

### Blank, not zero (ruled 2026-10-05)

A blank operand gives the Blank Answer (two spaces, propagated), not Orca's empty-reads-as-zero. Zero was not weighed when commit `25564ebf` changed ADR 0062 from "no effect" to the Blank Answer, so this ruling makes the choice explicit. Reason: under zero a rest survives a copy (Track to Timed Play) but becomes a note as soon as arithmetic touches it, e.g. a transposed tracker `.+@t…0C` plays `0C` at every rest; spec user story 17 requires a blank slot to stay blank through nested expressions. A per-type hybrid (zero for Numbers, blank for Notes) reintroduces the same bug, because a returned Note enters `.+` as a Number (ADR 0061). Accepted costs: Increment and Interpolation restart from `00` after a blank operand instead of holding state.

ADR 0062 must record that zero was considered and rejected, with this reason, and correct its claim that "Orca behaves the same way: an operator with an empty port has nothing to do". In Orca (`desktop/sources/scripts/core/operator.js` `listen`, `orca.js` `valueOf`) an empty port reads as `0` or the port's default: `A` and `C` run on zero and write every frame; `T` copying an empty Cell writes `.`; only the I/O operators `:`, `%`, `!`, `?` and `=` return early when a port they need is empty.

### Question 1: every operand blank (ruled 2026-10-05)

Keep it: a freshly typed `.+` is a valid root that gives the Blank Answer. Treating all-blank as incomplete protects only the bare spelling, since typing passes through `.+01`, which has one blank operand and clears under either rule; Orca likewise writes as soon as an operator is typed. Add to ADR 0062's Consequences: a value Function clears its Output Portal as soon as it is typed and reserves those Cells while its operands are blank.

### Question 2: a Jump copying two spaces (ruled 2026-10-05)

Keep it: a Jump whose aligned input is two spaces gives the Blank Answer, so a nested Jump returns blank. A root Jump already wrote spaces for empty input before ticket 03; answering the Absence Marker would write spaces south yet return nothing, breaking ADR 0061's rule that a Return is the same encoding the Function writes, and contradicting the Absence Marker's "plans no Cell write". Reword ticket 03's "applies only when an inline operand is blank" to say the rule withholds a blank encoding from the Absence Marker, not from copying Functions.

### S1 (ticket 03 review): `try_parse` (ruled 2026-10-05)

Remove `Parser::try_parse` with `SyntaxError::CommentIsNotAValue`, `SyntaxError::BlankOperandIsNotAValue` and `Expression::has_blank_operand` if nothing shipped needs them. Point the `parse` bench (`lang/benches/lang.rs`, no floor), `lang/tests/allocation.rs`, the `orcvs` properties (`tick.rs`, `language_map.rs`) and `interpret_source` at `analyze()`, checking no diagnostics and held Atoms where they relied on strict success. Ticket 03 review finding J2 is moot once the predicate's callers are gone.

### P2 (ticket 04 review): copied `**` (ruled 2026-10-05)

Keep suppression: characters Track copies are an ordinary Cell write. A copied `**` on a root's anchor covers its spelling and suppresses it rather than activating it (a Jump activates because it carries an answered Bang); a copied Function spelling over a running Function suppresses it under the ordinary covering rule, since Function Replacement needs a Function Atom and none is answered today. ADR 0063 states this and corrects "activates no root in the Tick it is copied": copied `**` never activates a root.

### J3 (ticket 04 review): where Track's selection lives (ruled 2026-10-05)

Keep `index % count` in the `lang` Interpreter (option A). Declare Track's count operand as `NonZeroU8` so a zero count cannot be built, removing the `expect("…never 00")` in `lang/src/functions/list.rs`.

## Comments

Superseded by ADR 0066 (PR #200), which replaced the Blank Answer these rulings refine, and ADR 0067 (PR #205), under which copied `**` relays a Bang as a Jump does and a `00` count is diagnosed at the Turn. The P2 and J3 rulings therefore no longer apply.

Ruling S1 does not depend on the Blank Answer and is carried by `.scratch/test-only-seams/issues/11-remove-strict-parsing.md`, which is open. The rejection of empty-reads-as-zero and the correction of ADR 0062's Orca sentence are recorded in ADR 0069's Considered options and in ADR 0062.
