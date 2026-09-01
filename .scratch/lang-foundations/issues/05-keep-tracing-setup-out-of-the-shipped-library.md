# 05 — Keep tracing setup out of the shipped library

**What to build:** Keep the tracing subscriber used by unit tests available to those tests without
shipping its setup code or dependency as part of the ordinary `lang` library.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Subscriber initialization is compiled only for tests.
- [ ] The subscriber dependency is development-only and no runtime dependency remains solely for
      test setup.
- [ ] Tests that opt into trace output retain their current behavior.
- [ ] Native, WASM, locked dependency, and dependency-audit gates pass.
