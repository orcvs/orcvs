# Repair CI and tier the verification gates

**Goal:** Return CI to green, then run the expensive gates on merge instead of on every pull request, without breaking the local and CI equivalence that `docs/tooling.md` promises.

## Why

CI is red and the current workflow definition has no recorded run. The one recorded run failed with `error: The platform you're compiling for is not supported by winit`. `console/Cargo.toml` sets `default-features = false` on `eframe`, which removes the windowing backend, so winit reaches its Linux branch with neither `x11` nor `wayland` enabled. The failure appears only on Linux, which is why every local gate passes on macOS.

Measurement shows where the time goes. All checks together take about 6 seconds with a warm cache. A cold workspace check takes 48 seconds, and a cold test build takes a further 59 seconds after it, because `cargo check` writes metadata and `cargo nextest` needs machine code. Compilation is the whole cost.

That fact sets the shape of the work. A split by check type repeats the same compile in every job and is rejected. A split by trigger removes work from the pull request path and is accepted. The three existing jobs already divide along the only line that avoids repeated compilation: a different target or a different platform.

## Order

Issue 01 first. Nothing else can be measured until one run is green.

## Issues

- `issues/01-restore-the-linux-build.md`
- `issues/02-remove-the-redundant-check-pass.md`
- `issues/03-move-the-slow-gates-to-the-merge-trigger.md`
- `issues/04-measure-the-warm-gate.md`
