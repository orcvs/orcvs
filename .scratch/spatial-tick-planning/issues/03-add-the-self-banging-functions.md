# 03 — Add the Self-Banging Functions

**What to build:** Implement the root-only Self-Banging Functions `^^`, `vv`, `<<`, and `>>` as a
composition of the two capabilities ADR 0006 gives them: an activation source, and one write through
a Portal the Function declares rather than the default result position. Build no third "movement"
concept. Movement is what that declared Portal does when its Function already stands at the
destination's origin.

The representation was chosen by `activation-representation/01`. Its **Revised answer —
Self-Banging Functions** section and the "Revised constraints" under it are the design of record.
Nothing below that ticket's "Superseded answer" heading may be cited.

**Stage:** 1 of 2. Issue 06 adds the four Directional Bang Functions that emit these. The split is by
Function group and not by layer, because the match over `Interpretation` is exhaustive: `orcvs` must
handle a Source effect as soon as `lang` can answer one, so a stage that added the representation
without the write path would ship an unreachable arm.

**Blocked by:** `grid-boundedness/01` — Decide whether the Grid's edge is a language concept.
02 and evaluation-machine/05 are resolved. The `grid-boundedness/01` blocker was added on `main`
after stage 1 was built, and stage 1 answers the question it asks without waiting for it: an
out-of-Grid displacement and a row-edge crossing both replace the Function's own Span with Bang,
per ADR 0006. If that decision lands differently, those two refusals and their tests are what it
reaches.

**Status:** ready-for-agent

**Tags:** release/v1

**Sources of truth:** `activation-representation/01` selects the representation (Revised answer
only); `CONTEXT.md` defines Self-Banging Function and Portal; ADR 0006 defines intrinsic Bang
activation and states the geometry as Cell offsets; ADRs 0004 and 0009 define validated Portal
effect bundles and fix the clear-then-write order; ADR 0020 defines producer and emission order;
ADR 0029 makes Sequence membership ask the declared kind of a Function.

### Representation and migration

- [x] `^^`, `vv`, `<<`, and `>>` are rows in `define_functions!`. Each declares no operand, an
      effect kind, and its Portal offset. `CONTEXT.md` already states this: "One of the root-only
      Source Functions `^^`, `vv`, `<<`, and `>>`".
- [x] `Atom::Activation` and `Token::Activation` are removed. The Parser makes an `Atom::Function`
      from each of the four spellings.
- [x] The `Atom::Activation` arm in `Sequence::new` is removed. These four Functions are refused by
      their declared kind, like every other effect Function. One rule keeps one mechanism, which is
      what ADR 0029 asks for.
- [x] `LanguageUnitKind::Activation` is removed. These Cells paint as Function. They paint as **Char**
      today — `orcvs/src/glyph.rs` maps `Token::Activation` to `G::Char` — so this is a visible
      change and a correction: under the design of record the four spellings are Functions.
      `CONTEXT.md` lists no Activation Glyph. A separate Glyph is a display request; file it as its
      own issue if the console must show these four spellings differently.
- [x] Tests that swept `Activation::ALL` read the Function table instead, so a fifth Self-Banging
      Function is covered the day it is declared.

### Capability 1 — the activation source

- [x] A root declares its activation source the way it already declares `can_emit_bang`, and nothing
      else decides it. These four declare intrinsic activation.
- [x] A Self-Banging Function in the Source Snapshot receives intrinsic Bang activation at its turn.
      The activation writes no `**` and routes through no ordinary Atom or value path.
- [x] The activation seed reads that declaration rather than naming a spelling or an Atom variant.
      The seeding pass no longer claims its members are only *potentially* active, and is renamed to
      what it now means.

### Capability 2 — a declared Portal offset, not a direction

- [x] A Function declares where its Portal sits as a Cell offset from its own anchor. The default is
      the ordinary result position one row south. Direction is not modelled: `^^` declares
      `(0, -1)`, `vv` declares `(0, 1)`, `<<` declares `(-1, 0)`, and `>>` declares `(1, 0)`.
      ADR 0006 states the geometry as offsets already, and the design of record keeps "Portals in
      their intended general role: any Source Function may resolve destinations at any Cells".
- [x] One write resolves that declared Portal and places the Function's own two-Cell spelling there.
      It refuses unless the newly entered Cells are empty and inside the Grid.
- [x] Which Cells are newly entered depends on the axis. A horizontal one-Cell move overlaps its old
      Span by one Cell and tests one Cell. A vertical move shares nothing and tests two. The write is
      not carved to match: per ADR 0004 the clear covers the whole old Span, and emission order
      settles the shared Cell.
- [x] A successful move is one preflighted bundle: spaces over the old Span, then the spelling at the
      displaced destination. The design of record spells eastward `>>` at Cells 0 and 1 as
      `Cell 0 <- space`, `Cell 1 <- space`, `Cell 1 <- >`, `Cell 2 <- >`.
- [x] A blocked or out-of-Grid move replaces the current Span with Bang.
- [x] Complete aligned root contact replaces the current Span with Bang *and* delivers activation.
      Partial Language Unit contact replaces it with Bang and diagnoses without delivering. Both
      block the move, per the design of record's four-outcome bundle.
- [x] Scheduling reserves both Portals of the bundle, not the destination alone. A clear is a write.
      A write that reaches a computation which already ran is a scheduler defect, and
      `orcvs/src/source/tick.rs` rejects the Tick for it. `Computation.outputs` is already a `Vec`,
      so the type holds several destinations today.

### Dispatch across the lang and orcvs seam

- [x] `Interpretation` gains a variant for a Source effect. `lang` answers with the spelling to write
      and the offset the Function declares. `orcvs` resolves that offset against the Grid and builds
      the Portal bundle. Terminal Output already crosses the seam this way: `lang` builds a
      `PlayCommand` and the output adapter delivers it. `lang` holds no Grid and no Position, so it
      cannot name a destination itself.
- [x] Each of the four Functions gets a real interpreter arm. No arm is `todo!()` or `unreachable!()`.
      ADR 0028 rules out a panic inside Tick planning.
- [x] The new variant serves issue 06, and issues 04 and 05 as well. Do not shape it for these four
      Functions alone.

### Kind and membership

- [x] These four Functions answer an effect and not a value. ADR 0025's single construction point
      refuses each by its declared kind, per ADR 0029. No check names a spelling.
- [x] Self-Banging Functions stay root-only Source effects. They are not operands, runtime values, or
      Sequence members.

- [ ] Stated destinations reach `computations` without a Terminal Output Function acquiring one:
      ADR 0009's refusal is raised in shipped code and covered by a test of its own, not by the
      `#[cfg(test)]` `carry` helper.

### Tests

- [x] Tick-by-Tick Source Grid tests cover movement in all four directions, Grid edges, and row
      edges.
- [x] Tests cover the blocked move, the out-of-Grid move, complete root contact, and partial contact.

## Comments

2026-09-10: Inherited the west-anchor activation coverage from issue 02, which resolved on ADR
0032's delivered scope. `bang_roots` already looks two columns west, but nothing reaches that arm:
every Function in `define_functions!` declares at least one operand, so a root anchored two columns
west always claims the Bang's own destination for that operand and a Bang there is operand contact.
The Self-Banging Functions are the first roots declaring none, which is what makes the arm live —
hence the new item above. `bang_roots` carries a doc comment naming this ticket as its owner.

2026-09-10: Restated against the recorded design. The ticket had read as eight Functions with a
movement mechanism of their own; ADR 0006 gives them two capabilities and ADR 0004 fixes how the
second one writes. `*^` and `^^` write the same literal to the same directional geometry and differ
in their activation source, plus the clear only `^^` performs. The items are grouped by capability
so an implementation cannot grow a third concept owning a second copy of the emptiness precondition.

Three consequences worth stating, because each is a plausible wrong turn:

- The activation asymmetry belongs in the declaration, not in the scheduler. `lang/src/atom.rs`
  already holds this line for `can_emit_bang` — "every Function states which it is, and nothing else
  is allowed to decide" — and ADR 0029 wants the `Function`-versus-`Activation` distinction in one
  exhaustive match rather than spread across the nine sites that ask a root a question today.
- The emptiness precondition is capability 2's, above `Portal`. `PortalError` has two variants and
  `portal.rs` opens by ruling Source content out of the module; occupancy is a fact about the current
  revision under ADR 0034.
- Precondition and write cover different Cells on purpose. Testing the two Cells a horizontal move
  lands on, or clearing only the Cell it leaves, are the two symmetrical bugs this ticket invites.

The prototype clause is gone: no prototype ticket or file ever existed, and ADR 0029 had already
settled what it deferred to. The west-anchor item now says "a zero-operand root" rather than naming
the Self-Banging Functions, because `*^` is equally zero-operand and reaches that arm without any of
capability 1's work.

2026-09-10 (second): Corrected. An earlier edit today removed this ticket's "focused prototype"
clause as a dangling reference. It was not dangling — the prototype is `activation-representation/01`,
`resolved`, in its own effort directory, and its **Revised answer** is the design of record. The
clause is restored as a citation. The search that missed it looked only inside
`spatial-tick-planning/`.

That prototype also decides two things this ticket had been treating as open:

- **Direction is not a concept.** The design of record keeps "Portals in their intended general
  role: any Source Function may resolve destinations at any Cells", and ADR 0006 already writes the
  geometry as offsets — north `(x, y-1)`, west `(x-2, y)`. A Portal offset is an output property
  every Function has; these Functions simply do not take the default. Modelling a `Direction` enum
  beside `Activation`'s four variants would mint a second four-way type for a fact the offset column
  already carries.
- **The bundle is four ordered writes, not two.** The design of record spells eastward `>>` at
  Cells 0 and 1 as `Cell 0 <- space`, `Cell 1 <- space`, `Cell 1 <- >`, `Cell 2 <- >`, and leans on
  ADR 0020's later-write-wins to commit `.>>`. That confirms the clear covers the whole old Span and
  is not carved around the overlap.

**The conflict a human owes this ticket.** The design of record says a Self-Banging Function "is not
an `Atom`, ordinary Expression value, or Sequence member" and is "a zero-operand Source Function".
ADR 0029 says the opposite of the first half: "`^^`, `vv`, `<<`, and `>>` are their own Atom variant,
so refusing them stays one arm of a match the compiler makes exhaustive" — and `evaluation-machine/05`
closed citing that as the reason the two forms are refused membership by different mechanisms. Both
documents are dated 2026-09-04; `Atom::Activation` predates both, landing 2026-08-31.

The two readings produce different tickets:

- **Zero-operand Source Functions.** Four new `define_functions!` rows. `computations()` picks them
  up unchanged, because they are `Atom::Function`. `Atom::Activation` is deleted and ADR 0029's
  membership argument is restated against the declared kind, as it already is for `*^`. No scheduler
  change beyond the activation seed.
- **A second kind of root.** `Atom::Activation` stays. `computations()` learns to mint a node from
  something that is not a Function, and the six questions the scheduler asks a node's root need one
  answerer per representation.

`v1-roadmap-wayfinding/07` carries an open item asserting the first reading — "`^^`, `vv`, `<<`, and
`>>` are root-only Source Functions with no operand or Sequence behaviour, as CONTEXT.md already
states" — which is evidence but not a decision, since that ticket is itself unresolved.

2026-09-11: The representation is settled. `^^`, `vv`, `<<`, and `>>` become zero-operand root-only
Source Functions. The "Open before implementation" note is removed, and the "conflict" stated in the
comment above it was not one. This comment records why.

**There is one design and one older implementation.** The commit times show it:

- 2026-08-31 — `Atom::Activation` enters `lang/src/atom.rs`.
- 2026-09-02 10:50 — `LanguageUnitKind::Activation` enters `orcvs/src/source/language_map.rs`.
- 2026-09-02 12:38 — `activation-representation/01` is created. Its first answer picks the shape the
  two commits above had already built.
- 2026-09-02 15:47 — the **Revised answer** replaces that first answer.
- 2026-09-04 11:58 — ADR 0029 is created.

The revision states its own reason: the first comparison "treated Function output as ordinary result
placement" and did not test the Portal model. That objection does not survive a declared Portal
offset, which is what capability 2 above gives these Functions.

**ADR 0029 does not decide the representation.** Its commit message names its subject: the glossary
had a name collision, and the Sequence rule listed five spellings. The ADR makes that rule ask the
declared kind of a Function instead. Its sentence about `Atom::Activation` describes the tree of that
day. It is not a constraint on this ticket.

**The migration helps ADR 0029 rather than weakening it.** One rule has two mechanisms today: a match
arm for `^^`, and a declared kind for `*^`. After this ticket, one declared kind covers all eight.

**Constraints accepted with this decision:**

- `define_functions!` gets four rows with no operand. The pervasion, answer-width and domain columns
  say nothing for them. Issue 04 and issue 05 add more rows of this shape. Do not split the macro in
  this ticket; look again when three such groups exist.
- Working code is removed: `Atom::Activation`, `Token::Activation`, the `sequence.rs` match arm, and
  `LanguageUnitKind::Activation`.
- The Glyph of these four spellings changes to Function. The superseded comparison called that a
  fault. Under the design of record it is correct, because these spellings are Functions.

2026-09-11 (second): Two facts and two decisions, all found or made before work starts.

**`define_functions!` accepts a row with no operand.** A temporary row was added to the table and
`lang` was compiled, then the row was removed. The `signature`, `domains`, `operands` and
`unary_operands` expansions each handle zero repetitions. Q1 is answered by evidence.

**The probe gave exactly one error, and it is correct behaviour.** The interpreter dispatches on an
exhaustive match over `Function` at `lang/src/interpreter.rs:137`. A new row must get an arm:

```text
error[E0004]: non-exhaustive patterns: `&atom::Function::ProbeNorth` not covered
   --> lang/src/interpreter.rs:137:46
```

The interpreter has two kinds of arm today. A value arm pushes an Atom onto the stack. A Terminal
Output arm returns `Interpretation::Play`. A Source-writing Function fits neither. This is the gap
the "Dispatch across the lang and orcvs seam" items above now close.

**Decision — `Interpretation` gains a Source-effect variant.** The alternative was to handle these
eight Functions in `orcvs` before the interpreter runs, and leave the arms as `unreachable!()`. That
puts a panic path inside Tick planning, which ADR 0028 rules out. The chosen option repeats a pattern
the codebase already has.

**Decision — scheduling reserves both Portals of a movement bundle.** Reservations make the
dependency edges. If only the destination is reserved, the clear can reach a computation that already
ran, and `tick.rs` rejects the whole Tick for that. Issue 05 states the same rule for Halt.

2026-09-11 (third): **Superseded by the fourth and fifth comments below.** Its stage boundary
was redrawn by Function group, so the scope list here is not this ticket's scope: capability 1
and root contact are in, the Directional Bang Functions are out, and the heading it names
("Kind, membership, and contact") does not exist in the file. Kept for the reasoning, not the
scope. Delivery split into two branches.

**Stage 1 — representation and write machinery.** The four spellings `^^ vv << >>` become
`define_functions!` rows. `Atom::Activation`, `Token::Activation`, the `sequence.rs` match arm and
`LanguageUnitKind::Activation` are removed. `Interpretation` gains its Source-effect variant and the
four Functions get real interpreter arms. `orcvs` resolves the declared Portal offset and builds the
write bundle.

Nothing activates these Functions in stage 1. `potentially_active` seeds value roots only, and these
answer an effect, so they become scheduler computations and sit inert. That is the property that
makes stage 1 a representation change and not a behaviour change: a failure in it is a parse, a
classification, or a membership failure, and never a movement failure.

Sections in scope for stage 1: "Representation and migration", "Dispatch across the lang and orcvs
seam", the write and precondition items of "Capability 2", and the declared-kind items of "Kind,
membership, and contact".

**Stage 2 — activation and emission.** Capability 1 in full: the declared activation source and the
seed that reads it. The four Directional Bang Functions `*^ *v *< *>`. Root contact, partial contact,
the west-anchor item, and the four-direction test matrix.

The boundary was chosen by layer rather than by Function group. A group split would have put
`^^ vv << >>` complete in stage 1 and `*^ *v *< *>` in stage 2, which lands movement before the
representation is proven on its own.

2026-09-11 (fourth): Split into two tickets. This one is stage 1 and covers `^^ vv << >>` complete.
The new issue 06 covers `*^ *v *< *>`.

The earlier plan split by layer: a representation stage with nothing activated, then a behaviour
stage. Implementation showed that boundary does not hold. The match over `Interpretation` in
`orcvs/src/source/tick/execution.rs` is exhaustive, so `orcvs` must handle a Source effect the moment
`lang` can answer one. A stage that stopped at the representation would leave that arm unreachable,
which `CLAUDE.md` forbids and which no test could drive.

Splitting by Function group works because the two groups differ only in their activation source.
`^^` activates itself, so stage 1 is reachable end to end. `*^` needs a Bang, and every mechanism it
needs beyond that is the one stage 1 builds.

One correction found while implementing: these Cells do not paint as a Glyph of their own today.
`orcvs/src/glyph.rs` maps `Token::Activation` to `G::Char`, so `^^` currently paints as an ordinary
character. The migration changes it to Function. That is still a visible change, and it is a
correction rather than a regression.

2026-09-11 (fifth): Stage 1 is built. Two items above stay open and the ticket stays
`ready-for-agent` for them. The west-anchor item that stood here has moved to issue 06, which now
carries it and its test. The stated-destinations item stays here, open, for the reason the
2026-09-10 inherited comment below gives: this branch states a destination from a declaration, not
from input, so it neither reaches ADR 0009's refusal nor retires the `#[cfg(test)]` raiser of it.
Four things are worth carrying forward.

**The activation source is a column of `define_functions!`.** `^^` answers an effect and takes its
Turn anyway, so `answers_value` stopped being the question the activation seed meant. Every row now
declares `Intrinsic` or `Bang`, `Function::is_intrinsically_active` reads it, and the three sites
that asked the value question about activation — the seed, the Turn prologue, and the bang-root
closure — ask this one instead. The declaration is also a term the Function-replacement guard now
compares: Raw Play and `^^` agree on every other column that guard reads and differ here, so a guard
still asking `answers_value` would have admitted that replacement.

**The west-anchor item cannot be driven in stage 1.** The arm is reachable now — `^^` is a root that
`bang_roots` answers with at a west anchor — but reaching it changes nothing, because both callers
act on a root only where the root is *not* intrinsically active, and every zero-operand root there is
takes its Turn without a Bang. `*^` is the first zero-operand root that waits for one. The item and
its test belong with it, and `bang_roots` now carries a doc comment saying so.

*Superseded the same day, by issue 06.* `*^` is built, the item is ticked, and the test that drives
the arm is `an_active_directional_bang_function_emits_its_self_banging_function`: its north and west
fixtures activate their producer through a Bang two columns east, and deleting the arm leaves both
emitting nothing. `bang_roots` names that test instead of naming a future one.

That leaves one acceptance item open, and it is the one inherited from the renamed file rather than
one of this ticket's own: ADR 0009's Portal refusal still has only its `#[cfg(test)]` raiser. The
Status line stays `ready-for-agent` for it. Every other item here is delivered, so an agent picking
this up should read the comment below before re-reading the list.

**Contact is classified against the Language Map, not the Lookup.** A `Lookup` indexes Expressions,
so a Comment and a standalone Bang are absent from it, and ADR 0006 classifies contact against every
Language Unit. The rule is stated without naming a direction: one complete Expression root covering
every newly entered Cell is aligned root contact, a Language Unit met across its edge is partial
contact, and anything else is silent. That reading is what makes the horizontal case work — a
horizontal move enters one Cell, and the root holding it is anchored two columns away, which is
ADR 0006's east and west anchor reached by contact rather than by a Bang.

**The bundle's own Span needed two exceptions.** The clear covers the Cells the producer stands in,
so `order_turns` would have ordered it after itself and rejected every Tick one of these takes a Turn
in, and execution's executed-computation guard would have rejected the Tick for reaching its own
already-attempted computation. Both except a producer that declares a Source write, which is the
declaration saying the overlap is the design. A successful move suppresses nothing, and that is a
rule rather than an omission: the Cells it enters are empty, so no Language Unit stands in them to
have been scheduled.

2026-09-10 (inherited from the ticket file this one renames): `test-only-seams/08` deleted the
scheduler `Configuration`, and with it the only shipped raiser of ADR 0009's "a Terminal Output
Function cannot have a Portal". The refusal now lives solely in the `#[cfg(test)]` `carry` helper,
whose doc says it goes away with the tests that needed it — which was expected to be this ticket's
work. So the change that first lets real input name a destination for a Terminal Output Function is
the same change that removes the last check for it, which is why the acceptance line above is stated
rather than left to be noticed.

What holds the rule in the meantime is the ordering of `computations`' `if` chain: the
`performs_terminal_output()` arm comes first and hands back `vec![]`, so a Terminal Output Function
never reaches the arm that resolves a destination. A stated-destination arm placed before that gate
would admit the pairing silently. The gate itself is covered by one incidental test —
`a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed` is the only test that fails
when its condition is replaced with `false`.

Carried forward verbatim except for one clause. The line had said the `carry` helper is the one
"this work retires"; it is not. A Self-Banging Function does state its destination through
`computations`, which is the shape `carry`'s doc was waiting for, but it states it from a
declaration and no Terminal Output Function can name one — the gate above still comes first, and
this branch kept it there. The refusal therefore has the same single test-only raiser it had
yesterday, and the item stays open for the Source Function that first lets input name a destination.
