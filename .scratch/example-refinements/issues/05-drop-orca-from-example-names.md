# 05 — Drop "Orca" from the example names

Status: ready-for-agent

`Help > Examples` lists Orca Tracker, Orca Sequencer and Orca Knobs. Name them Tracker, Sequencer
and Knobs, so their links are `?example=tracker`, `?example=sequencer` and `?example=knobs`.

## Acceptance criteria

- [ ] `Help > Examples` lists Tracker, Sequencer and Knobs.
- [ ] `?example=tracker`, `?example=sequencer` and `?example=knobs` open them, and the old
      `orca-` names open nothing.
- [ ] The guides, the website skill and the site's tests use the new names. Each guide still says
      which Learn Orca patch it ports.
