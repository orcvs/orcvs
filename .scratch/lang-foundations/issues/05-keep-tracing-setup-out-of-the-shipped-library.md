# 05 — Keep tracing setup out of the shipped library

**What to build:** Keep the tracing subscriber used by unit tests available to those tests without shipping its setup code or dependency as part of the ordinary `lang` library.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** Improvement

`orcvs` already carries the shape this asks for: its helper is behind a test gate and its subscriber is a development dependency. Copy that, rather than inventing an arrangement.

This ticket briefly recorded a `Blocked by` edge on issue 10, on the belief that the console took the subscriber's `fmt` feature from `lang` and would fail to build once this landed. That belief was false — the console's own `ansi` feature always implied `fmt`. The edge is removed. See issue 10's comments.

- [x] Subscriber initialization and its one-time guard compile only for tests, and the dead-code suppressions that currently hide them are gone with them.
- [x] The subscriber dependency is development-only, and no runtime dependency remains solely for test setup.
- [x] Tests that opt into trace output retain their current behavior.
- [x] The subscriber and the crates beneath it no longer appear in the library's normal dependency graph on either target.
- [x] Native, WASM, locked dependency, and dependency-audit gates pass.
