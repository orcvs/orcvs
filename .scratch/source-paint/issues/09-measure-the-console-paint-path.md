# 09 — Measure the console paint path

**What to build:** `07` claims that the cost of painting a Render Frame follows the viewport rather than the Source. `CLAUDE.md` asks for a benchmark for any path whose cost a change claims to move, and `console` has no bench target at all.

**Blocked by:** 07

**Status:** ready-for-agent

### Why this is filed rather than folded into `07`

`source-grid-rendering/05` made the same claim and deliberately added no benchmark: "No frame time is asserted and no benchmark is added — epaint already discards off-screen shapes at tessellation, so what this saves is Cell iteration and Shape construction, which is what the tests count." That reasoning was sound for what `05` could reach. It is no longer the best available, because `06` produced a layer that is measurable without a toolkit: `Paint::derive` takes a `RenderFrame` and a Position range and nothing else — no `Orcvs`, no `egui::Context`, no window — which is precisely the property that makes a criterion bench of it honest.

The counting tests in `07` stay whichever way this lands. They assert the shape of the claim — work follows the viewport — and they do it deterministically. A benchmark says what it costs, which is a different question, and `.scratch/benchmarks/spec.md` is the reason it is a separate ticket: the comparison lives in the action, so a local run produces a number that decides nothing, and adding the target is the only part of this that is local work.

### The measurement

- [ ] `console/benches/paint.rs`, criterion, `harness = false`, with `[lib] bench = false` on the crate the way `orcvs` and `lang` already set it — `mise run bench` passes `--benches` and `--output-format bencher`, and a unit-test harness reaching that flag fails the task before criterion runs.
- [ ] Measure `Paint::derive` at the fitted range and at a culled one, over Grid sizes wide enough to separate "follows the Source" from "follows the viewport". `orcvs/benches/source.rs`'s `FRAME_SIZES` is the existing precedent for which shapes and why; follow it rather than inventing a second set.
- [ ] `SourceShapes::new` is the other half and needs a `GlyphTable`, which needs an `egui::Context`. Decide whether that is worth a bench at all: if the Context cost swamps what is being measured, measure `Paint::derive` and `Paint::background_runs` alone and say in the bench's own doc comment why the shape step is absent. Do not build a benchmark whose number is mostly harness.
- [ ] The bench's doc comment states what the number is for, the way `orcvs/benches/source.rs` does throughout. The gate alerts at 150% and fails at 300%, comparing runs across hosted runners, so it cannot see a change smaller than tens of per cent — a benchmark added here is a regression alarm, not a measurement anyone reads off.

### The wiring, which is the part that is easy to leave half-done

- [ ] `mise.toml`'s `bench` task takes `--package console`. It names its packages explicitly today.
- [ ] `.github/workflows/bench.yml` adds `console/**` to the `paths` filter of **both** the `push` and the `pull_request` trigger. The two lists are duplicated because Actions does not support YAML anchors, and the comment above them says so — a change to one that misses the other is silent.
- [ ] That comment enumerates what each glob covers. Extend it rather than leaving `console/**` unexplained beside entries that each state their reason.
- [x] The merge tier's `test_persistence` runs `nextest` with `-E 'not kind(bench)'` after `e32c1ba`, so a new bench target does not return the merge queue to the hang PR #78 fixed. Confirm that is still the case rather than assuming it. Confirmed on this branch and on `origin/main` (`aeac29b` / PR #78); the filter is not work this effort lands.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package console --locked
bash scripts/check-tooling-contract.sh
actionlint
zizmor --offline .github/workflows
```

`mise run audit_deps`, because a `[[bench]]` target and criterion as a `console` dev-dependency are a manifest and feature change.

Deferred to CI and named on the `Not run` line: `mise run bench` itself. `.scratch/benchmarks/spec.md` states outright that the comparison lives in the action, so a local run produces a number that decides nothing. Run it only to confirm the target builds and criterion accepts the flags, and say that is why.
