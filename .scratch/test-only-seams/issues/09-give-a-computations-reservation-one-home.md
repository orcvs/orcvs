# 09 — Give a computation's reservation and its Function one home each

**What to build:** Two facts about a computation each live in more than one place today, with
nothing keeping the copies honest. Give each of them one home, and turn the agreement between what
is stored and what can be re-derived into something the compiler or a test enforces rather than
something a reader assumes.

**How wide a computation's result may be** is stored as a vector held beside the computations it
was derived from, computed once as a bottom-up pass because a computation's reservation reads its
children's. A second accessor re-derives the same fact under a hypothetical Function, reading the
stored vector for the children and the computation record for the hypothesis. The invariant that
re-deriving under a computation's own declared Function must agree with what was stored is written
down nowhere, tested nowhere, and enforced by nothing. Writing one side without the other silently
produces a scheduler that answers two different things to the same question — which is what the
stated-reservation fixture of `02` ran into, and what any edit that changes a Function without
recomputing the pass would run into equally.

The two accessors also read as one question with an extra argument, when they are different
questions: one reads a fact the schedule settled, the other asks what that fact would have been.
A name that cannot be mistaken for the other is part of the fix.

**Which Function a computation is running** is the second. The parsed Function lives on the
computation record and the live one lives on its execution state, and a replacement at an original
anchor changes only the second. The replacement guard reads the first. Two replacements reaching
one anchor within a Tick therefore check the second against the Function the Parser found rather
than against the one already in place, so a replacement that changes activation, output kind or
reserved width can be admitted where it should be refused, and a legitimate one refused where it
should be admitted. This is unreachable today only because nothing but a test fixture produces a
Function value; ADR 0034 defers the Source operation that would, and this becomes live the day it
arrives.

**Blocked by:** 03 — Delete the answer-substitution fork.

**Status:** resolved

- [x] A computation's reserved width has one home, and reading it is a read rather than an index
      into a structure held in parallel with the computations.
- [x] The hypothetical re-derivation is named so it cannot be read as the settled fact, and the
      agreement between the two is proven by a test over every computation rather than assumed.
- [x] The replacement guard reads the Function the computation is running, not the one the Parser
      found, and a test covers two replacements reaching one anchor within a Tick.
- [x] No behaviour changes for a Tick that does not hit the two-replacement case; that case's
      change is stated as a fixed defect and covered.
- [x] The four call sites that ask a reservation a question each ask it as a question, rather than
      matching on the variant and restating ADR 0036's rule locally.

## Comments

Raised while reviewing `02`, 2026-09-10. The architecture review of the Sequence branch circled
this area under "let the Reservation answer, instead of exporting its variants" and named the
symptom — four call sites re-implementing one rule. The cause is that the value has more than one
home, which is the same shape as the rest of this effort: a fact with two representations and
nothing enforcing that they agree.

The second half is a live defect held back only by an unbuilt capability, so it is worth fixing
before ADR 0034's deferred Function-producing operation makes it reachable, not after.

2026-09-10: Resolved by `9fb8a7b`. No behaviour changes, in any Tick.

A computation now carries its own reservation. The `Lookup` no longer holds a `Vec<Reserved>`
beside the computations, so no index can pair a width with the wrong computation.
`derive_reservations` takes `&mut [Computation]` and writes that one field, and `Lookup::reserved`
reads it. A computation is built reserving `Reserved::Pair`, which is what ADR 0036 gives a result
no declaration widens, so the field holds a reservation from the moment it exists.

`Lookup::reserved_with` is now `Lookup::would_reserve`. `Lookup::new` ends with a `debug_assert`
that every computation's settled width is the width its own declared Function re-derives, so every
Source any test in this crate builds is a case for it.
`a_reservation_agrees_with_the_width_its_own_declaration_derives` states the same fact as a test.
It states a `Reserved::Row` for the nested `.-` and re-derives, so the pervasive `.+` above it
reserves a row and `would_reserve` has to answer `Row` — without that the agreement compares two
answers of `Pair` and proves nothing. It also states the one width the agreement is false of: a
stated width is not a declared one, and re-deriving it narrows it away.

The replacement guard reads `states[..].function`. Two tests cover two replacements reaching one
anchor inside one Tick: `live_the_second_replacement_at_one_anchor_replaces_the_first`, where both
are admitted and the Turn runs the second, and
`live_a_second_replacement_at_one_anchor_faces_the_same_refusal`, where the second changes the
output kind and is refused whole while the first stands.

Four call sites now ask the reservation a question rather than matching on the variant:
`Reserved::cells_from`, `Reserved::admits_width`, `Reserved::admits_a_narrower_write` and
`Reserved::may_be_a_sequence`. No production code outside `impl Reserved` matches on the variant.
`derive_reservations` still compares against `Reserved::Pair`, which asks whether a width is still
the undecided default rather than restating ADR 0036's rule.

**The second half was not a live defect.** The ticket said a replacement that changes activation,
output kind or reserved width could be admitted where it should be refused. It could not. The
guard refuses any replacement that changes those three facts, so every admitted replacement agrees
with the Function it replaced on all three, and the parsed Function and the running one cannot
disagree at that check — before a second replacement or after one. Reverting the guard to read the
parsed Function passes all 357 tests, including both new ones above. What the guard relied on was
an invariant about its own refusals, held nowhere and written down nowhere; reading the running
Function is what removes the reliance. The fourth acceptance box has no fixed defect to state, so
this records that instead.

Eight mutation checks ran. Seven broke a named test: the widening term in `reserved_for`, each of
the four reservation questions, the `Lookup::new` assertion, and the rule that a replacement
overwrites an earlier replacement. The eighth is the parsed-Function revert above, which broke
nothing — which is the evidence for the paragraph before this one.
