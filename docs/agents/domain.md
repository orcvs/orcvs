# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root, or
- **`CONTEXT-MAP.md`** at the repo root if it exists — it points at one `CONTEXT.md` per context. Read each one relevant to the topic.
- **`docs/requirements/`** — the owning requirement for the area, then the ADRs newer than its
  snapshot marker, in the order its [index](../requirements/README.md#reading-order) states.
- **`docs/adr/`** — the full argument, rejected alternatives and history of a decision. In
  multi-context repos, also check `src/<context>/docs/adr/` for context-scoped decisions.
  `docs/adr/README.md` states how a new one is numbered and which of two ADRs is renumbered when
  they collide.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo:

```text
/
├── CONTEXT.md
├── docs/requirements/
│   ├── README.md
│   └── language.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

## Use the glossary's vocabulary

When output names a domain concept—in an issue title, refactor proposal, hypothesis, or test name—use the term defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept isn't in the glossary, reconsider whether it belongs to the project or note the gap for `/domain-modeling`.

A glossary entry defines a domain term without naming the toolkit, crate, module or type that implements it. An `_Avoid_` line may still name a toolkit's word when that word collides with the term, because it names what not to say.

## Flag ADR and requirement conflicts

If output contradicts an existing ADR or requirement, surface it explicitly rather than silently
overriding:

> _Contradicts ADR-0007 (event-sourced orders)—but worth reopening because…_

[`docs/requirements/README.md`](../requirements/README.md#keeping-the-snapshot-current) states how
a requirement that disagrees with an ADR, or with the code, is resolved.

## Recording a decision or a term

`/domain-modeling` decides when an ADR is warranted and what it holds. Updating the requirements
for a new ADR, an amendment or a Status change:
[`docs/requirements/README.md`](../requirements/README.md#keeping-the-snapshot-current).

Terms still go into `CONTEXT.md` the moment they settle. A renamed term is applied to the
requirements that use it in the same change. `CONTEXT.md` keeps definitions, distinctions and
`_Avoid_` lines; a rule about what the product does belongs in the owning requirement. Some entries
still carry behaviour, and `.scratch/requirements-snapshot/issues/01-trim-glossary-entries-to-definitions.md`
tracks moving it out. Until then, correct such a sentence where it contradicts an ADR, but do not add new
behaviour to an entry.
