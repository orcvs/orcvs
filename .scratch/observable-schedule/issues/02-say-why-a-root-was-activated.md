# 02 — Say why a root was activated

**Blocked by:** 01

**Status:** needs-triage

**Do not build this until the question below is answered.** It is filed so the fact is not lost, not so an agent can pick it up.

**What it would build:** `ComputationState` carries `activated: bool`. What it cannot say is which producer activated the root, or by which of ADR 0006's two paths. `active_roots` branches on both and then collapses them:

```rust
let banged = function.can_emit_bang()
    .then(|| relationships.bang_roots()).into_iter().flatten();
let contacted = advances(function)
    .then(|| relationships.contacted_roots()).into_iter().flatten();

for index in banged.chain(contacted).collect::<Vec<_>>() {
    if !nodes[index].function.is_intrinsically_active() && !active[index] {
        active[index] = true;
        pending.push(index);
    }
}
```

The chain is where the provenance goes. The shape it wants is roughly:

```rust
enum Activation {
    Intrinsic,
    Banged { by: usize },
    Contacted { by: usize },
}

fn active_roots(lookup: &Lookup) -> Vec<Option<Activation>>
```

`Option::is_some()` answers everything the `bool` answered, so `order_turns`'s existing reads of `active[..]` change shape but not meaning.

## The question to settle first

**When two producers both activate one root, does the record keep the first, or all of them?**

Today the fixpoint stops at the first writer because a `bool` cannot hold a second. That is an accident of the type, not a decision anyone made — and naming the fact forces the choice. It is a language question rather than an implementation one: it asks whether "this root was activated" is one event with an owner, or a set of deliveries that happen to converge.

ADR 0006 gives activation two delivery paths and does not say. Answer it — in this ticket's comments, or in an ADR if the answer needs explaining to a future reader — before any code is written.

Note also that the guard reads `!active[index]`, so under a first-writer-wins rule the fixpoint's `pending` push is what decides whose delivery is recorded, and that depends on `pending.pop()` order. If the answer is "the first", the ticket owes a statement of what "first" means that does not rest on the traversal order of a work list.

## If it is built

- Nothing about which roots are active may change. The set is the same; only the record beside it is new.
- The accessor follows `01`'s form and `interpreted`'s.
- `advances` and `can_emit_bang` stay the only two questions that decide delivery — this records what they already decide.
