# 03 — Name the running instance `Orcvs`

**What to build:** Move `app.rs` into the `orcvs` crate and rename `App` to `Orcvs`. Add a
CONTEXT.md entry that defines the running instance, and keep "Orcvs" as the name of the system
everywhere else.

**Blocked by:** 02 — Replace the egui input types in the application state.

**Status:** ready-for-agent

- [ ] `App` is named `Orcvs` at every use site, including `console.rs` and the tests.
- [ ] The type lives in the `orcvs` crate.
- [ ] CONTEXT.md defines the running instance and lists what it owns.
- [ ] The CONTEXT.md entry states no implementation detail and names no type field.
- [ ] The entry carries an `_Avoid_` line, as every other entry does.
- [ ] The existing doctest on the type still passes.

## Comments

`App` names nothing, and it already collides: `console.rs` writes `impl eframe::App for Console` and
`use crate::app::App` in the same file. After the split, `shell` holds both.

`regex::Regex` is the precedent for naming a crate's central type after the crate. The alternatives
— `Session`, `Instance`, `Machine` — all import a word from outside the Orcvs domain, and CONTEXT.md
has been kept deliberately closed.

The cost is real: CONTEXT.md uses "Orcvs" for the language and the system throughout, as in "Orcvs
language rules" and "the Orcvs host application". The new entry must make the two meanings
distinguishable rather than leave the reader to guess.

Do not add the entry before this rename lands. A glossary that describes a type which does not exist
is the problem this effort is trying to avoid.
