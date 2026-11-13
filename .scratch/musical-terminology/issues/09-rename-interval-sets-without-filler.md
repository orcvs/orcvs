# 09 — Rename Interval Sets without the filler

**What to build:** Every Interval Set name is two meaningful characters with no `_`. A musician
writes a C major triad as `C4MA`, a C dominant seventh as `C407`, and picks from the major scale
as `io`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Triads: `M_` → `MA`, `m_` → `mi`, `o_` → `di`, `+_` → `au`.
- [ ] Scales: major `MA` → `io` (Ionian), natural minor `mi` → `ae` (Aeolian), major pentatonic
      `p5` → `pM`, minor pentatonic `p3` → `pm`.
- [ ] Dominants: `7_` → `07`, `9_` → `09`. `b9` and `13` are unchanged.
- [ ] No Interval Set name contains `_`, and the uniqueness check across Interval Sets, Voicings
      and Functions still holds.
- [ ] Stack answers the new names (a stacked major triad answers `MA`).
- [ ] ADR 0105 is amended: the filler rule is gone, the naming rules cover the triad, mode-name and
      pentatonic choices, and the case pairs `mM`/`mm` and `pM`/`pm` are stated as accepted.
- [ ] The glossary, the Function reference, examples, site lessons and tests use only the new names.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md`. The Language is pre-release, so old names are not kept as aliases.
