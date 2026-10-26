# 07 — Record the built-in synth

**What to build:** An ADR and a `CONTEXT.md` entry for the built-in synth, decided with Toby before
any code (use the grilling and domain-modeling skills). They decide: the destination's name and
_Avoid_ list; which MIDI messages it answers (Note On and Off, and whether channels or Control
Change select sounds); how many voices sound at once and what happens past that; the sounds; that
embed mode selects it; that the audio context starts or resumes on the action that starts
Playback; and whether native gets it too. See `../spec.md` (Built-in synth) and
`docs/research/orcvs-site.md` section 5.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The ADR takes the next number in `docs/adr/` and follows the existing shape (decision,
      rejected alternatives, consequences).
- [x] Rejected alternatives include a Tone.js iframe bridged from JavaScript, as Learn Orca's Enfer
      is.
- [x] `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` pass.

## Comments

Resolved on 2026-10-26 by ADR 0087 and the Built-in Synth entry in `CONTEXT.md`, decided with Toby
in three rounds. Two answers moved the ticket's starting point: the synth lives in the page's sound
module and the console plays it by message, and the web build ships first with native in ticket
14. The research is `docs/research/tone-generation.md`.
