# Requirements snapshot — run record

Status: drafted, audited and reconciled through `c8cc197`; delivered as a draft pull request

This record lets another session resume the requirements rewrite without this conversation. It
follows the `requirements-rewrite` workflow: ADRs stay the binding, append-only history, and
`docs/requirements/` restates them as a snapshot taken at a recorded revision.

## Identity

- Repository: `orcvs/orcvs`, worktree root of the clone.
- Working branch: `claude/project-thread-rd8fij`.
- Target branch: `main`.
- Source SHA (baseline): `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4` (2026-11-14), the merge of
  PR #395. Highest ADR on that tree: 0109. Gaps: 0027 (claimed by an unlanded branch, per
  `docs/adr/README.md`), 0064, 0106.
- Last checked target SHA: `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4` (2026-11-14, Baseline).

## Scope and authorization

- Requested by Toby through the `/requirements-rewrite` command, with no further arguments. Read as:
  write a first requirements snapshot covering the whole accepted ADR catalogue, route readers to
  it, and update the ADR and domain workflow. Deliver as a draft pull request on the working branch.
- Not authorized: implementing recorded defects, changing any product decision, editing ADR bodies,
  merging.
- No existing requirements documents exist at the baseline, so this is a first snapshot rather than
  a refresh.

## Governing instructions and the snapshot model

- `AGENTS.md` (mirrored by the `CLAUDE.md` symlink) names `CONTEXT.md` and `docs/adr/` as the
  sources of truth for architecture and vocabulary. It ranks nothing above accepted ADRs, so it
  agrees with the snapshot model; the rewrite adds the requirements to the reading path beneath the
  ADRs' authority. No conflict to record.
- ADR workflow: `docs/adr/README.md` (numbering and collision rule, enforced by
  `scripts/check-tooling-contract.sh`). Status lines carry supersession and amendment. Decisions are
  settled in `.scratch/<effort>/` and recorded as ADRs.
- Domain-modelling workflow: `docs/agents/domain.md`, read by the mattpocock skills
  (`domain-modeling`, `diagnosing-bugs`, `tdd`, and others). `CONTEXT.md` is the single-context
  glossary.
- Code comments cite `CONTEXT.md` entries by name (for example `console/src/function_reference.rs`
  cites the Terminal Output Function and Halt entries), so moving a behaviour clause out of an
  entry must keep or redirect those citations.

## Topic owners

Topics follow ownership in the code and the task that should make a reader open the document.

| Topic | File | Rule prefix | Owns |
| --- | --- | --- | --- |
| Language | `docs/requirements/language.md` | LANG | Source, Grid, Cells, Positions, parsing, Language Units, Atoms and literal types, operands (blank, optional, defaulted), Comments, Function families, Lists, nested Returns |
| Evaluation | `docs/requirements/evaluation.md` | EVAL | Source Snapshot, Ticks, Turns, dependency scheduling, Portals, placement, cycles, error containment, time and feedback inputs, determinism, commit |
| Playback and outputs | `docs/requirements/playback.md` | PLAY | Playback Engine lifecycle and concurrency, requests, diagnostics, stop and rewind, MIDI, UDP, OSC, the Built-in Synth, output policy modules |
| Console | `docs/requirements/console.md` | CONS | Crate boundary to the UI toolkit, Source View, transform, paint and claims, themes, zoom, selection, keys, touch, Minimap, Cell geometry, splash, Panel, Toasts, Output list |
| Site | `docs/requirements/site.md` | SITE | The web console on the site, its settings, Source by message, deploy, links into the console, Copy Link |

Initial ADR assignment (primary owner; a topic redirects claims it does not own):

- Language: 0005, 0007, 0008, 0010, 0011, 0013, 0015, 0017, 0018, 0019, 0021, 0023, 0024, 0025,
  0026, 0029, 0030, 0033, 0035, 0039, 0043, 0049, 0054, 0061, 0062, 0063, 0067, 0069, 0070, 0071,
  0072, 0073, 0074, 0075, 0103 (language clauses), 0104, 0105, 0107, 0108, 0109.
- Evaluation: 0003, 0004, 0006, 0009, 0012, 0014, 0020, 0028, 0031, 0032, 0034, 0036, 0037,
  0057, 0060, 0065, 0066, 0068, 0077.
- Playback and outputs: 0001, 0002, 0016, 0041, 0055, 0056, 0059, 0076, 0085, 0087 (synth
  clauses), 0088, 0090, 0099, 0103 (sounding clauses).
- Console: 0022, 0038, 0040, 0042, 0044, 0045, 0046, 0047, 0048, 0050, 0051, 0052, 0053, 0058,
  0078, 0079, 0080, 0081, 0082, 0083, 0084, 0089, 0092, 0095, 0096, 0098, 0100, 0102.
- Site: 0086, 0087 (site clauses), 0091, 0093, 0094, 0097, 0101.

## Non-ADR sources (document-level triage)

| Source at `c8cc197` | Disposition | Owner / reason |
| --- | --- | --- |
| `CONTEXT.md` | Retained as glossary; behaviour clauses in entries are dispositioned per topic in `trace/` | Glossary keeps definitions; operational clauses belong to the owning requirement |
| `docs/functions.md` | Retained, redirected-to | Owns per-Function spellings, operands and examples, held to `lang/src/atom.rs` by `orcvs/tests/function_reference_doc.rs` and rendered by the site. Requirements state cross-Function rules and point here for the per-Function table rather than duplicating it |
| `docs/first-result.md` | Presentation-only | Tutorial; teaches, decides nothing |
| `docs/tooling.md`, `docs/agents/*` (except `domain.md`) | Out of scope (engineering workflow) | Engineering and tooling guidance stays with its owner |
| `docs/research/*` | Historical | Research inputs to ADRs |
| `.scratch/*/spec.md`, `decisions.md`, issues | Delivery tracking; evidence | Decisions that bind were recorded as ADRs. Open issues are where accepted-but-unbuilt behaviour is tracked, so gaps link to them |
| `docs/reference/learn-orca` | Out of scope | Reference material |

## Decisions

See `decisions.md` in this directory.

## Validation

Commands, prerequisites and observed results are recorded under "Validation log" as they run.

## Inventory (done 2026-11-14)

Each topic's trace is `trace/<topic>.md`: source disposition, evidence and gaps, conflicts, and
fresh-reader questions with expected answers written from the sources before drafting. Rows by
disposition at inventory: language 198, evaluation 172, playback 163, console 134, site 79.
Conflicts are reconciled in `decisions.md` (D1–D10). Decision requests sent to Toby on 2026-11-14:
D1 (maintenance), D5 (value-root activation), D6 (accept ADRs 0078 and 0079).

Baseline fresh-reader run: an agent with no access to `docs/requirements/` or this directory
answered all 30 questions through the existing reading path (`AGENTS.md`, `CONTEXT.md`,
`docs/agents/domain.md`, `docs/adr/`). Its answers are summarized under "Audit".

## Draft (done 2026-11-14)

`docs/requirements/{language,evaluation,playback,console,site}.md` hold 288 rules under one snapshot
marker each (`c8cc197`, ADRs through 0109), with "Accepted but not built, or built differently" and
"Open decisions" sections. `docs/requirements/README.md` is the index and the workflow guide: the
reading order, the topic list with markers, and same-change maintenance (decisions D1). Each trace
maps every retained row to its final rule ID and records merges.

Routing edits: `AGENTS.md` (sources of truth, Domain docs), `docs/agents/domain.md` (reading list,
conflict rule, recording decisions and terms), `docs/adr/README.md` (ADR-to-topic table, older
vocabulary). `CONTEXT.md`: Linked Console and Small Console added; Function, Clear, Pan, Operand
Literal corrected against the ADRs; Built-in Synth and Playback Engine behaviour moved to the
requirements. No ADR file changed (`git diff --stat c8cc197..HEAD -- docs/adr` lists only the
README).

## Audit (done 2026-11-14)

- Independent audit by a separate agent with no part in drafting: `audit-findings.md`. Two high
  findings (ADR 0088's Sound Module in no requirement; CONS-028 stating a retired Sequence rule with
  nonexistent tests), four medium (CONS-076 binding proposed ADR 0079; PLAY-540/520 duplicating
  SITE rules; CONTEXT.md Playback Engine contradicting PLAY-260; three "ruled" citations without a
  ruling), four low. All resolved: SITE-033 to SITE-038 added; CONS-028, CONS-076, CONS-130,
  CONS-131, CONS-133, PLAY-520, PLAY-540 rewritten; anchors fixed; glossary sentence repointed. The
  auditor's "Jump is now Read" row was wrong (ADR 0070 renames Jump to Copy) and was corrected
  rather than applied.
- Fresh-reader exercises: 30 questions in `reader-questions.md`, expected answers in each trace.
  Baseline through the old reading path: `baseline-answers.md` (main problems: no topic map, chains
  of superseded ADRs, stale facts in ADR bodies). After drafting, a new reader through the new path:
  `postdraft-answers.md`; the owning rule was usually found within three files. Its findings were
  applied: PLAY-230 wording (a blank-note Monophonic Play replaces nothing), LANG-130 cites ADR 0067's
  deferral, LANG-140 names `MA` and `io`, CONS-029 states ADR 0077's retained-record conditions,
  the README routes Stop/Rewind's effect on the absolute Tick to playback. E1, P6 and S5 stayed
  medium or low confidence because the questions are ambiguous, not because a rule is missing.
- Freshness: `origin/main` re-fetched at each phase boundary and before delivery; still
  `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4`, so no reconciliation was needed.

## Validation log

- `python3 -I check-requirements.py .` (the script below) — passed: 288 rules in 6 documents,
  snapshot markers present, no duplicate or undefined rule IDs, no broken relative links in the
  requirements, `docs/adr/README.md` or `docs/agents/domain.md`.
- `node --test scripts/tests/roadmap.test.ts` — passed (10/10).
- `node scripts/roadmap.ts > /dev/null` — passed; the effort lists issues 01–04.
- `cargo fmt --all -- --check` — passed (no Rust changed).
- `git diff --check c8cc197` — passed.
- Not run: crate clippy and nextest gates and the workspace pre-PR runs — the change edits no crate,
  manifest or Rust file. `mise run site_build`/`site_test` — the change does not touch `site/`,
  `docs/first-result.md` or `docs/functions.md`. Everything else in `AGENTS.md`'s list is deferred to
  CI or untouched.

```python
# Validates docs/requirements: snapshot markers, unique rule IDs, rule references, relative links.
import re, os, sys, glob
root = sys.argv[1] if len(sys.argv) > 1 else '.'
req = os.path.join(root, 'docs/requirements')
docs = sorted(glob.glob(os.path.join(req, '*.md')))
errors = []; defined = {}
for d in docs:
    t = open(d).read(); name = os.path.basename(d)
    if name != 'README.md' and not re.search(r'^> Snapshot: checked against `[0-9a-f]{7,40}` \(\d{4}-\d\d-\d\d\); incorporates ADRs through \d{4}\.', t, re.M):
        errors.append(f'{name}: missing snapshot marker')
    for m in re.finditer(r'^### ([A-Z]+-\d{3})\b', t, re.M):
        if m.group(1) in defined: errors.append(f'{name}: {m.group(1)} also defined in {defined[m.group(1)]}')
        defined[m.group(1)] = name
for d in docs:
    t = open(d).read(); name = os.path.basename(d)
    for rid in set(re.findall(r'\b((?:LANG|EVAL|PLAY|CONS|SITE)-\d{3})\b', t)):
        if rid not in defined and not re.search(re.escape(rid) + r'.{0,80}(reserved|merged|Open decision|\(proposed\))', t, re.I | re.S):
            errors.append(f'{name}: references undefined {rid}')
    for link in re.findall(r'\]\(([^)#\s]+)(?:#[^)]*)?\)', t):
        if re.match(r'[a-z]+:', link): continue
        if not os.path.exists(os.path.normpath(os.path.join(req, link))): errors.append(f'{name}: broken link {link}')
for extra in ['docs/adr/README.md', 'docs/agents/domain.md']:
    p = os.path.join(root, extra); base = os.path.dirname(p)
    for link in re.findall(r'\]\(([^)#\s]+)(?:#[^)]*)?\)', open(p).read()):
        if re.match(r'[a-z]+:', link): continue
        if not os.path.exists(os.path.normpath(os.path.join(base, link))): errors.append(f'{extra}: broken link {link}')
print(f'{len(defined)} rules in {len(docs)} documents')
print('\n'.join(errors) or 'no errors'); sys.exit(1 if errors else 0)
```

## Open items

- D5 (value roots and ADR 0006) and D6 (accepting ADRs 0078 and 0079) await Toby; both are listed
  as open in their requirements and in `issues/04`.
- Follow-ups: `issues/01` (trim glossary entries), `issues/02` (stale code citations), `issues/03`
  (ADR status-line forward pointers), `issues/04` (open decisions and missing glossary terms).

## Next action

On a D5 or D6 answer: apply it to `evaluation.md` or `console.md` (and the ADR status lines for D6,
or a new ADR for D5), move the item out of "Open decisions", and re-run the validator. Before merge,
re-fetch `main`; if it moved, classify `git diff --name-status c8cc197..origin/main` by topic and
reconcile affected topics, advancing only their markers.
