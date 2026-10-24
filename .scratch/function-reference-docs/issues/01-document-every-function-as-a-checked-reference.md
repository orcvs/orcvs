# 01 — Document every Function as a checked reference

**What to build:** `docs/functions.md`, a reference a reader can look a Function up in. Each entry
gives the spelling, operands and their domains, what the Function answers or does, where it reads
and writes, what makes it diagnose, and a worked example: the Source before a Tick and after it.
A test holds the document to the language.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Every Function in the Function table has one entry, found by its spelling.
- [x] Each entry states its operands in signature order with their domains, whether it answers a
      value or performs an effect, and whether it needs a Bang to take a Turn.
- [x] Every example ticks to the Source the entry shows, with the Play Commands and diagnostics it
      states, and a test fails when a Function or an example drifts.
- [x] The README and the first-result walkthrough link the reference.

## Comments

The console's Function Reference (`console/assets/function_reference.orcvs`) shows one example of
each Function in place, but it states no operand names, domains, Portals or failure cases, and it
is read inside the console. `CONTEXT.md` states the behaviour as glossary prose, one paragraph per
term, and some Functions are only named inside a family's entry. Neither is a lookup reference.

The Function table in `lang/src/atom.rs` is the authority for spellings, operand roles and types,
activation, and Portals. `CONTEXT.md` and the ADRs are the authority for behaviour. The reference
restates them for a reader and links back rather than becoming a second authority.

Every example is checked by `orcvs/tests/function_reference_doc.rs`: it runs each `orcvs tick N`
block's Ticks on the example before it and compares the written Source, the Play Commands of the
last Tick, and whether it diagnoses. Changing one expected answer (`0C` to `0D` under Multiply)
fails the test at that block's line. The first draft's expected Random answer was wrong and the
test caught it; the Bang entry states that `**` empties itself, which every MIDI example shows.
