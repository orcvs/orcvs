# 12 — Discard autosaves stored under the old spellings

Status: resolved
Blocked by: 02 — Read a pair in a direction; 05 — Write a value in a direction or at a Position; 06 — Push a value into an indexed pair; 07 — Accept the Read, Write and Copy families

**What to build:**

ADR 0070's proposed default for Sources written with the old spellings, if ticket 07 confirms it. A Source File has no version to check (ADR 0054), so Sources are read under the current grammar. Three respellings silently change what a stored spelling means: the `&` arrows become Reads (ticket 02), `@<` becomes the west Write (ticket 05) and `@t` becomes Push (ticket 06). Once all three have landed, the console discards its stored autosave once, with the report ADR 0054 gives a Source it cannot load, so no developer's autosave is reinterpreted as writes it never asked for. If ticket 07 settles a different answer, this ticket is rewritten to match or closed as wontfix.

## Acceptance criteria

- [x] An autosave stored before this change is discarded on the first launch after it and reported, and the next save writes over it.
- [x] An autosave stored after this change loads unchanged.
- [x] The persistence tests cover both, alongside the existing discarded-rather-than-migrated test.
- [x] Source Files opened through File > Open are not affected.

## Resolution

The console stores its Source under a new key, `orcvs_source_rwc` (`console::persistence::SOURCE_KEY`). The key is the marker, so the Source File format stays unversioned (ADR 0054). A start that finds nothing under it but a value under the old `orcvs_source` key (`STALE_SOURCE_KEY`) reports it through `crate::report::error!`, the same channel that reports an undecodable Source, and starts the empty Grid. The next save writes the new key and removes the old one, so the report happens once. eframe hands `App::new` read-only storage, so the removal has to wait for the first save. Values under the new key load unchanged. File > Open reads Source Files directly and never touches storage keys. Tests: `persistence::stored_source_tests::a_source_stored_under_the_stale_key_is_discarded_once_and_reported`, `a_source_stored_under_the_current_key_loads_unchanged`, and `console::storage_tests::a_stale_source_is_discarded_and_removed_by_the_next_save`, alongside `a_source_stored_at_a_previous_default_grid_is_discarded_rather_than_migrated`.
