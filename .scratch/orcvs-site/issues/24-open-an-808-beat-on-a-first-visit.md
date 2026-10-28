# 24 — Open an 808 beat on a first visit

**What to build:** A first visit to `/` has something to play. The web console opens a shipped
808 beat when it has no Source stored, instead of the empty Grid.

Toby's decisions (2026-10-27, grilled in the ticket 15 thread):

- The web console opens the beat only when nothing is stored, so a returning reader's work comes
  back. Native still starts on the empty Grid.
- It opens stopped, and Space plays it: that keypress is also the gesture that starts the page's
  audio.
- The 808 kit on channel `08` at the default 120 BPM, in sixteenths: kick `C` on every beat,
  snare `D` on 2 and 4, closed hat `F#` on every eighth. One bar, looping.
- It is a shipped example, `examples/808.orcvs`, listed in `Help → Examples` and linked as
  `?example=808`.
- At most a Comment naming each lane, and no prose.
- No `!c` effect sweep yet; one can come once the Sound Module design settles.

**Acceptance:**

- `examples/808.orcvs` plays its bar as decided with nothing diagnosed
  (`orcvs/src/playback/tests/shipped_trackers.rs`).
- The web console opens it when storage holds no Source; a stored Source, or one that fails to
  decode, starts as before.
- The site's `/` opens the beat on a first visit, and Space plays it on the kit
  (`site/tests/synth.spec.ts`, against a console built from the branch).

**Blocked by:** 08

**Status:** resolved
