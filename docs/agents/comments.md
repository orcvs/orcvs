# What a source comment may carry

A comment states what the code does, why, and the invariant it keeps, in the present tense and about the code as it stands. It does not narrate lineage. An ADR or ticket citation stays only where the constraint it names cannot be seen in the code itself.

This covers every comment in a tracked code or configuration file: `//`, `///` and `//!` in `.rs` files, tests and benches included, and `#` or `//` comments in `.toml`, `.sh`, `.ts`, `.js` and workflow `.yml` files. It does not cover Markdown: `CONTEXT.md`, ADRs and `.scratch/` tickets are where lineage goes.

## The rule

Apply it one sentence at a time, in this order.

1. **Is the sentence lineage?** Ask: _would it still be true, and still worth saying, if the code had been written this way from the start?_ If not, it is lineage. Lineage covers what the code replaced; what was retired, deleted, relocated or renamed; what this code or another module "used to" do; which ticket or ADR introduced it; and an alternative that was tried or considered and then rejected. The usual signal words are _replaces/replaced, retired, relocated, no longer, used to, was once, previously, formerly, the old, is gone, now_ (when it contrasts with a before). **The words alone decide nothing.** They are lineage when their subject is the source code or the design. They are behaviour when their subject is runtime state or the Source being edited. "A stop belongs to a device this engine no longer holds" describes a device released while the program runs, so it stays. "The unaligned `##` scan this replaced could cut a row" describes an earlier version of the file, so it goes.
2. **Remove lineage, but keep the invariant it was protecting.** If the history was there to stop someone undoing something, rewrite it as a present-tense constraint that says what breaks: "Do not X: X causes Y." A rejected alternative may appear only as the thing the constraint forbids, never as the story of how it came to be rejected. If no invariant is left once the history is gone, delete the sentence.
3. **Keep a citation (`ADR NNNN`, `.scratch/…`) only if both of these hold:** the sentence states a constraint a reader could not recover from the code in front of them, and the cited document is where the reasoning for that constraint lives. Most often this is a decision forbidding a change that looks obvious: reading a clock, adding state, collapsing two types that are separate on purpose. Remove the citation when it only records which ADR or ticket produced the code, when the code already shows the rule it names, or when it points at a resolved ticket as provenance. A `.scratch/` citation stays only while that ticket is open **and** the comment states the limitation the ticket tracks. Once the ticket is `resolved`, the citation goes, and so does the limitation if it has been fixed.

**Where lineage goes instead.** What changed and why belongs in the **commit message**, which is the default home and is always one `git log -L` or `git blame` away. A decision and the alternatives it rejected belong in an **ADR** under `docs/adr/`. A negative result, a measurement, or an open follow-up belongs in a **ticket** under `.scratch/<feature>/` (see `issue-tracker.md`). None of these goes stale in the source, because none of them is in the source.

## What the rule does not touch

Some comment text is read by a gate, so shortening it must leave what the gate reads.

- **`// SAFETY:` before an `unsafe` block** is required by `clippy::undocumented_unsafe_blocks`, which the workspace denies. Remove lineage from it, but keep `SAFETY:` and every invariant the block relies on.
- **Doc comments are compiled.** A Rust fence in `///` or `//!` is a doctest that `cargo test --doc` compiles and runs (`no_run` compiles it without running it, `ignore` skips it), and an intra-doc link is checked by the `RUSTDOCFLAGS="-D warnings" cargo doc` gate in `check_pull_request`. Deleting an example deletes a test, so treat it as a test change. A rewritten sentence must keep its links resolving.
- **Tool directives** keep their syntax, and only their free text is held to the rule. The `reason = "…"` on `#[allow]` and `#[expect]`, and the explanation `AGENTS.md` requires beside a lint suppression, say why the lint is inapplicable now, not how the suppression arrived.

## Lineage: before and after

`orcvs/src/source/language_map.rs`, on `walk_row`:

```rust
// Before
/// The whole row goes to the Parser. ADR 0035 moved the Comment into the
/// parse, so there is no pre-pass left that decides where a row's Source
/// stops: the `||` introducer is a spelling the Parser recognizes where a
/// spelling is read, and the Comment it opens claims every Cell after it. The
/// unaligned `##` scan this replaced could cut a row in the middle of a
/// Function, because an Expression may begin at any column and a two-Cell
/// spelling holding a `#` could present one to an overlapping byte pair.

// After: the moved Comment and the replaced scan are history. What is left is
// the rule, and the trap the scan fell into, stated as a constraint.
/// The whole row goes to the Parser, which alone decides where a row's Source
/// stops: the `||` introducer is a spelling read where a spelling is read, and
/// the Comment it opens claims every Cell after it. Do not find Comments with
/// a byte scan ahead of the parse: an Expression may begin at any column, so
/// an overlapping byte pair can split a two-Cell spelling.
```

`orcvs/src/grid.rs`, in the doc comment on the Grid round-trip properties, the paragraph that begins "This replaces the effort's wiring seed". It explains a test that no longer exists and cites a ticket as provenance. The after text is nothing: the property it introduces is named by its own test.

`lang/src/atom.rs`, in `each_midi_domain_type_accepts_exactly_its_protocol_range`:

```rust
// Before
// Relocated from the two validator functions this replaced. The domain
// is now a property of the type, so the conversion is what has to hold
// over the whole byte, and every input is cheap enough to enumerate.

// After: the relocation is the commit's story. The reason for the loop stays.
// The domain is a property of the type, so the conversion has to hold over
// the whole byte, and every input is cheap enough to enumerate.
```

## Lineage reduced to its invariant

`orcvs/src/midi.rs`, asserting the safety action's messages:

```rust
// Before
// The bytes, not a delta: a count that only grew would be satisfied by
// the narrower All Notes Off loop this action replaced.

// After: the replaced loop goes. The weaker assertion it would have passed stays.
// The bytes, not a delta: a count that only grew would be satisfied by an
// action that sends All Notes Off alone, without the rest of each channel's
// safety triple.
```

## A citation that stays, and one that goes

Both are in the module comment of `lang/src/tick.rs`.

This one stays:

```rust
//! A Function whose result depends on time or on where it sits reads it from
//! here rather than from a clock or a static. ADR 0003 puts every piece of
//! language state in the Source Snapshot, so anything that is *not* in the
//! Snapshot has to arrive as an explicit input: that is what makes an
//! identical Source Snapshot interpreted at an identical Tick produce an
//! identical Tick Plan.
```

The constraint is that no Function reads ambient state. The code cannot show that, because absence of a clock read is invisible, and reading one is exactly the obvious shortcut the sentence forbids. ADR 0003 is where the reasoning lives.

This one goes:

```rust
// Before
//! ADR 0012's explicit interpretation inputs.

// After: the ADR only records where the module was decided. The module's
// next paragraph already states the constraint it would point at.
//! Explicit interpretation inputs.
```

## In review

None of this is lintable. It holds in review, so a review of this repository asks of every comment in the diff whether it would still be true had the code been written this way from the start.
