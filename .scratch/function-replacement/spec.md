# Name what a Function replacement changes

**Status:** ready-for-agent

## Goal

`deliver_output` refuses a Function replacement when any of six terms differ between the incoming Function and the running one. All six answer one diagnostic string, so no test can say which term refused a replacement, and a byte-identical duplicate of one term reached `main` through a merge without the suite noticing. Replace the chain with a comparison that answers a named change, and let the diagnostic name it.

Nothing here changes which replacements are refused. The same pairs are refused before and after; what changes is that a refusal says which of five facts differed.

## What the guard is, and what it is not

The guard is unreachable in production. No Function body answers `Atom::Function` — every `Interpretation::Cell` carries a Number, Note, Bang or Empty — and ADR 0034 defers the Source operation that would produce a Function value, stating outright that "nothing in Source produces a Function value". The only path that reaches the guard is the test-only `stated::plan_with_answers`. `test-only-seams/09` records the same fact about its sibling defect: it "is unreachable today only because nothing but a test fixture produces a Function value; ADR 0034 defers the Source operation that would, and this becomes live the day it arrives."

That is the whole scope of this effort. It is a change of shape to a rule stated ahead of the language that reaches it, not a fix to anything a Source can do today. Do not let a reviewer read the duplicate as a live defect: `a || b || b` is the predicate `a || b`, so no Tick behaved differently while it stood.

## The reachability of each term — do not re-derive this

Derived by sweeping the `define_functions!` rows at `lang/src/atom.rs:965-993` against `function_kind!` and `FunctionKind`. The table's four groups are:

| group | `answers_value` | `is_intrinsically_active` | `can_emit_bang` | `source_effect` |
| --- | --- | --- | --- | --- |
| Value, bang=false (11 rows) | true | true | false | `None` |
| Value, bang=true — Delay, Equality, Euclidean | true | true | **true** | `None` |
| Terminal Output (5 rows) | false | false | false | `None` |
| Directional Bang (4 rows) | false | false | false | `Some { Emit }` |
| Self-Banging (4 rows) | false | **true** | false | `Some { Advance }` |

Reading the terms in the order the guard applies them:

1. **Answer kind** — fires, but never alone. Every `answers_value == true` row is also `(intrinsic, no write)`, and no `answers_value == false` row is, so any pair differing here also differs on activation or on the write.
2. **Activation** — fires, but never alone. All four cross-group pairs that differ here also differ on answer kind or on the write.
3. **Bang emission** — independently reachable, and **untested**. `Equality` against `Add` differs on this term and agrees on every other. No assertion in the repository exercises it.
4. **Write** — independently reachable and tested. `SelfBangingEast` against `SelfBangingNorth` differ only in the declared offset.
5. **Width** — **not reachable at all.** `reserved_for` answers `Row` only when a Function declares a Sequence answer or widens over one. No row in the table declares `Sequence` — `lang/src/atom.rs:790-792` says "No Function answers `true` today" — so the recursion can never bottom out in a `Row`. The one thing that can mint a `Row` is a fixture-stated reservation, and `orcvs/src/source/tick/execution.rs:897-905` asserts that a stated reservation and a stated Function replacement cannot be combined. The term evaluates `false` unconditionally in every Tick that can reach the guard.
6. The sixth term is a byte-identical duplicate of the fourth, at `execution.rs:463` and `:485`, arrived through `17c4078` and `f63f5b2`.

One consequence to carry into the work: `a_replacement_that_changes_only_the_activation_source_is_refused` claims RawPlay and `^^` "agree on every other column it reads". They do not — `^^` declares a write and RawPlay does not. The test predates the write term. Its name survives because Activation is still the first difference; its comment does not.

## Decisions

- The comparison splits at the crate line. `lang` answers the four changes it declares; `orcvs` composes that answer with the width comparison, which reads a schedule `lang` has no access to. Today's term order already puts width last, so appending it preserves the order exactly.
- The answer is a first-change-wins `Option<ReplacementChange>` under a stated order. A replacement differing on several terms reports the first.
- The order is today's order, unchanged. Q3's sharper diagnostic is the only observable change this effort makes; re-ordering would make a second one for no gain.
- The four declaration comparisons are a `const` table of `(ReplacementChange, fn(Function, Function) -> bool)` walked in order, not an `if` chain. A chain leaves a duplicated comparison re-creatable as a dead arm the compiler does not warn on; a table lets a test assert each variant appears exactly once. This shape is new to the workspace — `domains()` at `lang/src/atom.rs:842-846` already puts fn pointers in a `const` slice, but per-variant inside a match arm rather than as a flat table. It is the right shape here and becomes the shape for comparing two declarations and naming what differs. It does not become a library: there is one caller, and an abstraction over one caller is the shallow module this work exists to avoid.
- Variants are `AnswerKind`, `Activation`, `BangEmission`, `Write`, `Width`. None shadows an existing type name — `ActivationSource` and `SourceEffect` are already types in `lang`.
- The rationale currently interleaved through the guard becomes doc comments on the variants. The two separately-argued comments on the duplicated term merge into one. If they cannot be merged, that is evidence the two terms were not the same question and the merge has found something.
- The guard keeps reading the Function the computation is **running** rather than the one the Parser found, and the comment recording that no test pins the choice is carried forward unchanged. This effort does not move it.
- No new ADR. The rule is unchanged and already rests on ADRs 0004, 0009, 0032 and 0036; only the diagnostic's specificity moves. `collapse-expression-map/spec.md` is the precedent for declining one on a change of shape behind a crate-private seam.
- `CONTEXT.md` gained a **Function Replacement** entry when this effort was specified, rather than as a ticket.

## Required behavior

No replacement admitted today may be refused afterwards, and none refused today may be admitted. The four existing assertions on the diagnostic message change to name a term; nothing else in the suite changes. A ticket that needs a behavioural test rewritten beyond those four has changed something it should not have.

## Out of scope

- The Source operation that would produce a Function value, which ADR 0034 defers.
- Whether the guard should read the running or the parsed Function.
- Any change to which facts are held fixed. The five are the five.
- Splitting `tick.rs` or narrowing the `tick.rs`/`execution.rs` seam. Both were raised in the same architecture review and are efforts of their own.
