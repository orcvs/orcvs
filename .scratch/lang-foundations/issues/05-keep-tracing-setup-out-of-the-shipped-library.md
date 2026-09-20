# 05 — Keep tracing setup out of the shipped library

**What to build:** Keep the tracing subscriber used by unit tests available to those tests without shipping its setup code or dependency as part of the ordinary `lang` library.

**Blocked by:** 10 — Declare the tracing subscriber's formatting feature where it is used.

**Status:** ready-for-agent

**Tags:** Improvement

`orcvs` already carries the shape this asks for: its helper is behind a test gate and its subscriber is a development dependency. Copy that, rather than inventing an arrangement.

- [ ] Subscriber initialization and its one-time guard compile only for tests, and the dead-code suppressions that currently hide them are gone with them.
- [ ] The subscriber dependency is development-only, and no runtime dependency remains solely for test setup.
- [ ] Tests that opt into trace output retain their current behavior.
- [ ] The subscriber and the crates beneath it no longer appear in the library's normal dependency graph on either target.
- [ ] Native, WASM, locked dependency, and dependency-audit gates pass.
