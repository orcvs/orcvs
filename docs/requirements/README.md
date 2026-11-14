# Requirements

These documents state what Orcvs is required to do, in current words, one rule in one place. They
are a snapshot: each restates what the accepted [ADRs](../adr/) decide as of the revision named in
its marker, and decides nothing itself. ADRs stay the binding, append-only record of each decision
and its argument, and new decisions keep arriving as ADRs.

## Reading order

1. [`AGENTS.md`](../../AGENTS.md) for the repository contract and the task routing.
2. [`CONTEXT.md`](../../CONTEXT.md) for what each term means.
3. The owning requirement below, for what the behaviour must be, why, and where it is implemented
   and tested.
4. [`docs/adr/`](../adr/) for the full argument, the rejected alternatives, the history, and any ADR
   accepted after the requirement's snapshot marker. Where a newer ADR differs, the ADR binds.

Per-Function spellings, operands and examples live in the [Function reference](../functions.md),
which a test holds to the Function table; the requirements state the rules that cross Functions and
link there for the rest.

## Topics

Every topic of the product has a requirement. Each marker names the commit it was checked against
and the highest ADR it incorporates.

| Topic | Open it when the task touches | Snapshot marker |
| --- | --- | --- |
| [Language](language.md) | Source, the Grid, Cells and Positions, parsing, Language Units, Atoms and literal types, operands (blank, optional, defaulted), Comments, Function families, nested Returns | `c8cc197`, 2026-11-14, ADRs through 0109 |
| [Evaluation](evaluation.md) | Ticks, Turns and their order, Portals, placement and movers, activation and Bangs, cycles and errors, time and feedback Functions, committing a Tick against live edits | `c8cc197`, 2026-11-14, ADRs through 0109 |
| [Playback and outputs](playback.md) | The Playback Engine's lifecycle, Stop and Rewind and what they do to the absolute Tick, the Tick Grid, diagnostics, MIDI, UDP and OSC sends, the Built-in Synth's voices | `c8cc197`, 2026-11-14, ADRs through 0109 |
| [Console](console.md) | The console window, Source View, Pan, Zoom, Region and keys, touch, Paint and Themes, the Minimap, the Panel, Toasts, the Output list, settings | `c8cc197`, 2026-11-14, ADRs through 0109 |
| [Site](site.md) | The site, the small console and its page messages, links into the console, Copy Link, page settings, the console's web build and deploy | `c8cc197`, 2026-11-14, ADRs through 0109 |

Engineering workflow, tooling and CI are not product behaviour and have no requirement here:
[`AGENTS.md`](../../AGENTS.md) and [`docs/tooling.md`](../tooling.md) own them.

## Keeping the snapshot current

Accepting an ADR updates the owning requirement in the same change: restate the new or changed
rule, and advance that document's snapshot marker to the commit the change is based on and the new
ADR number. A change that cannot update the requirement says so in the marker (for example "behind:
ADR 0112 not yet restated"), so readers know to check the ADR. A link fix or rewording does not
advance a marker; only reconciling the topic against a newer revision does.

A requirement never overrides an accepted ADR. A requirement that disagrees with one is a defect in
the requirement: fix the requirement, or write a new ADR through the
[ADR process](../adr/README.md) if the decision itself should change.

Each document lists accepted-but-unbuilt behaviour, and behaviour the code implements differently,
in its own section, linked to the `.scratch/` issue that tracks the work. Code that lacks a rule
does not retire the rule.

The run record and source-to-rule trace for the snapshot are in
[`.scratch/requirements-snapshot/`](../../.scratch/requirements-snapshot/run-record.md).
