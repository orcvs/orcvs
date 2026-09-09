# 01 — Name the native MIDI decision once

**What to build:** One place in `orcvs` that decides whether a native MIDI backend exists on the target and what a running Orcvs therefore uses for output, so that nothing else — no other `orcvs` module, and no consuming crate — restates that condition. The console still discovers and selects MIDI destinations exactly as it does today; it just asks `orcvs` what backend it has instead of deriving it from the target operating system.

This is a refactor with no behaviour change. It exists because a consuming crate cannot test an `orcvs` feature: `feature = "native-midi"` inside the shell names the shell's own feature. Without a seam the shell has to mirror the flag and keep the mirror in sync forever.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Whether a native MIDI backend is available is decided in exactly one place in `orcvs` source; no other module or crate spells out the equivalent target condition for MIDI. The `midir` target table in `orcvs/Cargo.toml` is the one remaining spelling of the condition, which the "No manifest, dependency, or feature change" item below forbids this issue from touching; issue 02 replaces it with the `native-midi` feature.
- [x] `orcvs` exposes a default output adapter and a MIDI backend type that name a valid type on every supported target, including WASM.
- [x] The console obtains its running Orcvs and its MIDI device selection through that seam and carries no target condition of its own for MIDI.
- [x] Native MIDI destination discovery, selection, all-notes-off safety, and delivery-failure reporting behave exactly as before on macOS, Windows, and Linux.
- [x] On a target with no native MIDI backend the console still starts and plays silently, as it does today.
- [x] No manifest, dependency, or feature change.
- [ ] `mise run check` passes.

## Comments

### The seam

`orcvs/src/native_midi.rs` is the one place. It holds a `#[cfg(any(target_os = "macos",
target_os = "windows", target_os = "linux"))]` / `#[cfg(not(...))]` pair over two inline
`backend` modules, and that pair is the whole of the decision. Each module answers the same
three things — `NativeMidiBackend`, `AVAILABLE`, and, through the parent module,
`NativeMidiOutputAdapter` and `output_adapter()` — so every other module names an answer
instead of restating the question.

The target with no native service gets a real `MidiBackend` rather than a hole: it offers no
destination and refuses to connect, and `MidiOutputAdapter` holding no connection accepts
every submission and delivers nothing. That is what removed the condition from `app.rs`,
`playback.rs`, `lib.rs`, `shell/src/lib.rs`, and `shell/src/console.rs`: `Orcvs` has one
default adapter type on every target, `MidiSelectionHandle` exists on every target, and the
console holds one `MidiDeviceSelection<NativeMidiBackend>` unconditionally. The console asks
`native_midi::AVAILABLE` for the two places where behaviour genuinely differs — whether the
MIDI menu exists, and whether a Playback failure has a status line to reach or only the log.

### The silent backend is compiled and tested on every target

`SilentMidiBackend` — the implementation the `cfg(not(...))` arm re-exports as
`NativeMidiBackend` — lives in a third module, `mod silent`, gated
`#[cfg(any(test, not(any(target_os = "macos", target_os = "windows", target_os = "linux"))))]`.

That third `cfg` is deliberate and is not a fourth spelling of the decision: it widens the
non-native arm by `test`, so the code that ships on a target with no MIDI service is the code
the ordinary `cargo nextest run --package orcvs` pass exercises on macOS and Linux. The
non-native `mod backend` is then a re-export, not a second implementation, so there is one
body under test.

Without this the item above — the console starts and plays silently — was ticked against
three assertions that no tier executed. Native CI runs on macOS and Linux, where the module
was `cfg`-ed out; `check_wasm` compiles the wasm target but runs nothing; `test_wasm` runs
`wasm-pack test … shell --test wasm`, the shell crate's integration target only. The three
tests now run in-tree at `native_midi.rs:178`, `:185`, and `:196`.

Grep proving the count, over workspace source:

```
$ grep -rn -e 'target_os' -e 'native-midi' --include='*.rs' --include='*.toml' lang orcvs shell Cargo.toml
orcvs/Cargo.toml:29:[target.'cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))'.dependencies]
orcvs/src/native_midi.rs:17://! `native-midi` feature; until then it is the one restatement this module
orcvs/src/native_midi.rs:62:#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
orcvs/src/native_midi.rs:120:#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
orcvs/src/native_midi.rs:142:    not(any(target_os = "macos", target_os = "windows", target_os = "linux"))
```

Line 17 is prose in the module doc-comment naming the manifest remainder; line 142 is the
`test`-widened arm above. The `orcvs/Cargo.toml` line is the `midir` target table. It is a
manifest, which this issue forbids changing, and issue 02 replaces it with the `native-midi`
feature.

### `mise run check`

Left unticked deliberately. The repository contract defers `mise run check` to CI and says not
to run it to pass a gate. What was run instead: `cargo fmt --all -- --check`, crate-scoped
clippy and `cargo nextest run` for `orcvs` and `shell`, the workspace clippy, nextest, and
doctest passes, and `mise run check_wasm`. The crate-scoped `orcvs` pass is now 289 tests,
which includes the three silent-backend tests that previously compiled without running.
