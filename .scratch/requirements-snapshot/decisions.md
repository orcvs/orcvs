# Requirements snapshot — decisions

Each entry records the question, the evidence, how it is applied in the snapshot, and its basis:
**source-stated** (an accepted ADR or its status line says so), **proposed** (this run's inference,
not final) or **ruled** (a dated maintainer answer). A proposed entry never makes a rule binding by
itself: where it would change what an ADR decides it stays an open decision for the ADR process.

## D1. How the snapshot keeps up with new ADRs — ruled 2026-11-14

Toby chose same-change maintenance on 2026-11-14: accepting an ADR updates the owning requirement
and its snapshot marker in the same change. Written into `docs/requirements/README.md`,
`docs/adr/README.md`, `docs/agents/domain.md` and `AGENTS.md`.

## D2. Ownership splits — proposed

- ADR 0037 (Tick Grid, Overrun): PLAY owns it; EVAL redirects.
- ADR 0067's dynamic-Portal scheduling amendments: EVAL states the scheduling rule; LANG states
  what Track and Read read and points to EVAL for ordering.
- ADR 0087: SITE owns the Linked pick, the page message protocol and timing; PLAY owns the synth as
  a destination, listing order and auto-pick.
- ADR 0103: LANG owns the Chord Play spelling and building; PLAY owns the sounded commands.

## D3. Rulings recorded only in `.scratch/` — proposed

A dated maintainer ruling that records detail no ADR states, and changes nothing an ADR decides, is
restated in the owning requirement with its date and `.scratch/` source, marked "ruled, no ADR".
Examples: the Pitch family `#+ #- #p #s` (`.scratch/chords-and-pitch/spec.md`), the site's 16 KiB
link limit and service-worker rule (`.scratch/orcvs-site/`), console settings keys
(`.scratch/` console settings efforts). A ruling that would reverse an ADR is not restated as
binding; see D4.

## D4. Standalone operand literals — open for the ADR process

ADR 0021 says a standalone operand literal is invalid; ADR 0067 leaves diagnosis to the Parser; a
2026-10-16 ruling in `.scratch/source-diagnostics/issues/01` reverses it for Number and Note pairs,
with no ADR. The code still diagnoses. Snapshot: LANG states ADR 0021's rule as binding and lists
the pending reversal under open decisions. Recording the reversal needs a new ADR.

## D5. Value roots and ADR 0006's "inert until Bang" — awaiting Toby

ADR 0006 says an ordinary root is inert until Bang. ADR 0032 kept "value roots calculate each Tick"
as the first implementation's policy, and code (`ActivationSource`), `docs/functions.md` and later
ADRs assume value roots are intrinsically active. No ADR retires 0006's wider rule. Snapshot: EVAL
states the activation rule as an open decision, describing the built behaviour as observed
implementation, not as binding.

## D6. ADRs 0078 and 0079 are proposed but built — awaiting Toby

Accepted ADRs 0038, 0047, 0053, 0058 and 0091 cite 0079's amendments, and `CONTEXT.md` describes
both. Snapshot: CONS lists their content under "Proposed, built" apart from the binding rules, so
nothing proposed reads as binding. Accepting them is a status-line change through the ADR process.

## D7. Status lines that do not name a later superseding ADR — proposed

0005 (by 0070), 0006 (Portal delivery of Bang, by 0071), 0014 (contact activation, by 0072),
0017 (`@<`/`@>` reservation, by 0070), 0001 ("all-notes-off", widened by 0016), 0053 (follow-OS
appearance, by its own 2026-10-31 amendment). In each case the later accepted ADR states what it
replaces, so the snapshot follows the later ADR (source-stated by that ADR). The status lines are
left unedited: `docs/adr/README.md` does not say status-line edits are permitted outside the ADR
process. Follow-up for the ADR process.

## D8. CONTEXT.md behaviour clauses — proposed

Several glossary entries carry operational policy (Pending, Defaulted Operand, Return, Clear,
Built-in Synth, Pan, Theme and others). Rust comments cite some entries by name. Snapshot: the
owning requirement states the rule; `CONTEXT.md` keeps definitions. Entries whose behaviour clauses
are cited from code are trimmed only where the citation still resolves; the rest are listed as
follow-ups in the run record.

## D9. Conflicts between an ADR and the code or CONTEXT — gaps, not rulings

An implementation that differs from an unambiguous accepted rule is a gap, recorded beside the rule
with its tracking issue or "no issue found":

- PLAY: ADR 0016 says teardown after a refused delivery clears Timed and Mono ownership; the code
  (and `CONTEXT.md`, citing `.scratch/midi-output-family/issues/02`) keeps the schedule and retries.
- PLAY: ADR 0055 names four eviction classes; the code adds `NetworkFailure`.
- SITE: ADR 0087 says a message later than the delay sounds at once; `synth-host.js` drops messages
  queued during audio start-up that are more than 500 ms late.
- SITE: ADR 0101 says every pushed commit's run finishes; workflow concurrency replaces a pending
  run (the archive is still published).
- LANG: `docs/functions.md` and `CONTEXT.md` Clear omit ADR 0109's defaulted `legato`.

## D10. ADR 0016 legato edge rules "for Toby to confirm" — proposed

The amendment is in an accepted ADR, so the snapshot treats it as binding and notes the pending
confirmation.
