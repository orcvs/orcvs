# Issue tracker: Local Markdown

Issues and specs for this repo live as markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01` — never a single combined tickets file
- Triage state is recorded as a `Status:` line near the top of each issue file (see `triage-labels.md` for the open role strings); completed issues use `resolved`
- Optional orthogonal collections are recorded as a comma-separated `Tags:` line
  near the top of an issue. Release membership uses a namespaced tag such as
  `Tags: release/v1`; the issue remains in its ordinary effort and may appear in
  both generated roadmap sections.
- Comments and conversation history append to the bottom of the file under a `## Comments` heading

## Roadmap and release scope

`scripts/roadmap.ts` (`pnpm roadmap` / `pnpm roadmap:html`) rolls every effort's
`Status:`/`Blocked by:`/`Tags:` lines up into one "where are we" view — it is
derived, never hand-maintained.

A `.scratch/ROADMAP.md` file, if present, declares one release scope:

```markdown
# Release title

Tag: release/v1
Goal: One-line prose summary of the release.
Definition: <feature-slug>/definition-of-done.md
Gate: <feature-slug>/NN
```

`Tag:` and `Goal:` are required if the file exists at all. `Definition:` is an
optional path (relative to `.scratch/`) to a plain checklist doc; `Gate:` is an
optional `feature-slug/NN` reference to the one open issue whose resolution
closes the release — the script uses it to compute a critical path (via the
existing `Blocked by:` graph) and a parallel-work bucket, both derived, not
authored. Every open tagged issue must be in the Gate's dependency closure, so
“parallel” means mandatory independent work rather than optional scope. After
the Gate resolves, retain the reference as release history: the script accepts
the settled tagged Gate only when no tagged release work remains open. Without
a `Gate:`, tagged issues are just listed by dependency depth.

## When a skill says "publish to the issue tracker"

Create `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, using the next two-digit number from `01` and creating the directories if needed.

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` — the Notes / Decisions-so-far / Fog body.
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.
