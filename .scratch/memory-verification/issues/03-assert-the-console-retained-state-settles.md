# 03 — Assert the Console's retained state settles

**What to build:** Running the real Console for a long sequence of frames leaves its retained state
flat, so a leak that only shows up after an hour of live coding fails a pull request in seconds.

**Blocked by:** 01 — Count allocations on the Tick and Render Frame paths; console-testing/04 — Drive the console input path through the real App.

**Status:** ready-for-agent

- [ ] The Console is driven frame by frame under the harness `console-testing/04` establishes, at
      its fixed size and pixels-per-point, with no new dependency and no new feature.
- [ ] The assertion is a steady-state comparison: the delta measured over one span of frames matches
      the delta over the next span. Not an absolute count, and not a comparison against a cold start.
- [ ] Retained egui state is checked as well as allocation, so a widget whose `Id` varies per frame
      is caught growing the retained map even if the allocation total looks flat.
- [ ] The frame counts are chosen so the first span is past warm-up, and the choice is stated.
- [ ] Input is driven during the measured spans, not just idle frames, so paths that only run on a
      keystroke are covered.
- [ ] The tests run inside the existing `cargo nextest run --workspace --profile ci --locked` line.
      No mise task, no workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

This is the only leak test in the effort, and the only one that needs a long run rather than a
count. What it is looking for: per-`Id` growth in egui's retained memory, a diagnostics buffer with
no cap, a texture handle recreated every frame.

`console-testing` already decided the tooling — `egui_kittest` at 0.36 with the `eframe` feature and
no others — and already verified that the Console constructs under `build_eframe` with no refactor.
None of that is reopened here. If the harness turns out not to exist in the shape this needs, that
is a comment on `console-testing/04`, not a second dependency decision.
