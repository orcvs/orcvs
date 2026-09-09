# 02 — Put midir behind a native-midi feature

**What to build:** A `native-midi` feature on `orcvs` that gates the `midir` dependency and the native MIDI backend, on by default so every build that exists today resolves identically. Turning it off gives a running Orcvs that composes, executes, and renders Source exactly as before but produces no MIDI output, and whose dependency tree contains no `midir` and no system audio library. The shipped console keeps its MIDI: it enables the feature explicitly for its native targets.

Feature name is `native-midi`, not `midi`: the target-agnostic MIDI vocabulary — destinations, backends, output adapters, Play Commands — stays present either way. What the feature gates is the one backend that talks to the platform.

**Blocked by:** 01 — Name the native MIDI decision once.

**Status:** resolved

**Tags:** release/v1

- [x] `orcvs` declares a `native-midi` feature, on by default, and `midir` is an optional dependency reached only through it.
- [x] With the feature enabled, dependency resolution and the public surface are unchanged from today, and so is behaviour but for one connect-error string that no enabled build can reach (see *What landed*).
- [x] With the feature disabled, `orcvs` builds with neither `midir` nor a system audio library anywhere in its dependency tree, and a running Orcvs uses an output adapter that emits nothing.
- [x] The console enables the feature for its native targets, so the shipped application still lists and selects MIDI destinations.
- [x] The WASM target and its browser regressions are unaffected in either feature state.
- [x] Verification exercises the feature disabled, enabled, and crossed with `persistence`, rather than assuming the two compose. Standing tier coverage is narrower than the one-off run: it compiles all four cells and runs three, the (disabled, `persistence`) cell being compiled only (see *Feature crossings*).
- [x] Tooling checks and documentation record the feature, its default, and what disabling it gives up.
- [x] `mise run check_wasm` and `mise run audit_deps` pass here; `mise run check` and `mise run test_persistence` are Deferred to CI by the repository contract and are the pull-request and merge tiers' to run (see *The four commands in the last item*).

## Comments

### Release freeze

Recorded during the `release/v1` issue alignment on 2026-09-04, and revised on 2026-09-09. This
issue now carries the `release/v1` tag and blocks `v1-release/03`, so it must land **before**
`v1-release/03` cuts the candidate SHA — never inside the window that runs from the cut until
`v1-release/01` records the GO decision.

The reason is evidence, not behaviour. `v1-release/03` records a `cargo deny --locked check` result
that describes one dependency tree. Making `midir` optional changes that tree, so a merge inside the
window leaves the recorded result describing a build that is no longer the candidate. Listing this
issue as a blocker of `v1-release/03` is what enforces that: the SHA cannot be cut until it is
resolved.

### What landed

Recorded 2026-09-09.

The feature is `native-midi`, declared in `orcvs/Cargo.toml` and **on by default**
(`default = ["native-midi"]`, `native-midi = ["dep:midir"]`). `midir` is now `optional = true` and
stays in its `cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))` table,
because the feature and the table answer different questions: the feature says whether this build
wants a native backend, the table says where one could exist. Dropping the table would have put
`midir` into a default-featured WASM resolution, which is the one thing this must not change.

What disabling it gives up is delivery, and nothing else. A running Orcvs still composes, executes,
and renders Source, still has a `MidiOutputAdapter`, and still exposes every MIDI item it did — the
adapter simply holds `SilentMidiBackend`, which offers no destination, refuses to connect, and
accepts every submission without sending anything. `native_midi::AVAILABLE` is then `false`, so the
console presents no MIDI menu, exactly as it already does on a target with no MIDI service.
`orcvs/src/native_midi.rs` states both halves of that where each belongs: `AVAILABLE` is a `const`,
so a feature-off build carries `const _: () = assert!(!AVAILABLE)` and every feature-off compilation
checks it — the browser build included; the adapter's behaviour is behaviour, so it is a test, run
by the tier's `--no-default-features` pass. Clippy's `assertions_on_constants` is what found the
first draft asserting a constant inside a `#[test]`, in the very pass this change added.

The console asks for the feature by name rather than leaning on the default: `shell/Cargo.toml`
takes `orcvs` with `default-features = false` in the plain table and re-enables `native-midi` in
`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. So the shipped desktop application
keeps its destination list even if `orcvs`'s default ever changes, and the browser build asks for no
native backend at all — which is the feature state it already had.

The WASM leg, which `.scratch/v1-roadmap-wayfinding/issues/07` asked this ticket to state: it uses
default features. `mise run check_wasm` clippies the workspace for `wasm32-unknown-unknown` with
each member's defaults on, so `orcvs` is compiled there with `native-midi` enabled, and `trunk
build` builds the shell binary whose `orcvs` dependency has defaults off. Both compile the silent
arm, because the target half of the condition is false either way, and neither resolves `midir`.
Nothing about the browser build changes in either feature state. The browser regressions themselves
run in `mise run test_wasm`, which the contract defers to CI; what this checkout ran is
`mise run check_wasm`, which compiles those test targets for both feature states of `orcvs` on
`wasm32-unknown-unknown` and builds the application twice.

`Cargo.lock` is unchanged: an optional dependency is still a dependency of the workspace, and the
enabled tree is byte-identical to the tree before this change.

The only behaviour difference in an enabled build is a message. `SilentMidiBackend`'s connect error
now reads "this build has no native MIDI backend" rather than "this target ..."; the cause can now
be a feature rather than only a target. It is unreachable in an enabled build, since `AVAILABLE`
gates the only path that connects.

### The disabled dependency tree

The claim is checked rather than described. `mise run audit_deps` resolves the feature-off tree and
fails if `midir` or a system audio library is still in it, and `scripts/check-tooling-contract.sh`
pins those lines along with the manifest shape — `midir` optional, `midir` in its target table and
not in the plain one, `default = ["native-midi"]`, and the console's two declarations.
`docs/tooling.md` records the feature, its default, and what turning it off gives up.

The exact commands, run on macOS (`aarch64-apple-darwin`):

```
$ cargo tree --package orcvs --edges normal --prefix none --locked | grep -E '^(midir|alsa|coremidi|coremidi-sys)'
midir v0.11.0
coremidi v0.9.2
coremidi-sys v3.2.1

$ cargo tree --package orcvs --no-default-features --edges normal --prefix none --locked | grep -E '^(midir|alsa|coremidi|coremidi-sys)'
$ echo $?
1
```

The second command prints nothing: `midir`, `coremidi`, `coremidi-sys`, `core-foundation`,
`core-foundation-sys`, `objc2`, `block2`, `libc`, and `log` all leave the tree with the feature off.
`cargo tree` resolves the host target, so the same two lines check ALSA on the Linux runner and
CoreMIDI on the macOS one, which is why the guard lives in `audit_deps` and not in a comment.

### Feature crossings

The feature was exercised in both states and crossed with `persistence` rather than assumed to
compose — clippy and `cargo nextest run` for `orcvs` at all four cells, plus `shell` in both
persistence states and the `orcvs` doctests with the feature off. `check_pull_request` now carries
two of those crossings permanently: `cargo clippy --package orcvs --all-targets
--no-default-features --features persistence` and `cargo nextest run --package orcvs
--no-default-features`. The scope is `orcvs` because it is the only crate the feature reaches, and
the plain pass is the one that runs, because the tests that state what disabling the feature gives
up are compiled only with it off.

### The four commands in the last item

`mise run check_wasm` and `mise run audit_deps` were run and passed. `mise run check` and
`mise run test_persistence` are Deferred to CI by the repository contract, which says not to run
them to pass a gate. The item once named all four and was left unticked against the two this
checkout must not run, which put a `resolved` issue in front of the roadmap planner showing seven
of eight criteria met — a discrepancy a reader has to reconstruct from a comment. The criterion now
names what each side actually owes, so the tick means what it says and the deferral is on the line
rather than under it. What ran in their place is in the report: `cargo fmt`, crate-scoped
clippy and `cargo nextest run` for `orcvs` and `shell`, the four feature cells above, the tooling
contract and its own fixture suite, and the roadmap planner.

### What review found

Recorded 2026-09-09, from three independent reviews of the branch diff.

Two of the new contract assertions did not hold what their own comments claimed, and both were
demonstrated rather than argued. The console's native declaration was pinned by its feature list
alone, but Cargo unions a dependency's declarations and honours `default-features = false` only if
every one of them says it — so dropping it from
`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` put `orcvs feature "default"` back into
the console's native build while the contract stayed green, which is precisely the borrowed default
naming the feature exists to prevent. The native table now asserts it too. The same line pinned the
feature list as exactly `["native-midi"]`, so `["native-midi", "persistence"]` — a manifest this
contract has every reason to accept, `shell` already having that feature — was rejected with a
message reading as though `native-midi` were absent; it is now pinned by what the list contains.

The `audit_deps` guard was pinned at `^if printf` and nothing else, so a half-finished rename left
the pinned assignment in place, unused, and piped an unset name into the grep. An empty string
matches nothing, so the audit could no longer fail on a `midir` regression while all four pinned
lines still matched. The assertion now names the variable, stopping before the `-E` pattern that
awk cannot carry.

The feature-off test asserted only that `submit` and `safety_reset` return `Ok`, which its sibling
in `silent` already asserts over a backend it constructs itself. The claim only this test can make
is the wiring — that `output_adapter()` reaches the silent backend — and that was the part left
unstated: a feature-off `backend` re-exporting something that forwarded commands and returned `Ok`
passed it. Confirmed by writing that backend and watching the old test pass. It now asserts the
pairing by its consequences, no destination and no obtainable connection, and fails against that
mutant.

Four assertions had no fixture scenario in a suite where every other one does — the WASM target
table, `native-midi = ["dep:midir"]`, the audit's `exit 1`, and the two repairs above. All now do.

Two records were wrong rather than under-pinned: `docs/tooling.md` and the `miri` task comment both
stated the ALSA link unconditionally, which this change made conditional, and three acceptance
criteria above overstated what holds — behaviour is not quite unchanged in an enabled build, the
standing tier runs three of four feature cells rather than four, and the last criterion named two
commands the repository contract forbids running here, leaving a `resolved` issue showing seven of
eight met.

### Land before the candidate SHA

The "Release freeze" comment above applies to this change as written. Making `midir` optional
changes the dependency tree `cargo deny --locked check` sees, so this must land **before**
`v1-release/03` cuts the candidate SHA — never inside the window between the cut and
`v1-release/01`'s GO decision, where it would leave the recorded audit describing a build that is
no longer the candidate.
