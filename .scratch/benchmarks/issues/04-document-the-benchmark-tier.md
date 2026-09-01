# 04 — Document the benchmark tier

**What to build:** The tooling and agent docs describe the benchmark gate, including the one promise it breaks.

**Blocked by:** 03 — Gate merges on the benchmark series.

**Status:** resolved

- [x] `docs/tooling.md` lists the benchmark tier alongside the pull-request and merge tiers.
- [x] It states that this is the one gate whose verdict CI owns and a local run cannot reproduce, and why.
- [x] `AGENTS.md` names `mise run bench` in the performance risk gate.
- [x] `CONTEXT.md` is unchanged.

## Comments

`CONTEXT.md` is a glossary of the Orcvs domain and holds no implementation detail. A Benchmark is tooling, like nextest or cargo-deny, and belongs in `docs/tooling.md`.

`docs/tooling.md` currently promises that `mise.toml` defines the commands used both locally and in CI so the same checkout executes the same gates. The measurement command keeps that promise; the comparison does not, because it lives in the action. Write the exception down rather than leave it to be discovered.
