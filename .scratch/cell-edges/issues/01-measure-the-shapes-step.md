# 01 — Measure the shapes step

**What to build:** A benchmark of the step that turns a Paint into Shapes, at the same Grid sizes and viewport cases `paint_derive` uses (fitted, culled, single, empty), so the edge work after it can be compared against today's per-Cell borders. `paint_derive` measures the derivation only. See `../spec.md` and ADR 0078.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A Criterion benchmark builds the Source Shapes from a derived Paint at each existing benchmark size and viewport case, and joins the CI benchmark series and its floors as the existing ones do (`.scratch/benchmarks/spec.md`).
- [ ] It reaches the shapes step without a shipped item that only the benchmark uses: no `pub` widened for it, no parameter or branch only it populates. If that is not possible from the bench crate, the ticket records why and what it measures instead.
- [ ] Tessellation is not measured; the ticket's notes say so.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass. The comparison itself is CI's.
