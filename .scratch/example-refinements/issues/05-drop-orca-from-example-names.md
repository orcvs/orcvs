# 05 — Drop "Orca" from the example names

Status: resolved

`Help > Examples` lists Orca Tracker, Orca Sequencer and Orca Knobs. Name them Tracker, Sequencer
and Knobs, so their links are `?example=tracker`, `?example=sequencer` and `?example=knobs`.

## Acceptance criteria

- [x] `Help > Examples` lists Tracker, Sequencer and Knobs.
- [x] `?example=tracker`, `?example=sequencer` and `?example=knobs` open them, and the old
      `orca-` names open nothing.
- [x] The guides, the website skill and the site's tests use the new names. Each guide still says
      which Learn Orca patch it ports.

## Comments

### 2026-10-29 — Resolved

`console/src/examples.rs` names them Tracker, Sequencer and Knobs, and `link_name` derives
`tracker`, `sequencer` and `knobs` from the names, so `each_example_links_by_its_name` holds the
links. The `orca-` names are kept by nothing: they shipped the same day in PR #336, so no alias
is kept. ADR 0094 still spells its example link `?example=orca-tracker`; it records the decision
as made and is left as written.
