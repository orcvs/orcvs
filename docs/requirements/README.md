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
4. Nothing newer: a change to an accepted ADR updates every topic in the same change, and the check
   below fails while any marker has not taken in the accepted ADRs as they stand, so the requirement
   restates every accepted ADR. Open [`docs/adr/`](../adr/) only for an ADR's argument, its rejected
   alternatives or its history.

Per-Function spellings, operands and examples live in the [Function reference](../functions.md),
which a test holds to the Function table; the requirements state the rules that cross Functions and
link there for the rest.

## Topics

Every topic of the product has a requirement. Each opens with its scope, which names what it does
not own and which topic does, then its snapshot marker.

| Topic | Open it when the task touches |
| --- | --- |
| [Language](language.md) | Source, the Grid, Cells and Positions, parsing, Language Units, Atoms and literal types, operands (blank, optional, defaulted), Comments, Function families, nested Returns |
| [Evaluation](evaluation.md) | Ticks, Turns and their order, Portals, placement and movers, activation and Bangs, cycles and errors, time and feedback Functions, committing a Tick against live edits |
| [Playback and outputs](playback.md) | The Playback Engine's lifecycle, Stop and Rewind and what they do to the absolute Tick, the Tick Grid, diagnostics, MIDI, UDP and OSC sends, the Built-in Synth's voices |
| [Console](console.md) | The console window, Source View, Pan, Zoom, Region and keys, touch, Paint and Themes, the Minimap, the Panel, Toasts, the Output list, settings |
| [Site](site.md) | The site, the Small Console and its page messages, links into the console, Copy Link, page settings, which console the site deploys |

Each rule is one `###` heading that starts with its rule ID: the topic's prefix (`LANG`, `EVAL`,
`PLAY`, `CONS`, `SITE`) and a three-digit number. An ID is stable and never reused, so commits, code
comments and other rules cite it; a gap in the numbers is a retired or never-used ID, not a missing
rule. Headings are grouped by subject and ordered by number within a subject. A new rule takes the
next unused number in its topic.

Engineering workflow, tooling and CI are not product behaviour and have no requirement here:
[`AGENTS.md`](../../AGENTS.md) and [`docs/tooling.md`](../tooling.md) own them.

## Keeping the snapshot current

Every topic's marker reads "incorporates ADRs through NNNN and ADR changes dated through
YYYY-MM-DD (adr xxxxxxxx)": the document has been reconciled against every ADR up to that number,
every ADR change dated up to that day, and the accepted ADR files exactly as the fingerprint
`xxxxxxxx` records them. The fingerprint covers every accepted ADR's file name and text, so any
change to an accepted ADR changes it, whatever date the change carries: a second change dated a
marker's own day, an edit to an ADR's body, an ADR that fills a numbering gap, or an accepted ADR
being rejected. An ADR is newer than a marker when its number is above the "through"
number (a new ADR never refills a numbering gap; see [Numbering](../adr/README.md#numbering)), or
when its Status paragraph or an `## Amendment, <date>` heading carries a later date. No marker is
ever left behind an ADR: a change that cannot reconcile every topic does not merge.

A proposed ADR, one whose Status paragraph starts `Status: proposed`, decides nothing yet, and a
rejected one (`Status: rejected`) decides nothing at all: neither counts as newer than a marker.
Accepting a proposal is the Status change that makes it newer, by its acceptance date, and adds it
to the fingerprint. A change to a proposed or rejected ADR owes
the markers nothing.

Any change to what an ADR decides updates the requirements in the same change: a new ADR, a dated
amendment, or a Status change such as accepting a proposed ADR or marking one superseded.

1. Date the ADR change where the ADR shows it: an `## Amendment, YYYY-MM-DD` heading, or the date in
   the Status paragraph ("Accepted by … on YYYY-MM-DD").
2. Restate the new or changed rule in the owning requirement, with its rule ID, Sources and
   Evidence, and add the ADR to that marker's Source ADRs.
3. Add the ADR to the owner table in [`docs/adr/README.md`](../adr/README.md#the-requirements-restate-these-decisions).
4. Advance every topic's marker, the topics the ADR does not touch included, since the marker
   records what the topic was checked against. `node scripts/check-requirements.ts --marker` prints
   the line every topic carries: the highest ADR number, the latest change date and the fingerprint.
   A new ADR raises the number; an amendment or Status change moves the date unless it falls on the
   marker's own day; any change moves the fingerprint.
5. Done when `node scripts/check-requirements.ts` passes. It fails while any ADR is numbered or dated
   past a marker, while a marker is dated past the latest ADR change, while a marker's fingerprint
   is not the accepted ADRs' own, or while an ADR names an amendment in any form but
   `## Amendment, YYYY-MM-DD`. It reads one tree, so it fails the same way locally and in CI.

An ADR change that alters nothing a requirement states still advances the markers, and says so in
its pull request. A link fix or rewording of a requirement leaves the markers as they are.

A requirement never overrides an accepted ADR. A requirement that disagrees with one is a defect in
the requirement: fix the requirement, or write a new ADR through the
[ADR process](../adr/README.md) if the decision itself should change.

Each document ends with two sections:

- **Accepted but not built, or built differently** lists each rule the code does not yet meet,
  linked to the `.scratch/` issue that tracks the work. The rule still binds: code that lacks a rule
  does not retire the rule.
- **Open decisions** lists, under **Built, awaiting a record**, behaviour the code has that no ADR or
  dated ruling records yet: preserve it as built. Under **Undecided** it lists questions no rule
  answers beyond the ones each item names. ADR housekeeping, such as a Status line to update, is
  tracked in `.scratch/` rather than listed here.

The run record and source-to-rule trace for the snapshot are in
[`.scratch/requirements-snapshot/`](../../.scratch/requirements-snapshot/run-record.md).
