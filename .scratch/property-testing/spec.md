# Encode the documented invariants as property tests

**Goal:** Turn the laws that CONTEXT.md and the ADRs already state into tests that run, so a glossary claim and a check are the same artefact.

## Why

`AGENTS.md` already requires this at the parser boundary: "boundary or property tests; fuzz when exposure warrants it". No property-testing dependency exists in the workspace today.

The glossary is already written as property specifications. "A Position can be obtained only from the Grid that contains it, so a Position outside its Grid does not exist" is a containment law. "It partitions each row from left to right into non-overlapping complete Language Units" is a coverage and disjointness law. "General arithmetic wraps within this byte range" is a totality law. None of them is checked.

The language migration in `orcvs-language-migration` will rewrite the parser. Properties bound to the glossary survive that rewrite. Tests bound to today's internals do not.

## Rules

Properties encode invariants that CONTEXT.md and the ADRs already assert. They are written only for behaviour that exists. An unimplemented law is tracked as an issue, never as an `#[ignore]` test that no signal can reach.

The glossary is authoritative. When a property and CONTEXT.md disagree, either the code is wrong or the glossary sentence is wrong. Fix whichever is wrong in the same change. Never weaken a property to match the code.

Use a property only where the input space is large or structured. Several of these laws have domains small enough to test exhaustively: two `u8` operands is 65,536 pairs, one MIDI value is 128 cases. An exhaustive loop proves those completely, and a random sample does not.

## Tooling

`proptest`, as a dev-dependency for non-WASM targets only. Every invariant here is platform-independent logic, so running it under `wasm-pack` buys nothing and adds a dependency graph to maintain.

Tests live in inline `#[cfg(test)]` modules, as every module in this repo already does. This is forced as well as conventional: `LanguageMap` and its `Range` are `pub(super)`, so an integration test cannot reach the partition invariant at all.

Pull requests run 32 cases. The merge tier runs the 256-case default. Counterexample files are committed, so a CI failure reproduces locally.

## Issues

- `issues/01-add-proptest-for-native-targets.md`
- `issues/02-grid-position-round-trip.md`
- `issues/03-parser-totality-on-ascii-input.md`
- `issues/04-language-map-row-partition.md`
- `issues/05-exhaustive-arithmetic-and-note-conversion.md`
- `issues/06-record-the-glossary-authority-rule.md`

## Later

Stateful testing of the Source model — a generated sequence of edits against a model — is a second tranche. It is the largest of the candidates and the least like the others, so it does not belong in the first set.

## Order

This effort follows `crate-boundaries`. A property written against `console` today moves crates immediately afterwards.
