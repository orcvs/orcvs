# Handoff: Orcvs timing Functions and chord notation research

2026-11-08. Thread "Research new timing operators" in the Orcvs project (repo `orcvs/orcvs`). This
has been research and design only. No code, ticket, branch or PR exists yet.

## Where the work is

- Timing proposals: `docs/research/timing-operators.md`. It ends with six decisions
  for Toby.
- Chord and pattern proposals: `docs/research/chords-and-patterns.md`. The preferred
  design is the "A chord notation (added 2026-10-31)" section. The doc ends with four decisions for
  Toby.

Read both docs before doing anything. They hold the proposals, the prior-art survey and the
sources, and none of that is repeated here.

## Conversation state

1. Toby asked for timing Functions beyond Orca ("At Tick or Sec? Every?") and for deep research
   into Tidal and other tools. The timing doc came out of that.
2. Toby pointed at the archived repos `orcvs/orcvs-ts` (chords, `arp`, Euclidean with rotation,
   `t:` seconds and `f:` frame windows, nested `ptn`) and `orcvs/orcvs-code` (only a VS Code LSP
   scaffold). Both are archived. A session has to add them to read them. The chord doc came out of this.
3. **Toby (2026-10-31): two Cells is a convention for Function spellings and operands, not a
   limit.** So the chord doc now prefers lead-sheet chord symbols (`C4m7`, `G37`, `F4M7/A`, `^1` for
   an inversion) inside a Payload closed by `!`. That's the rule `!u` and `!o` use (ADR 0085). A
   Payload holding several chords is a progression. It's read by `!#` Chord Play
   (`!# channel velocity length index`) and `.c` Chord Tone (`.c index tone`).
4. Toby hasn't answered any of the open decisions. The last message to him summarised the chord
   notation.

## Open decisions (wait for Toby; don't re-ask settled ones)

- **Timing:**
  - Ticks only (no seconds)?
  - A `?` Pulse-condition family?
  - Keep or retire `~*`?
  - Euclidean rotation as a third operand on `~%`, or a new Function?
  - During `~:` as one step, or `whenmod`?
  - Triplets through Ratchet, or a Ticks-per-beat setting?
- **Chords:**
  - Payload notation (A), the four-Cell form (B), or a bracketed literal (C)?
  - Should `!#` send several Play Commands? That amends CONTEXT.md's "one output per Expression".
  - `.s` and `.c` as one-Note value Functions?
  - Scale names, or scale indexes?

## Facts the next agent needs

- Today every Function spelling is two Cells, and the families are:
  - `.` numeric,
  - `~` Tick and feedback,
  - `=` Copy,
  - `&` Read,
  - `@` Write,
  - `*` activation,
  - `!` terminal output.

  `?` and `:` are unused as first characters.
- One Tick is a 16th note: `TICKS_PER_BEAT = 4` in `orcvs/src/opts.rs`. BPM runs from 1 to 15000.
- Constructs that claim a variable run of Cells already exist: the Payload (ADR 0085), the Comment
  (ADR 0074), and Track's list (ADR 0063, ADR 0067). A Return is one two-Cell encoding (ADR 0061).
- A typed operand refuses `**`, and a nested Clear leaves its parent invalid (ADR 0073). The only
  exception is Play's `rest: note` clause (ADR 0075), which is the precedent for the proposed Pulse
  operand.

## Next steps once Toby decides

Follow the repo's process, which Toby insists on (see team memory): a ticket under `.scratch/<slug>/`
(`docs/agents/issue-tracker.md`), then a prototype on real Grid Cells
(`docs/agents/syntax-prototypes.md`, starting from `lang/bang-function-prototype.html`, on a
throwaway `prototype/<slug>` branch), then an ADR, then a test-first implementation. Default to one
PR per effort, with a commit per ticket. Implement everything a decided ADR says.

## Suggested skills

- `mattpocock-skills:grilling`: to walk Toby through the open decisions, if he wants to settle them
  interactively.
- `mattpocock-skills:prototype`: for chord-notation or timing syntax prototypes on Grid Cells.
- `mattpocock-skills:domain-modeling`: for the ADRs (Pulse operand, chord Payload, multi-output
  `!#`) and CONTEXT.md terms.
- `mattpocock-skills:tdd`: for implementation.
- `mattpocock-skills:research`: only if more prior-art digging is asked for.
