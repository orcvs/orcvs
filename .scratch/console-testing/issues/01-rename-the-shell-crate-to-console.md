# 01 — Rename the shell crate to console

**What to build:** Rename the `shell` crate to `console` so the code uses the word the glossary already
uses, and remove the two placeholder tests that occupy the crate's test surface.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `shell/` becomes `console/`, and `[package] name`, `[lib] name`, `[[bin]] name` and both `path`
      fields follow.
- [ ] The workspace `members` list in the root `Cargo.toml` names `console`.
- [ ] `Trunk.toml`, `index.html`, `check.sh`, `.typos.toml`, `.gitignore` and `assets/` move with the crate.
- [ ] `mise.toml` follows: the `test_shell` task and its `ts` alias, `check_wasm`'s `cd shell`, and
      `test_wasm`'s `wasm-pack test --headless --firefox shell --test wasm` line.
- [ ] `scripts/check-tooling-contract.sh` is updated in the same change, because it pins the
      `test_wasm` run line byte-identically and will otherwise fail.
- [ ] `.github/workflows/test.yml` is checked; it calls mise tasks only, so it is expected to need no change.
- [ ] `console/src/lib.rs`'s `test_something` is deleted along with the `trace` helper it calls. It
      logs `"etc"` and asserts nothing.
- [ ] `console/tests/app_test.rs` is deleted. It contains only a commented-out `test_terminator`.
      `tests/common/mod.rs` goes with it: `tests/wasm.rs` does not reference it.
- [ ] Path citations in `restyle-egui-console/02` that name `shell/src/style.rs` and
      `shell/src/console.rs` are updated to the new crate.
- [ ] `mise run check` passes.

## Comments

The glossary already speaks this way. `CONTEXT.md` uses "console" as a plain noun inside the
definitions of Orcvs, Cursor, Marker, and Render Frame, and defines it nowhere. Meanwhile
Application Command Function lists "shell command" under `_Avoid_`, so the crate is named after a
word the glossary warns off.

Do this before the test work rather than after. The tests added by `03`, `04` and `05` bake crate
paths into new files, and renaming afterwards means a second sweep across them.

`feat/egui-theming` is not the source of this rename and is not a prerequisite. That branch is an
ancestor of the current code, not a pending redesign: its `console/src/` still holds `app.rs`,
`grid.rs`, `playback.rs` and `render_frame.rs`, from before `crate-boundaries` extracted the `orcvs`
crate, and its merge base with `main` is from 2024-11-12.
