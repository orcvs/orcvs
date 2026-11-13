# 01 — Settle the decisions the PR #383 review left open

_Ticket numbers below refer to the effort's earlier ticket set, which the vertical slices 02–13
replaced on 13 November 2026._

**What to build:** Every naming and Built-in Synth choice the review of PR #383 raised has an
answer in `decisions.md`, so ticket 20 can write the tickets against settled names.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Ticket 14's controller map: which controllers `OF`, `R2` and `DL` take (`OF` on CC73 Attack
      Time runs backwards, since a higher `OF` skips more of the attack; neither CC92 nor CC94 is
      a General MIDI assignment), and whether the ticket stops calling the map "General MIDI".
- [x] Whether `MD`, `OF`, `LN` and `EQ` stay global although General MIDI controllers are
      per channel, and whether CC74 on the acid voice's channel moves `EQ` as well as the cutoff.
- [x] `07` (dominant 7th) beside `o7` (diminished 7th): rename one, or record the pair as accepted.
- [x] The other near-identical names: `di`/`au` beside `o7`/`+M`, `au` beside `a9`, `io`/`lo`,
      `di`/`do`, and `23`/`24` beside `D2`/`D3` (a bare `24` reads as figured bass). Rename each or
      record it as accepted.
- [x] Ticket 11's names: `rate` for a period (a higher value is slower) or `division`/`period`;
      "Every" although Tidal's `every` applies a change every nth cycle.
- [x] Note spelling, the open item in `decisions.md` that ticket 04 depends on.
- [x] Whether Reset All Controllers keeps resetting the acid voice's sound controllers, against
      MIDI's recommended practice for receivers.
- [x] Which Bjorklund reference ticket 16 matches: Toussaint's table (E(3,4) is `x.xx`, today's
      pattern) or Tidal's `bjorklund` (`xxx.`).
- [x] Which convention gaps are in scope or recorded as out of scope: a rule for hex versus
      decimal numbers in the docs, the `C4 + 1 = C5` example, MIDI clock out, swing, and the fixed
      four Ticks per beat.
- [x] Each answer is recorded in `decisions.md`, and the Open section lists only what is still open.

## Answer

Settled with Toby on 13 November 2026 and recorded in `decisions.md`: `+_` → `+5` and `o7` → `d7`;
Clock's and Every's operand is `division`; every Built-in Synth effect is per channel, with `OF` on
CC85, `LN` on CC75, `R2` on CC92 and `DL` on CC94, and the acid voice's own controllers winning on
its channel; Reset All Controllers follows RP-015; `~%` matches Tidal's `bjorklund`; Note spelling
is documented, not changed; prose gives decimal names with the Source hex beside them; timing
features are out of scope.
