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

**Status:** ready-for-agent

- [ ] A computation's reserved width has one home, and reading it is a read rather than an index
      into a structure held in parallel with the computations.
- [ ] The hypothetical re-derivation is named so it cannot be read as the settled fact, and the
      agreement between the two is proven by a test over every computation rather than assumed.
- [ ] The replacement guard reads the Function the computation is running, not the one the Parser
      found, and a test covers two replacements reaching one anchor within a Tick.
- [ ] No behaviour changes for a Tick that does not hit the two-replacement case; that case's
      change is stated as a fixed defect and covered.
- [ ] The four call sites that ask a reservation a question each ask it as a question, rather than
      matching on the variant and restating ADR 0036's rule locally.

## Comments

Raised while reviewing `02`, 2026-09-10. The architecture review of the Sequence branch circled
this area under "let the Reservation answer, instead of exporting its variants" and named the
symptom — four call sites re-implementing one rule. The cause is that the value has more than one
home, which is the same shape as the rest of this effort: a fact with two representations and
nothing enforcing that they agree.

The second half is a live defect held back only by an unbuilt capability, so it is worth fixing
before ADR 0034's deferred Function-producing operation makes it reachable, not after.
