# Deepen the Source module's seams after the Language Map rework

**Status:** ready-for-agent

## Goal

Two architecture reviews of `orcvs/src/source/` found friction that survived the Language Map
rework, plus friction that rework introduced. Each ticket here removes one duplicate description
of a fact the module already holds, or moves an invariant from prose into a type.

Nothing here changes what the language does. ADR 0018 (the Language Map derives from character
Source), ADR 0020 (Tick effects order by Source Position), and ADR 0024 (the Language Map records
spellings, not Atom types) all stand; several tickets exist because the code does not yet match
what those ADRs already say.

## Delivery order

Derived, never authored. Each issue carries its own `Status:` and `Blocked by:` lines, and
`pnpm roadmap` rolls those into the order and into the parallel-work bucket. A list here would be
a second statement of a fact the tickets already hold, which is the friction this effort exists to
remove.

## Required behavior

No Source that parses today may parse differently after any of these tickets. Each is a change of
shape, not of language semantics, and the existing suite is the evidence: a ticket that needs a
behavioural test rewritten to pass has changed something it should not have.

Where a ticket claims a path got cheaper, `orcvs/benches/source.rs` is the arbiter. The repository
contract requires a reproducible benchmark for a performance claim, and three commits on this
branch deliberately made none because the machine could not produce a trustworthy number.
