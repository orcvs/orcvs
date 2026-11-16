# Requirements

These documents state what Orcvs is required to do, in current words, one rule in one place. They
are a snapshot: each restates what the accepted [ADRs](../adr/) decide as of the revision named in
its marker, and decides nothing itself. ADRs stay the binding, append-only record of each decision
and its argument, and new decisions keep arriving as ADRs.

A few rules rest on a dated maintainer ruling recorded in `.scratch/` that no ADR states, because it
adds detail without changing what any ADR decides. Each such rule says "no ADR" in its Sources line
and names the ruling; for it, the requirement and that ruling are the whole record. Changing one of
those rules takes an ADR, as changing any decision does.

## Reading order

This is the one statement of the order; every other document points here.

1. [`AGENTS.md`](../../AGENTS.md) for the repository contract and the task routing.
2. [`CONTEXT.md`](../../CONTEXT.md) for what each term means.
3. The owning requirement below, for what the behaviour must be, why, and where it is implemented
   and tested.
4. Nothing newer: a change to an ADR updates every topic in the same change, and the check below
   fails until it does, so the requirement already restates every accepted ADR. Open
   [`docs/adr/`](../adr/) only for an ADR's argument, its rejected alternatives or its history.

Per-Function spellings, operands and examples live in the [Function reference](../functions.md),
which a test holds to the Function table; the requirements state the rules that cross Functions and
link there for the rest.

## Topics

Every topic of the product has a requirement, and each opens with its snapshot marker.

| Topic | Open it when the task touches |
| --- | --- |
| [Language](language.md) | Source, the Grid, Cells and Positions, parsing, Language Units, Atoms and literal types, operands (blank, optional, defaulted), Comments, Function families, nested Returns |
| [Evaluation](evaluation.md) | Ticks, Turns and their order, Portals, placement and movers, activation and Bangs, cycles and errors, time and feedback Functions, committing a Tick against live edits |
| [Playback and outputs](playback.md) | The Playback Engine's lifecycle, Stop and Rewind and what they do to the absolute Tick, the Tick Grid, diagnostics, MIDI, UDP and OSC sends, the Built-in Synth's voices |
| [Console](console.md) | The console window, Source View, Pan, Zoom, Region and keys, touch, Paint and Themes, the Minimap, the Panel, Toasts, the Output list, settings |
| [Site](site.md) | The site, the Small Console and its page messages, links into the console, Copy Link, page settings, which console the site deploys |

Engineering workflow, tooling and CI are not product behaviour and have no requirement here:
[`AGENTS.md`](../../AGENTS.md) and [`docs/tooling.md`](../tooling.md) own them.

## Keeping the snapshot current

Every topic's marker reads "incorporates ADRs through NNNN and ADR changes dated through
YYYY-MM-DD": the document has been reconciled against every ADR up to that number and every ADR
change dated up to that day. An ADR is newer than a marker when its number is above the "through"
number (a new ADR never refills a numbering gap; see [Numbering](../adr/README.md#numbering)), or
when its Status paragraph or an `## Amendment, <date>` heading carries a later date. No marker is
ever left behind an ADR: a change that cannot reconcile every topic does not merge.

Any change to what an ADR decides updates the requirements in the same change: a new ADR, a dated
amendment, or a Status change such as accepting a proposed ADR or marking one superseded.

1. Date the ADR change where the ADR shows it: an `## Amendment, YYYY-MM-DD` heading, or the date in
   the Status paragraph ("Accepted by … on YYYY-MM-DD").
2. Restate the new or changed rule in the owning requirement, with its rule ID, Sources and
   Evidence, and add the ADR to that marker's Source ADRs.
3. Add the ADR to the owner table in [`docs/adr/README.md`](../adr/README.md#the-requirements-restate-these-decisions).
4. Advance every topic's marker to the highest ADR number and the change's date, the topics the
   ADR does not touch included, since the marker records what the topic was checked against. A new
   ADR raises the number; an amendment or Status change to an older ADR moves the date alone.
5. Done when `node scripts/check-requirements.ts` passes. It fails while any ADR is numbered or dated
   past a marker, or while an ADR names an amendment in any form but `## Amendment, YYYY-MM-DD`.
   On a pull request, `node scripts/check-requirements.ts --since HEAD^1` also fails when the change
   touches `docs/adr/` and leaves any topic's marker where it was, unless that marker already names
   the highest ADR and the latest ADR change date. Only that exemption lets a second ADR change dated
   a marker's own day through unchecked, so such a change still reconciles every topic by hand.

An ADR change that alters nothing a requirement states still advances the markers, and says so in
its pull request. A link fix or rewording of a requirement leaves the markers as they are.

A requirement never overrides an accepted ADR. A requirement that disagrees with one is a defect in
the requirement: fix the requirement, or write a new ADR through the
[ADR process](../adr/README.md) if the decision itself should change.

Each document lists accepted-but-unbuilt behaviour, and behaviour the code implements differently,
in its own section, linked to the `.scratch/` issue that tracks the work. Code that lacks a rule
does not retire the rule.

The run record and source-to-rule trace for the snapshot are in
[`.scratch/requirements-snapshot/`](../../.scratch/requirements-snapshot/run-record.md).
