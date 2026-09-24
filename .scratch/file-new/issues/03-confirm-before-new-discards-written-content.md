# 03 — Confirm before New discards written content

**What to build:** Choosing `File > New` while the Source holds written content asks before
replacing it. Cancelling leaves the environment exactly as it was. New on an already-empty Source
asks nothing and opens straight away.

**Blocked by:** 02 — File > New opens an empty default Grid.

**Status:** resolved

**Tags:** release/v1

New is irreversible and it destroys work: there is no undo across an Open, and with `persistence`
on the next ordinary save overwrites the stored revision the discarded Source came from. It is one
menu click away from `Load Function reference`, which is equally destructive and equally unasked —
so whatever this builds should be able to cover both, even if only New uses it here.

The console has no confirmation or modal anywhere today. This introduces the first one, which is
why it is its own ticket rather than a line inside `02`: the pattern it sets will be reached for
again.

- [x] New on a Source holding written content asks before opening.
- [x] Cancelling leaves the Source, Grid, Cursor, playback state and Source View untouched, and
      stores nothing.
- [x] Confirming opens the empty default Grid exactly as `02` specifies.
- [x] New on an empty Source does not ask.
- [x] The confirmation is keyboard-reachable and dismissable, and Escape cancels rather than
      confirms.
- [x] While it is open, Source keys do not reach the Grid behind it — the console already keeps
      keys from the Source while a popup is open, and this follows that rule rather than inventing
      a second one.
- [x] The pattern is general enough that `Load Function reference` could adopt it without being
      rewritten. Whether it does is out of scope here.
- [x] Tests cover: asked and confirmed, asked and cancelled, and not asked on an empty Source.
- [x] The scoped gates for `console` pass, and the egui skill's guidance is followed for any new
      presentation.

## Comments

Deliberately separable. If New is wanted before this lands, `02` ships an unconfirmed, destructive
New and this follows; the acceptance criteria above do not depend on that ordering.

The question is `Console::discard_asking_first(ask, DiscardConfirmation { question, discard })`.
The caller decides `ask` — New asks when any Cell of the running Source is written — and
`discard` is a plain `fn(&mut Console)`, so `Load Function reference` can adopt it by passing its
own question and `Console::load_function_reference` without the confirmation changing. It is an
`egui::Modal`: nothing behind it takes a click or focus, it opens with Cancel focused so Enter on
arrival is the safe answer, Tab reaches Discard, and Escape or a click outside cancels. While it
asks, `keyboard_elsewhere` is set alongside the open-popup rule, so Source keys never reach the
Grid behind it. Cancelling touches nothing and stores nothing; confirming runs the same
`Console::new_source` an unasked New runs.

Review found the command Zoom chords reached the Source View behind the question: `show_source_scene`
reads them from egui's input rather than through the Source's event routing. They are now dropped
from input whenever `keyboard_elsewhere` holds, so the same rule covers them. That also applies
while a menu is open or the Bpm field is focused, where a chord used to zoom the View behind.

Tests in `kittest_tests`: `file_new_on_written_content_asks_and_confirming_opens_an_empty_source`,
`file_new_cancelled_leaves_the_environment_as_it_was` (Playback still playing on the same engine),
`file_new_on_an_empty_source_opens_without_asking`,
`escape_cancels_the_question_and_keys_never_reach_the_source_behind_it`, and
`the_question_is_answered_from_the_keyboard`. The two `02` tests now confirm through
`choose_new_and_discard`, since each writes content before New.
