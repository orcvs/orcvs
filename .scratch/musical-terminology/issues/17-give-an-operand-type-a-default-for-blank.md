# 17 — Give an operand type a default for blank

**What to build:** A Function can declare an operand whose blank Cells mean a fixed value, so a
reader can leave an optional setting unwritten and get its default.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] An operand type, beside ADR 0108's `Optional<O>`, decodes two blank Cells as a value the
      Function table declares, and decodes written Cells as the underlying type.
- [ ] Blank Cells are treated alike however they got blank (left empty, emptied by a Portal write,
      or returned by a nested Function), as ADR 0107 and 0108 require of a blank note.
- [ ] Source Paint, Info and the Language Map show an unwritten defaulted operand as valid.
- [ ] A new ADR records the decision and its relation to ADR 0108.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked -- -D
      warnings`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo nextest run --package lang --locked` and `cargo nextest run --package orcvs --locked`
      pass, and `mise run site_build` and `mise run site_test` pass.

## Comments

Decided in `decisions.md` as the prerequisite for Monophonic Play's `legato` operand (ticket 18).
