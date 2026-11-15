# 05 — Decide teardown after a refused delivery

**Status:** needs-triage

**What to decide:** ADR 0016 says teardown after a refused delivery ends all Timed and Mono ownership.
The code keeps the schedule, which records only what an output adapter accepted, so a refused
submission is retried at the next executed Tick
(`a_refused_submission_leaves_the_schedule_standing_for_the_next_tick`). `docs/requirements/playback.md`
lists the difference under PLAY-260 and "Open decisions" (decisions D9).

**Done when:** an ADR amendment records the retry, or the code is changed to follow ADR 0016.
