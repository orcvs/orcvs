# 02 — Pin the site's sound module to the synth's controller table

**What to build:** A check that fails when `site/src/console/sound-module.ts` and the `synth`
crate disagree about a controller number, a load value, the acid channel's overrides, or what Reset
All Controllers clears.

**Status:** needs-triage

**Blocked by:** 01

**Why:** `sound-module.ts` hand-mirrors the table (`EFFECTS`, the acid `CU`/`DC` overrides at
`0x30`, `ACID = 0x8`), and `site/tests/sound-module.spec.ts` asserts the TypeScript against itself.
Nothing ties either to the crate.

**Constraint:** ADR 0087 keeps the WASM module free of imports and glue, so this is a build-time
artefact or a contract test, not a runtime export.

- [ ] Decide between a generated artefact the site reads and a contract test over both sources.
- [ ] A changed controller number or load value in `synth` fails a gate until the site follows.
