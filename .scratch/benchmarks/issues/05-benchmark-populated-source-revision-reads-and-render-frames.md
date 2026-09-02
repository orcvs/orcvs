# 05 — Benchmark populated Source revision reads and Render Frames

**What to build:** Extend the performance gate so maintainers can measure the cost of repeatedly reading an unchanged populated Source revision and deriving its application-facing Render Frame, catching per-frame allocation or traversal regressions in the same benchmark history used for language performance.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Criterion measures unchanged `SourceCommander::read_revision()` calls over representative populated Source sizes without including fixture construction in the measured iteration.
- [x] Criterion measures `Orcvs::render_frame()` over the same representative populated Source sizes through the public application entry point.
- [x] Benchmarked Source includes dense valid Language Units and realistic incomplete or invalid live-edit content.
- [x] Benchmark results make growth across Source sizes visible and prevent a fixed-size case from hiding whole-map work.
- [x] The standard local benchmark command emits the new measurements in the CI-compatible output format.
- [x] Benchmark CI runs when Orcvs Source, Language Map, Render Frame, or benchmark configuration changes and compares the new measurements with the stored series.
- [x] Tooling checks and documentation describe Criterion as covering both language execution and populated Source rendering paths.
- [x] The scoped Rust gates and `mise run bench` pass.

## Comments

`orcvs/benches/source.rs` measures three Source shapes — 16x16, 32x32, and 64x64. The console opens at 32x32, and each step changes the Cell count fourfold, so whole-map work reads off the series as growth rather than hiding inside one number. It does: `source_read_revision` is 22/41/83 ns and `source_render_frame` is 1.7/5.8/22.8 µs across the three.

The fixture tiles the Expression shapes an editing session actually holds — complete arithmetic, a nested Expression, a Bang, Activation Characters, a malformed operand, and a half-typed Expression — from a different starting point on each row, separated by spaces so a row holds several extents rather than one, and cut to the column count wherever that lands. The cut is not a defect: a row that ends mid-Expression is exactly what a live edit leaves behind.

`Orcvs` has no public Grid accessor, and only the Grid that owns a Position mints one, so the Render Frame is how the fixture obtains the Positions it selects and writes through. That keeps the benchmark on the same public path the shell uses.

`--benches` in the `mise` task is load-bearing twice over. Without it `cargo bench` also runs the `orcvs` integration-test harnesses, which reject `--output-format`. With it, cargo still selects the library — `--benches` means every target with `bench = true`, and that includes the lib — so `orcvs` needs the same `[lib] bench = false` that `lang` already carries. Both failures were found by running the command, not by reading the flag's description.

Benchmarking `orcvs` puts `midir` in the bench jobs' build, and `midir` links ALSA on Linux. `test.yml` already installs `libasound2-dev`; `bench.yml` did not, so both bench jobs would have failed building `alsa-sys` on the first push that touched `orcvs/**` and the series would have stopped growing silently. The step is now in both jobs and asserted by the contract.

`populated_app` checks that its writes landed. `Orcvs::write` reports a rejected edit through `tracing` and returns nothing, and a bench binary installs no subscriber, so an unchecked fixture failure would be measured as an empty Grid and reported as a plausible Render Frame number.

The action's stored series keeps `name: lang`. The name was chosen before `orcvs` was benchmarked and now reads oddly, but it is how the action finds the history on `gh-pages`: renaming it abandons every point already stored. `spec.md` records that.

Fixing the contract script was unplanned. `assert_not_contains lang/Cargo.toml '^criterion([.]workspace)?='` was meant to forbid a workspace-level criterion but matched the plain dev-dependency issue 01 added, so `mise run check` and the contract's own test suite were both red before this work started. The assertions now say what was intended: criterion is a plain versioned dev-dependency of `lang` and `orcvs` and of nothing else.
