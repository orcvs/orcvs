# 18 — Repair the persistence feature

**What to build:** Make `cargo build --features persistence` compile, and keep it compiling. The feature is wired into `Cargo.toml` and ten types carry `#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]`, but serde is never imported anywhere, so every one of those derives fails to resolve. Nothing in CI builds the feature, so it broke silently and stayed broken.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] `cargo build --features persistence` compiles with no errors.
- [ ] `Grid` carries the feature's derive, so the `Source` and `App` fields that hold one can be derived at all.
- [ ] Every other field type a `cfg_attr` derive reaches either carries the derive or is deliberately excluded, decided type by type and written down — a runtime handle is not state worth persisting.
- [ ] CI builds the feature, so the next break is a red build rather than a discovery months later.
- [ ] A round-trip test: a Source serialized and deserialized holds the same Cells and the same shape.

## Notes

Found by review, not yet triaged by a human.

Two separate failures stacked on top of each other, both verified on this branch:

**First layer — the derive macros are not in scope.** `cargo build --features persistence` reports 20 errors, all `cannot find derive macro Serialize` / `Deserialize`, across `opts.rs` (2 sites), `glyph.rs` (2), `console.rs` (1), `app.rs` (1), `source/expression_map.rs` (3) and `source/source.rs` (1). None of those files imports serde. The reviewer counted 21; on this branch it is 20 errors from 10 derive sites.

**Second layer — the field types have no impls.** Adding `#[cfg(feature = "persistence")] use serde::{Deserialize, Serialize};` to those six files clears the first layer and exposes 29 errors of the form `the trait bound X: Serialize is not satisfied`, for `Grid`, `Cursor`, `SourceCommander`, `CancellationToken`, `Bpm`, and `ArrayVec<Atom, 32>` — the `Atoms` behind `Source::parsed`. `Grid` fails at two derive sites, `App::grid` and `Source::grid`. So repairing the feature is not a matter of adding six imports; it is a decision, per type, about what is state and what is a live handle. `CancellationToken` and `SourceCommander` are plainly the latter.

**What is pre-existing and what this branch added.** The missing imports are on `main` unchanged, and so is the un-derived `Grid` reachable through `App::grid` — `App` has held a `pub grid: Grid` all along. What this branch added is a second site: `Source` used to hold `opts: Opts`, and `Opts` does carry the `cfg_attr` derive; it now holds `grid: Grid`, which does not. Repairing the feature therefore now requires deriving on `Grid` where before it required deriving on `Grid` for `App` alone. The gap is not new; this branch widened it.

**Why it rotted.** `.github/workflows/test.yml` never mentions the feature. The build job runs `cargo build --profile ci --workspace` and `cargo clippy --workspace --no-deps --tests`; the test job runs `cargo build --profile ci --workspace` then nextest per package. All default features. Nothing in CI has ever compiled this code, which is how ten derives ended up written against an import that was never there.

**Worth asking before repairing.** No caller reads the feature and no issue depends on it. Deleting the feature and its ten `cfg_attr` lines is a smaller, honest change than making them all compile, and it would leave nothing to rot. If persistence is genuinely wanted, repair it and add it to CI; if it is aspirational, remove it. Either answer closes this ticket — what should not stand is ten derives that have never compiled.

## Comments

**2026-08-26 — filed (agent)**

Filed from a review note on the issue 10 branch. Both claims verified before filing: the 20-error build, and the `Grid` bound that appears once the imports are added.
