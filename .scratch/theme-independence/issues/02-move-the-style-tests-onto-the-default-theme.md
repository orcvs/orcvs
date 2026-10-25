# 02 — Move the style tests onto the default Theme

**What to build:** The style module's tests check Cell visuals, border composition and chrome without naming a built-in. Each test takes the default Theme for its appearance and overrides only the fields its behaviour depends on. Expected colours come from that Theme's fields. Tests that check one built-in's chrome against the Theme record become one check over every built-in the registry holds. The byte baseline frozen from Okabe–Ito's composed visuals is deleted, because field-reading tests cover the behaviour it guarded.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** resolved

- [x] The style module's tests name no built-in Theme function or identity.
- [x] The chrome-record check runs over every built-in the registry holds and reads each one's table in the Theme record.
- [x] `okabe_ito_reproduces_the_pre_refactor_cell_visuals_exactly` is gone, and the behaviours it covered (unclaimed, claimed, Portal, Locked, Diagnostic, Cursor and Region states) each have a test that reads the Theme's fields.
- [x] Where the default sets a needed channel transparent, or sets two channels alike, the test overrides those fields and says why.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.

## Comments

`okabe_ito_chrome_matches_the_decided_record` and `orcvs_light_chrome_matches_the_decided_record` became `every_built_in_chrome_reads_its_theme_keys`, which runs over every built-in the registry holds. Okabe–Ito has no property table in `theme.md`'s format, so until 07 the check reads each built-in's fields rather than its table. 07 adds the table read.

Removed: `caret_border_change_is_visible_but_restrained` pinned one palette's selection-border delta (80–120). The defaults don't satisfy it, so it isn't a property every built-in has. Removed: `okabe_ito_reproduces_the_pre_refactor_cell_visuals_exactly`. Existing field-reading tests cover each of its states: placement crossing, Portal, Locked, Diagnostic, Cursor and Region.

Overrides: `transparent_borders()` clears the role, Diagnostic and Output Portal border colours, because the dark default draws role borders that outrank the border a test asks about. Cursor tests clear `cursor.foreground`, because the dark default recolours the Cursor's glyph.
