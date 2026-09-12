# 15 — Decide whether the actor keeps a general transition hatch

**What to decide:** `PlaybackCommand::Adapter(AdapterTransition<A>)` (`orcvs/src/playback.rs:399`,
alias at `:402`) expands to `Box<dyn FnOnce(&mut PlaybackInner<A>) + Send>` — an arbitrary closure
that the task runs against the whole of its state. No ticket in this effort named it. Decide whether
it stays as it is, gets narrowed to the transitions that exist, or gets documented as the deliberate
exception.

The case for it is real. It is crate-private, it is documented in place, and it is how `select` and
`refresh` cross the seam without the general `PlaybackCommand` enum learning MIDI vocabulary — the
same tension `ADR 0041`'s note on `published_destinations` records.

The case against is what ADR 0041 is for. The decision's whole content is that only the task mutates
the state, and that what it can be asked to do is a known set of transitions the type system
enumerates. A variant carrying an arbitrary `FnOnce(&mut PlaybackInner<A>)` is a hole in exactly that
enumeration: it admits any mutation, in any order, from any crate-internal caller, and the enum no
longer tells a reader what can happen to a run.

Narrowing it — two named variants for the two transitions that exist — costs the generic `Adapter`
seam its generality and puts MIDI vocabulary somewhere. That is the same choice ADR 0041 leaves open
about where the destination subscription belongs, so the two should probably be settled together.

**Status:** needs-triage

**Sources of truth:** `orcvs/src/playback.rs:397-402` (the variant and its alias); ADR 0041's closing
paragraphs (what the message set is supposed to guarantee, and the open question beside it).

- [ ] The decision is recorded, either as an ADR amendment or as this ticket's answer.
- [ ] If the hatch stays, the ADR says so — that the message set enumerates every transition *except*
      the adapter's, and why.
- [ ] If it goes, the transitions that replace it are named, and where the MIDI vocabulary lands is
      decided alongside ADR 0041's open question about `published_destinations`.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and
`console`.

## Comments

Filed from the `playback-actor` review ledger as **CR-16**, minor.

Minor because nothing is broken and the hatch is crate-private. Filed because it is an unnamed
exception to the one property this effort exists to establish, and an unnamed exception is the kind
that grows.
