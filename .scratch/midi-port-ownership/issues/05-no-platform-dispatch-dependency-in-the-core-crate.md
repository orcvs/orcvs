# 05 — The toolkit-free crate carries no platform dispatch dependency

**What to build:** The build that promises to carry neither the MIDI library nor a system audio
dependency actually carries neither. The platform dispatch binding the deleted bridge needed is
gone from the manifest, and the tooling check that enforces the promise catches the class of
platform binding rather than the enumerated list of crate names it happens to know — so the next
one cannot slip past it the way this one did.

**Blocked by:** 03.

**Status:** resolved

- [x] The toolkit-free crate declares no platform dispatch dependency, unconditionally or otherwise.
- [x] Building that crate without default features produces a dependency tree with no platform MIDI
      or system audio binding, on macOS as well as on the other targets.
- [x] The tooling contract check matches the class of platform binding rather than a fixed list of
      crate names, and fails when a new one is introduced without a recorded rationale.
- [x] A rationale is recorded for any platform dependency that remains, per the repository's
      dependency policy.
- [x] The tooling-contract script's own tests cover the new matching, and the workflow linters pass.
