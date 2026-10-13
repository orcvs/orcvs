# 02 — Say why a root was activated

**Blocked by:** 01, dependency-derivation/02

**Status:** needs-triage

**Do not build this until the question below is answered.** It is filed so the fact is not lost, not so an agent can pick it up.

**What it would build:** activation is recorded twice, as a bare `bool` both times, and neither record can say which producer activated a root or by which path.

- **At schedule time**, `active_roots` (`orcvs/src/source/tick/dependencies.rs:129-142`) returns `Vec<bool>`: the fixpoint closure the schedule orders against. Its doc (`:109-120`) says it is deliberately wider than the Tick, because which root a delivery actually reaches depends on values no schedule has yet.
- **At run time**, `ComputationState.activated` (`orcvs/src/source/tick/execution.rs:105`) is set from two sites: a Bang write to a root's cardinal anchors (`:635`), and a Bang Function firing over its own Span (`:892`). `Turn::Taken { activated }` (`orcvs/src/source/tick/ordering.rs:181`) already carries the roots a dynamic write activated in the Turn that took it. Those writes are idempotent; nothing distinguishes a first writer from a later one.

`active_roots` closes over one delivery declaration, `can_emit_bang` over `bang_roots`, in `activate` (`dependencies.rs:150-170`), and the guard `!active[index]` keeps the first (`:157-163`):

```rust
if !nodes[index].function.can_emit_bang() {
    continue;
}
for relationships in reservations(lookup, index) {
    for index in relationships.bang_roots().collect::<Vec<_>>() {
        if !nodes[index].function.is_intrinsically_active() && !active[index] {
            active[index] = true;
            pending.push(index);
        }
```

The fixpoint is where the schedule's provenance goes; the two `activated = true` sites are where the execution's goes. Its doc (`:121-126`) states the one declaration. The shape it wants is roughly:

```rust
enum Activation {
    Intrinsic,
    Banged { by: usize },
}

fn active_roots(lookup: &Lookup) -> Vec<Option<Activation>>
```

`Option::is_some()` answers everything the `bool` answered, so `order_turns`'s existing reads of `active[..]` change shape but not meaning.

## The questions to settle first

**Which record carries the provenance?** `active_roots` is an over-approximation: a root it marks may never be reached this Tick, so provenance there says who *could* activate a root. `ComputationState.activated` records who actually did. The two answer different questions, and the ticket must say which one it is asking before the one-or-all question below means anything.

**When two producers both activate one root, does the record keep the first, or all of them?**

Today the fixpoint stops at the first writer because a `bool` cannot hold a second. That is an accident of the type, not a decision anyone made — and naming the fact forces the choice. It is a language question rather than an implementation one: it asks whether "this root was activated" is one event with an owner, or a set of deliveries that happen to converge.

ADR 0006 gives activation two delivery paths and does not say; ADR 0072 removed contact activation, leaving one. Answer it — in this ticket's comments, or in an ADR if the answer needs explaining to a future reader — before any code is written.

Note also that in `active_roots` the guard reads `!active[index]`, so under a first-writer-wins rule the fixpoint's `pending` push is what decides whose delivery is recorded, and that depends on `pending.pop()` order. If the answer is "the first", the ticket owes a statement of what "first" means that does not rest on the traversal order of a work list.

## If it is built

- Nothing about which roots are active may change. The set is the same; only the record beside it is new.
- The accessor follows `01`'s form and `interpreted`'s.
- `can_emit_bang` stays the only question that decides delivery — this records what it already decides.

## Comments

### Audit at cad296df — 2026-09-29

Re-read against the code; the body is corrected in place.

- `active_roots` has three delivery paths, not two: `landed` (Jump, `copies_language_unit()`) was
  added beside `banged` and `contacted` (`orcvs/src/source/tick.rs:849-858`, introduced with
  directional Jump in `4d71ce58`). The quoted snippet, the `Activation` sketch and the "only two
  questions" line are updated; the fn doc at `:806-812` still says two paths.
- The body conflated two records. `active_roots` returns `Vec<bool>`, the schedule's deliberately
  wide closure; `ComputationState.activated` (`orcvs/src/source/tick/execution.rs:72`) is set at run
  time from three sites (`:496-499` Jump landing, `:531-535` Bang write, `:743` advance contact).
  Which of the two should carry provenance is now the first question for triage.
- Status stays `needs-triage`; blocker `01` is resolved.

2026-10-12 — Source-bundle ticket 01 deleted `advances(function)`. Where this ticket quotes it, read `SourceBundle::clears_own_span() && writes_displaced()` on the Function's declared bundle, which `producer_edges` names `moves`.

### 2026-10-13 — Audit against origin/main cf01e3c

Body corrected in place for `7960ba1` (ADR 0072 removed contact activation), `25ebc77` (Jump
renamed) and `a8ca4cf` (`active_roots` moved to `orcvs/src/source/tick/dependencies.rs:129`).
Activation now has one delivery declaration and two run-time sites, so the `Contacted` and
`Landed` variants are gone from the sketch. Both questions still stand. Both blockers are resolved;
Status stays `needs-triage`.
