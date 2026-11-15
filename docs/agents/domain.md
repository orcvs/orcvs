# Domain Docs

How the engineering skills consume this repository's domain documentation when exploring the
codebase. The repository has one context; its domain documents are the three below.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root — the glossary.
- **`docs/requirements/`** — the owning requirement for the area, then the ADRs newer than its
  snapshot marker, in the order its [index](../requirements/README.md#reading-order) states.
- **`docs/adr/`** — the full argument, rejected alternatives and history of a decision.
  `docs/adr/README.md` states how a new one is numbered and which of two ADRs is renumbered when
  they collide.

## Use the glossary's vocabulary

When output names a domain concept—in an issue title, refactor proposal, hypothesis, or test
name—name it by its bold `CONTEXT.md` heading.

If the concept isn't in the glossary, reconsider whether it belongs to the project or note the gap for `/domain-modeling`.

A glossary entry defines a domain term without naming the toolkit, crate, module or type that implements it. An `_Avoid_` line may name a toolkit's word when that word collides with the term, because it names what not to say.

## Flag ADR and requirement conflicts

If output contradicts an existing ADR or requirement, surface it explicitly rather than silently
overriding:

> _Contradicts ADR 0003 (Source Snapshot is complete language state)—but worth reopening because…_

[`docs/requirements/README.md`](../requirements/README.md#keeping-the-snapshot-current) states how
a requirement that disagrees with an ADR, or with the code, is resolved.

## Recording a decision or a term

`/domain-modeling` decides when an ADR is warranted and what it holds. Updating the requirements
for a new ADR, an amendment or a Status change:
[`docs/requirements/README.md`](../requirements/README.md#keeping-the-snapshot-current).

Terms go into `CONTEXT.md` the moment they settle. A renamed term is applied to the requirements
that use it in the same change. `CONTEXT.md` keeps definitions, distinctions and `_Avoid_` lines;
a rule about what the product does goes in the owning requirement. Where an existing entry carries
behaviour, correct the sentence if it contradicts an ADR and leave moving it out to
`.scratch/requirements-snapshot/issues/01-trim-glossary-entries-to-definitions.md`.
