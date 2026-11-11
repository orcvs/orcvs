# 03: Timed and Monophonic Plays take an optional note

**What to build:** Timed Play `!~` and Monophonic Play `!%` declare `note: Optional<Note>`, so a blank note on either plays nothing and raises no diagnostic, as on `!>`. See `../spec.md`.

**Blocked by:** 02

**Status:** resolved

- [x] Through `Source::execute`: a blank note on `!~` and on `!%` sends no Play Command and raises no diagnostic, for both the written and the Portal routes.
- [x] A half-typed or mistyped note on either is still diagnosed.
- [x] A blank length, or one out of range, is diagnosed on a blank step.
- [x] A `!%` with a blank note leaves its channel's held voice to its own Note Off: a voice already sounding is not cut short, which a playback test shows.
