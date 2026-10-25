# plughost 0.2.0 evaluation

Research date 2026-10-25. This note answers whether plughost, a Rust host for VST3, CLAP and Audio
Unit plug-ins that runs plug-in code in helper processes, fits Orcvs. The fit assessment and
recommendation come first; the source audit of the four 0.2.0 crates follows and carries the
evidence each fit claim cites.

## Recommendation

**Defer. Do not add plughost to the workspace now, and do not start the guide's Phase 1 spike until
Orcvs has decided it makes sound.** The library is well engineered for what it claims, and its
realtime path holds up in the source (section 3). The obstacles are on the Orcvs side and in the
library's age:

1. **Orcvs has no audio engine to host a plug-in in.** Orcvs is a sequencer: the Playback Engine
   hands each Tick's Output Commands to an output adapter, and the only shipped adapter writes MIDI
   bytes to a port through `midir` (`orcvs/src/playback.rs:89-92`, `console/src/console_midi.rs`).
   No crate opens an audio device, and `docs/research/audio-runtime-terminology.md` reserves "Audio
   engine" for exactly the module Orcvs does not have. plughost leaves the audio device, transport
   clock and routing to the application, so adopting it means adding an audio device backend such
   as `cpal`, an Audio Engine that owns the callback, and a control thread that supervises helpers.
   That is a change of product scope from "Orcvs sends MIDI" to "Orcvs makes sound", which is a
   language and product decision before it is a dependency decision.
2. **It does not compile on two of the three platforms Orcvs builds.** plughost builds only for
   `aarch64-apple-darwin` and `x86_64-pc-windows-msvc` (section 2). The VST3 module stops a Linux or
   wasm32 build with `compile_error!` (`plughost-formats-0.2.0/src/vst3/module.rs:236`). Orcvs's
   pull-request tier runs clippy and nextest on Linux and compiles the WASM target
   (`.github/workflows/test.yml`), and only the merge tier runs macOS. Every use would sit behind a
   target `cfg`, and no pull-request gate would compile the code that uses it.
3. **The library is 25 days old and has one author and no outside users.** Thirteen releases since
   2026-09-30, a breaking 0.2.0 that changed the state format and IPC protocol, exact `=0.2.0` pins
   between host and helper, and about 650 downloads (section 11). Building on it now means tracking
   a moving API by hand, and plug-in state saved by one release may not load in the next.

None of these says plughost is the wrong host. They say the question to settle first is whether
Orcvs should make sound at all. Until that is decided, Orcvs's MIDI can reach any plug-in already:
route the output port to a DAW or plug-in host through the macOS IAC Driver or loopMIDI on Windows.

## How it would fit, if Orcvs decides to make sound

This section records where plughost would sit, so a later decision starts from the seams rather
than from the guide's generic sketch.

**An output adapter in the console, native only.** The plug-in path is one more `OutputAdapter`
beside `MidiOutputAdapter`. The toolkit-free `orcvs` crate already keeps platform MIDI out and
leaves discovery and ports to the console (`orcvs/src/native_midi.rs:1-8`), and ADR 0022 keeps the
core crate free of the UI toolkit. A plug-in adapter belongs in `console` behind
`cfg(any(all(target_os = "macos", target_arch = "aarch64"), all(windows, target_arch = "x86_64")))`,
with the same silent fallback the MIDI backend uses elsewhere.

**The Output Command set maps onto plughost's events without loss.** Note On and Note Off become
plughost's `NoteOn` and `NoteOff`, and Control Change and Pitch Bend travel as three-byte `Midi`
events (section 5). The safety action Orcvs sends on stop, rewind, disconnect and destination
change (All Notes Off, Reset All Controllers, centred Pitch Bend on every channel, `CONTEXT.md`
Playback Engine entry) is 48 messages, far inside the 4096-event block limit. Orcvs's tempo could
fill plughost's `Transport` for tempo-synced plug-ins, which the MIDI path cannot carry today.

**The Tick grid does not survive the seam as it stands.** ADR 0037 holds every Tick to a fixed grid
of deadlines, and the engine carries each Tick's deadline (`orcvs/src/playback.rs:271-300`). But
`OutputAdapter::submit` takes only the commands (`orcvs/src/playback.rs:90`), and it is called
inline on the engine's task once the Tick executes (`orcvs/src/playback.rs:915`). An adapter that
forwards commands to an audio callback can only place them at the start of the next quantum, so
each Tick sounds late by its task wake-up jitter plus up to one quantum, plus the chain's fixed
`latency × quantum` (section 4). At 128 frames and 48 kHz that is under 6 ms in all, which is close
to what the `midir` path already costs, because `midir` also sends on arrival with no timestamp.
Holding the grid sample-accurately would need the deadline to cross the seam and a mapping from the
Tick clock onto the audio device's sample clock. That is a change to the Playback Engine's
interface, so it wants its own ADR.

**Faults fit the diagnostics Orcvs already has.** plughost mutes output on an overflow, a
wrong-sized callback, a failed quantum or a latency change, and stays muted until the control
thread calls `resync_realtime` or `recover_realtime` (sections 3 and 8). An adapter refusal is
already a Playback diagnostic that does not stop later Ticks (ADR 0002, ADR 0055), so a helper
fault would surface the same way. What Orcvs lacks is the control thread that drains plughost's
health and decides to recover; today nothing in the console supervises a child process.

**Editors are the easy part.** Every format's editor opens as a separate window owned by the
helper process (section 9). Nothing embeds into egui, so the console needs only a menu entry that
asks the helper to open or close a slot's editor, and the parenting and lifetime problems the guide
warns about do not arise.

**Packaging is new work.** On macOS the helper must ship as an `LSUIElement` `.app` with Hardened
Runtime and the `disable-library-validation` and `allow-unsigned-executable-memory` entitlements
(section 10). Orcvs has no app bundle, signing or notarisation step today, so a plug-in feature
brings the whole release pipeline with it.

**Dependency policy.** `CLAUDE.md` asks for a recorded rationale for any dependency and a
`mise run audit_deps` pass. The licences are MIT OR Apache-2.0 and the build has no `build.rs`,
bindgen or C++ step (section 1), so the policy is not an obstacle. The `realtime-audit` feature
downloads binaries or builds LLVM at build time and would stay off.

## What would change this recommendation

- A decision that Orcvs makes sound. That opens the Audio Engine design, the seam change above and
  the choice of host, and would be recorded as an ADR.
- plughost reaching a release that keeps its state format and protocol across minor versions, with
  outside users and a second maintainer.
- Linux support upstream, or a decision that the plug-in path is tested only in the merge tier.

If the first happens, the guide's Phase 1 (headless render of a VST3 synth to WAV, state save and
reload) is the right spike, and it should run in a throwaway crate outside the workspace so no
Orcvs gate has to compile plughost before the decision is made.

## Source audit

This audit reads the four published 0.2.0 crates as data, plus the upstream repository at tag
`v0.2.0` (commit `e0ce719f7b693f2cc8009eec488d0fe77876f79c`, recorded in each crate's
`.cargo_vcs_info.json`). Nothing was built or run. Each claim carries one tag:

- **[documented]**: stated in a README, `llms.txt`, rustdoc prose or the changelog only.
- **[code]**: seen in the 0.2.0 crate source.
- **[CI-tested]**: a test exercises it in the upstream `CI` workflow (`.github/workflows/ci.yml`).
- **[unverified]**: inferred or not checkable from the sources available.

The upstream `CI` workflow runs one job, `verify`, on `macos-15` (Apple Silicon) and
`windows-2022`. It runs `scripts/verify-macos.sh` or `scripts/verify-windows.ps1`, which build the
fixture plugins and the helper and then run `cargo test --workspace --locked -- --include-ignored
--skip benchmark`, so the `#[ignore]` tests that need fixtures and a desktop session do run in CI.
It then runs per-feature Clippy passes, `cargo test -p plughost --features realtime-audit --test
realtime_audit -- --include-ignored` with `RTSAN_ENABLE=1`, and `scripts/package-consumer.py`
(ci.yml:21-70, verify-macos.sh:11-20). "CI-tested" below means one of these steps.

### 1. Pinned versions, dependencies, licences, unsafe, protocol

**Versions.** `plughost`, `plughost-core`, `plughost-helper` and `plughost-formats` are all
0.2.0, published 2026-10-24 [documented, crates.io]. Each internal dependency is an exact pin,
`version = "=0.2.0"` (plughost-0.2.0/Cargo.toml, `[dependencies.plughost-core]`), and the README
tells applications to depend with `=0.2.0` too (plughost-0.2.0/README.md:13-17) [code]. All four
use edition 2024 and `rust-version = "1.92"`; the upstream `rust-toolchain.toml` pins `1.92.0`
[code].

**Crate roles.** `plughost` is the application library. It does not depend on `plughost-formats`
at all, except as a dev-dependency, so no plugin-format code links into the application
[code: plughost-0.2.0/Cargo.toml]. `plughost-helper` is the body of a separate executable the
application must build and ship, and it links `plughost-formats` with all three format features
[code: plughost-helper-0.2.0/Cargo.toml; documented: README.md:21-34].

**Key dependencies** (from the manifests and the shipped `Cargo.lock` files) [code]:

| Crate | Dependency | Locked version | Licence | Purpose |
|---|---|---|---|---|
| plughost | interprocess, json5, serde_json, tempfile, walkdir | 2.4.6 (interprocess) | permissive | control socket, scan cache |
| plughost, core (per OS) | rustix, libc, objc2-core-foundation / windows-sys 0.61.2 | | permissive | shm, process groups, Job objects |
| plughost-core | postcard 1.1.3, serde_with, bitflags | 1.1.3 (postcard) | permissive | wire encoding |
| plughost-formats | vst3 0.3.0 | 0.3.0 | MIT OR Apache-2.0 | VST3 COM bindings |
| plughost-formats | clack-host / clack-extensions 0.2.0 | 0.2.0 | MIT OR Apache-2.0 | CLAP host |
| plughost-formats | clap-sys 0.5.0 | 0.5.0 | MIT/Apache-2.0 | CLAP headers |
| plughost-formats (macOS) | objc2 0.6, objc2-audio-toolbox, objc2-avf-audio, objc2-core-audio-kit | | permissive | Audio Units |
| plughost-formats (Windows) | libloading 0.9.0 | | ISC | module loading |
| plughost-core (opt.) | rtsan-standalone 0.3.0 | 0.3.0 | Apache-2.0 | `realtime-audit` only |

The helper's lockfile holds 132 packages and the application's 171, the latter including
dev-dependencies such as `cpal` [code: `grep -c '^name'` over the two lockfiles].

**Native build steps.** None of the four crates has a build script: each manifest says
`build = false` [code]. The `vst3` 0.3.0 crate also has `build = false`, excludes the SDK
(`exclude = ["/vst3sdk"]`), and ships pre-generated bindings in a 670 KB `src/bindings.rs`; its
changelog says users no longer supply a VST3 SDK at build time [code: vst3-0.3.0/Cargo.toml,
vst3-0.3.0/CHANGELOG.md]. No bindgen, CMake or C++ compiler runs in a default build
[code]. The only build-time native step is `rtsan-standalone-sys`, reached only through the
`realtime-audit` feature and only when `RTSAN_ENABLE` is set: its `build.rs` downloads prebuilt
RealtimeSanitizer libraries from GitHub or clones and builds LLVM with `git` and `cmake`
[code: rtsan-standalone-sys-0.3.0/build.rs:7-9, 82, 103-151]. Upstream marks that feature "not
for shipping" (crates/plughost/Cargo.toml:24-25 upstream). The test fixtures need Xcode, CMake and
the VST3 SDK 3.8.1 archive, pinned by SHA-256, but only in a repository checkout
(scripts/build-test-plugins.sh:307-335) [code].

**Licences.** All four crates are `MIT OR Apache-2.0` [code]. Upstream `deny.toml` allows only
0BSD, Apache-2.0, ISC, MIT, Unicode-3.0 and Zlib across all features and dev-dependencies, checked
by `cargo deny ... check licenses bans sources` in CI [CI-tested: verify-macos.sh:20,
deny.toml:23-35]. Advisories are not part of that CI invocation [code]. The README states that the
VST3 SDK headers used by `vst3` and the CLAP headers used by `clap-sys` are MIT-licensed, that a
distributed helper must carry their notices, and that VST is a Steinberg trademark
(plughost-0.2.0/README.md:229-236) [documented]. Which SDK release the `vst3` 0.3.0 bindings were
generated from is not stated in the crate; its 0.2.0 entry mentions SDK 3.8.0 [unverified].

**Unsafe volume** (occurrences of `unsafe {` in `src/`, tests included) [code]:

| Crate | Source lines | `unsafe {` | `unsafe fn/impl/extern` | `SAFETY` comments |
|---|---|---|---|---|
| plughost | 7,983 | 45 | 5 | 26 |
| plughost-core | 12,311 | 54 | 28 | 53 |
| plughost-helper | 5,734 | 71 | 12 | 34 |
| plughost-formats | 18,338 | 429 | 85 | 82 |

Most unsafe code sits in `plughost-formats` (COM, CLAP and Objective-C calls), which runs only in
the helper process. The application-side unsafe is in shared-memory mapping, the doorbell, the
clock and the callback's mailbox (plughost-core-0.2.0/src/ipc/shared/memory.rs,
doorbell.rs, realtime.rs; plughost-0.2.0/src/realtime/mailbox.rs) [code]. No crate sets
`forbid(unsafe_code)` or `undocumented_unsafe_blocks` [code].

**Helper and host must match.** The control channel is length-prefixed postcard with no type
information, and `PROTOCOL_VERSION = 34` (plughost-core-0.2.0/src/ipc.rs:1-3, 21) [code]. The
helper's first message is a `Hello { protocol, token }` whose encoding is fixed across versions;
the application compares `protocol` and fails with `Error::ProtocolVersion` carrying both values,
then kills the helper (plughost-0.2.0/src/helper.rs:164-175) [code]. The shared-memory ring has its
own layout word `FORMAT = 1`, and the helper refuses another (plughost-core-0.2.0/src/ipc/shared/realtime.rs:36-37,
300-302) [code]. The `ipc` module is `#[doc(hidden)]` and exempt from semver
(plughost-core-0.2.0/src/lib.rs:21-22, README) [documented]. In practice the application and the
helper must be built from the same plughost version. `PluginState::to_bytes` is promised readable
by later versions from 0.2.0 on (plughost-core-0.2.0/src/state.rs:91-104) [documented].

### 2. Platforms per format, and wasm32

**Supported targets.** The README and `llms.txt` name exactly two: `aarch64-apple-darwin` on
macOS 15 or newer, and `x86_64-pc-windows-msvc`. Intel Macs, Rosetta, Linux and
cross-architecture bridging are unsupported (plughost-0.2.0/README.md:9-10, 135; llms.txt:31-33)
[documented]. docs.rs builds only those two targets (`[package.metadata.docs.rs]`) [code].

| Format | macOS arm64 | macOS x86_64 | Windows x64 | Linux |
|---|---|---|---|---|
| VST3 | yes [CI-tested] | unsupported [documented] | yes [CI-tested] | `compile_error!` [code] |
| CLAP | yes [CI-tested] | unsupported [documented] | yes [CI-tested] | no platform code [code] |
| Audio Unit (v2/v3) | yes [CI-tested] | unsupported [documented] | n/a | n/a |

The Audio Unit module is gated `#[cfg(all(feature = "au", target_os = "macos"))]`
(plughost-formats-0.2.0/src/lib.rs:25-26), and the VST3 module emits
`compile_error!("plughost supports macOS and Windows")` on any other OS
(plughost-formats-0.2.0/src/vst3/module.rs:235-236) [code]. The x86_64 macOS path has code
(`HOST = Architecture::X86_64` in plughost-0.2.0/src/scan/bundle.rs:120-123) but no CI and no
support statement [unverified].

**macOS minimum.** The realtime doorbell calls `os_sync_wait_on_address` and
`os_sync_wake_by_address_any`, which the module comment says need macOS 14.4 or newer
(plughost-core-0.2.0/src/ipc/shared/doorbell.rs:1-5, 70-73) [code]. These are declared as
ordinary `extern "C"` symbols, not weak-linked, so an application binary linking `plughost-core`
probably fails to load on older macOS rather than failing at runtime [unverified].

**wasm32 and other targets.** The upstream release workflow publishes from macOS because "the
crates do not compile outside macOS and Windows" (release.yml:73-74) [documented]. The code
agrees: `plughost-core` defines `Doorbell::wake`/`sleep` only for `target_os = "macos"` and
`windows` (doorbell.rs:39-59), the clock only for those two (clock.rs:27, 55), and shared memory
only for `unix`/`windows` (memory.rs:10-13, 65); `plughost` has no `cfg` at its crate root
(plughost-0.2.0/src/lib.rs:53-101) and defines `HOST` only for `aarch64` and `x86_64`
(scan/bundle.rs:120-123) [code]. There is no stub or no-op backend. `plughost` therefore fails to
compile on `wasm32-unknown-unknown` and on Linux; the only way to compile it out is to declare it
under a `[target.'cfg(...)'.dependencies]` table and `cfg`-gate every use [code, inferred;
not compiled]. On `x86_64-pc-windows-msvc` it is a supported target [CI-tested].

### 3. RealtimeEndpoint synchronisation and the callback path

**Layout.** `Chain::realtime_endpoint` creates a ring of `latency + 2` lanes (3 to 5), each a
single-quantum shared-memory mapping, plus one control mapping of 64-byte-spaced atomic words and
256-byte per-lane headers (plughost-core-0.2.0/src/ipc/shared/realtime.rs:3-17, 34-65;
plughost-0.2.0/src/chain/realtime.rs:105-121) [code]. Quantum `n` travels in lane `n % lanes`.
Each header's sequence word is written last with `Release` and read with `Acquire`, which hands the
lane between processes; the reader re-checks the sequence after copying to detect a lane rewritten
out of turn (realtime.rs:385-420, 541-616) [code]. Shared memory is an unlinked POSIX `shm_open`
object on macOS and a paging-file section on Windows, sized once so the peer cannot truncate it
(memory.rs:1-5, 78-99) [code].

**Wake mechanism.** There are no semaphores, Mach ports or mutexes on this path. The doorbell is
a 32-bit counter in the control mapping. On macOS the callback does `fetch_add` and
`os_sync_wake_by_address_any(..., SHARED)`; the helper's worker waits with
`os_sync_wait_on_address`. On Windows, where `WaitOnAddress` is per-process, the callback calls
`SetEvent` on an auto-reset event duplicated into the helper, and the worker waits with
`WaitForSingleObject(INFINITE)` (doorbell.rs:24-37, 62-121) [code]. Timestamps use
`mach_absolute_time` or `QueryPerformanceCounter` (clock.rs:27-75) [code].

**What the callback does** (`RealtimeEndpoint::process_audio_f32/f64`,
plughost-0.2.0/src/realtime/endpoint.rs:224-380) [code]: it brackets itself with an `inside` counter the control thread's `halt` spins on
(plughost-0.2.0/src/realtime.rs:239-269); validates the block without allocating; copies input,
automation, events and a postcard-encoded `Transport` (via a 176-byte stack buffer) into the
lane; stamps the clock, publishes the sequence and rings the doorbell (core realtime.rs:220-237,
385-420); reads the output of quantum `sequence - latency` if published, never waiting for it; and
on a new generation swaps in a replacement ring or dry-delay buffer through a lock-free
`AtomicPtr` mailbox whose old value the control thread frees (mailbox.rs:1-95).

**Allocation and locks.** The callback allocates nothing and takes no lock in the code paths read
[code]. Its storage (`last: Vec<f64>`, the dry delay, `Notes` for 256 notes) is allocated when the
endpoint is created (endpoint.rs:161-192; notes.rs:9) [code]. The caller must pass
`Events::for_block()` for `produced`, which preallocates 4,096 records and 64 KiB of SysEx
(plughost-core-0.2.0/src/event/events.rs:50-58) [code]. The integration test
`the_realtime_callback_allocates_nothing` and `..._across_a_recovery`
(crates/plughost/tests/realtime_audit.rs:56-58, 207-209 upstream) count allocations through a
global allocator on both platforms and run RealtimeSanitizer on macOS, covering stopped, priming,
processing, dry fallback, missed outputs, overflow, size mismatch, resync and recovery
[CI-tested: ci.yml:65-68, "Audit the realtime callback"].

**Syscalls.** The README says the callback "never waits, locks, or allocates"
(README.md:117) [documented]. It does make one non-blocking system call per callback: the doorbell
wake (`os_sync_wake_by_address_any`, a `ulock` wake, or `SetEvent`) in `Ring::submit` via
`Control::ring` (realtime.rs:418, 205-208; doorbell.rs:26-29) [code]. The audit test's comment
says RTSan aborts on "any ... system call" (realtime_audit.rs:1-6), so the CI pass implies RTSan
does not intercept these wake calls rather than that none happen [unverified]. The callback can
block only through the wake syscall itself; the worker side blocks freely [code].

**Worker scheduling.** The helper's worker takes the macOS time-constraint policy for the
quantum, creates and joins its own `AudioWorkIntervalCreate` workgroup (not the device's), holds an
App Nap-preventing activity, or joins MMCSS "Pro Audio" on Windows; refusals go to diagnostics
(plughost-helper-0.2.0/src/process/realtime/schedule.rs:1-6, 56-120) [code].

**Deadline miss.** When the output due this callback is not published, the callback counts
`missed` and plays the `Fallback`: `Silence` (default) or `Dry`, the input delayed by the whole
latency so it lines up with processed output (plughost-0.2.0/src/realtime.rs:21-30;
endpoint.rs:369-373) [code]. It ramps from the last played sample into the fallback over one
quantum, and crossfades back over one quantum on the next processed output (endpoint.rs:410-462)
[code]. It never repeats the previous block [code]. The helper counts a quantum as `late` when it
completes more than `latency * quantum` of wall time after submission; that is the only deadline
value, set by `start_generation` (chain/realtime.rs:152-158; realtime.rs:475-477) [code].

**Hard faults stop the stream until the application acts.** A full ring (helper `latency + 2`
quanta behind), a callback whose frame count differs from `quantum`, invalid input, a failed
quantum, a protocol error or a latency change all set `resync`; the endpoint then plays the
fallback on every callback until the control thread calls `Chain::resync_realtime`,
`recover_realtime`, or stop/reset/start. Nothing resynchronises automatically
(endpoint.rs:288-329; plughost-0.2.0/src/realtime.rs:88-93; chain/realtime.rs:172-185) [code].
Tests `a_callback_of_another_size_resyncs_until_the_endpoint_restarts` and
`a_stalled_plugin_misses_overflows_and_recovers_after_a_restart` cover this
(crates/plughost/tests/realtime.rs:350, 386 upstream) [CI-tested].

**Timeouts.** The callback has none. Defaults are `Timeouts { load: 300 s, control: 60 s,
process: 10 s }`, helper start 30 s and exit 2 s (chain.rs:42-50; helper.rs:28-30); hang detection
is left to the application through `RealtimeStatus::stalled` (realtime.rs:119-126) [code].

**Measured cost.** The benchmark guide reports, for 0.1.3 and 0.1.4 on local hardware, no late
quanta at 64 and 128 frames on Apple Silicon and 64/128/480 on Windows, with callback p99.9 of
35 to 80 µs and a worst single callback of 104 µs (macOS) and 340 µs (Windows)
(crates/plughost/benches/README.md:225-240 upstream) [documented]. The `realtime` bench is not in
CI and no figure is given for 0.2.0 [code: ci.yml].

### 4. Added latency per chain

The endpoint adds `latency * quantum` frames, with `latency` in 1..=3 quanta (default 1), on top
of the plugins' reported latency; `start_realtime` returns the sum
(plughost-0.2.0/src/realtime.rs:39-41; chain/realtime.rs:33-36, 74-76, 139-170) [code]. With the
default this is one device buffer: 128 frames at 128-frame callbacks, 2.67 ms at 48 kHz. This is
in addition to the device's own input and output buffering [code, inferred]. The design requires
every callback to be exactly `quantum` frames; an application whose device delivers several
quanta per callback needs `latency` of that many quanta (endpoint.rs:56-60) [documented].
Plugin latency compensation within one chain is the plugins' own; offline rendering removes the
reported latency (README.md:73-74) [documented]. Output equals offline output `latency` quanta
later in tests `clap_/vst3_realtime_output_is_the_offline_output_latency_quanta_later` and
`paced_callbacks_play_the_offline_output` (tests/realtime.rs:111, 118, 492 upstream)
[CI-tested]. One chain is one helper process; multiple chains each add their own latency in
parallel, not in series [code, inferred].

### 5. Events, limits and transport

`Event<'a> { offset: usize, port: usize, data: EventData<'a> }`, with `EventData` one of
`Midi([u8; 3])`, `SysEx(&[u8])` (F0 to F7), `NoteOn(Note)`, `NoteOff(Note)` and
`Expression(NoteExpression)` (plughost-core-0.2.0/src/event.rs:51-87) [code]. `Note` holds
`id: Option<u32>` (at most `i32::MAX`), `channel`, `key` and `velocity: f64`; expressions are
Volume, Pan, Tuning, Vibrato, Expression, Brightness and Pressure (event.rs:91-160) [code].
Offsets are sample offsets from the block start (or render start offline), in non-decreasing
order and inside the block (event.rs:43-53; plughost-0.2.0/src/realtime/endpoint.rs:209-211)
[code]. Events are not latency-aligned (chain/audio.rs:332-335) [documented].

Per-block limits: `MAX_BLOCK_EVENTS = 4096` records, counting automation points too and summed
across slots, and `MAX_BLOCK_SYSEX_BYTES = 64 KiB` (event.rs:10-20) [code]. Ports that take MIDI
only get the MIDI 1.0 form and lose note IDs and non-pressure expressions (event.rs:62-72)
[documented]. Over-limit input fails the block; in realtime it is an error that also forces a
resync; output events beyond `produced`'s room count as `dropped_events`
(realtime.rs:111-115; endpoint.rs:296-310) [code]. On every new generation the endpoint emits a
note off for each note it saw started, up to 256 tracked notes (endpoint.rs:268-276; notes.rs:9)
[code]. Fault corpora for shared output events (late, excessive, wrong port, non-finite velocity,
SysEx outside the area) ship in `plughost-core-0.2.0/src/ipc/corpus/shared_output/` [code].

Transport is `BlockContext { frames, transport: Option<Transport> }`, where `Transport` carries
`sample_position: i64`, `beat_position`, `bar_position`, `tempo`, `time_signature`, `playing`
and `loop_region`, all optional except position and playing (transport.rs:76-96, 199-218)
[code]. There is no recording flag [code]. In realtime the transport travels in each lane header
(realtime.rs:123-128, 398) [code]. `tests/events.rs` upstream holds 13 integration tests
[CI-tested, contents not reviewed per test].

### 6. Offline rendering API

All three functions are generic over any `Process<S>` implementor, which `Chain` is
(plughost-core-0.2.0/src/render.rs:102-160) [code]:

```rust
pub fn render<S: Sample, P: Process<S>>(processor: &mut P, input: &[&[S]], frames: usize,
    events: &Events, options: &RenderOptions) -> Result<Rendered<S>, P::Error>;
pub fn render_with_schedule<S: Sample, P: Process<S>>(processor: &mut P, input: &[&[S]],
    frames: usize, events: &Events, options: &RenderOptions, schedule: &RenderSchedule<'_>)
    -> Result<Rendered<S>, P::Error>;
pub fn render_stream<S: Sample, P: Process<S>>(processor: &mut P, request: RenderInput<'_, S>,
    options: &RenderOptions, consume: impl FnMut(Delivery<'_, S>),
    cancelled: impl FnMut() -> bool) -> Result<RenderProgress, P::Error>;
```

(render.rs:204-210, 225-232, 336-342) [code]. `Rendered` holds `channels: Vec<Vec<S>>` aligned
with the input (`frames + tail` each), `events`, the removed `latency`, `tail` and a
`RenderStatus` (render.rs:178-191) [code]. `RenderOptions` defaults to
`TailPolicy::UntilSilence { threshold: 1.585e-5 (−96 dBFS), hold_seconds: 1.0 }` with
`max_tail_seconds: 30.0` (render.rs:51-67) [code]. `RenderSchedule` carries `TransportChange`s,
`AutomationEvent`s and `AutomationRamp`s; blocks split at transport changes and the event budget
(render/timeline.rs:8-77; render.rs:220-223) [code]. `render_stream` keeps memory bounded by the
block size plus the silence hold and checks `cancelled` before each block (render.rs:326-335)
[code]. `RenderSession::{new, push, finish}` supports segment-by-segment rendering
(render/session.rs:25-100) [code]. Through `Chain` each block is one IPC round trip over shared
memory, which the guide measures at 10 to 60 µs fixed cost per block
(benches/README.md:140-143 upstream) [documented].

### 7. Scanner

`Scanner::new(helper, cache)` uses the platform's standard directories, `jobs = 1`,
`stall_timeout = 30 s` and excludes a bundle after 2 consecutive failures
(plughost-0.2.0/src/scan/mod.rs:236-247) [code].

- `discover()` / `discover_controlled(&ScanControl, &dyn Fn(&ScanEvent) + Sync) -> Catalog`
  never loads plugin code. It uses unchanged cache entries, VST3 `moduleinfo.json` and Mach-O/PE
  headers, writes no cache, and lists Audio Units from the registry through a helper without
  instantiating them (mod.rs:294-327) [code].
- `scan(&dyn Fn(&BundleScan) + Sync) -> Result<Catalog, Error>` / `scan_controlled` loads each
  bundle in its own short-lived helper and writes changed results atomically before announcing
  them (mod.rs:329-345; scan/cache.rs:1-6) [code].
- `retry(path)` rescans one bundle with a 24-hour stall timeout, for licence dialogs
  (mod.rs:220-228, 371-377) [code].
- `ScanControl::cancel()` stops admissions and terminates active helpers; `ScanItem::skip()` skips
  one item; `ScanStage` progress renews the stall deadline (scan/control.rs:10-133) [code].
- The cache is JSON with `SCHEMA = 3`, keyed by path, with file stamps and the host architecture;
  0.1 caches read as empty (scan/cache.rs:23-40; CHANGELOG.md:13-14) [code].
- `Catalog::resolve(&PluginId)` maps a stored identity back to a `PluginRef` (mod.rs:155-160)
  [code].

The upstream integration test target `scan_scheduling` (custom harness) and corpus tests for
moduleinfo, binary headers and cache files cover this [CI-tested by name; contents not reviewed].

### 8. Failure and recovery

Each `Chain` owns one helper process. On macOS the helper runs in its own process group, which
plughost kills with `SIGKILL`; on Windows it runs in a kill-on-close Job object
(plughost-0.2.0/src/helper/process/scope.rs:5-40, 71-79) [code]. The helper polls its parent every
20 ms and kills its group when the parent goes (plughost-helper-0.2.0/src/lifetime.rs:4-27) [code].
After a timeout, protocol error or cancellation the helper is terminated and later calls return
`Error::Crashed` until recovery (llms.txt:750-753) [documented]. `HelperMonitor` observes exit
without IPC and reports `end_reason()`; `ChainCanceller::cancel()` kills a hung helper from any
thread (helper/process.rs:11-25; llms.txt:756-799) [code, documented].

- `Chain::snapshot(&mut self) -> Result<ChainSnapshot, Error>` saves each slot's plugin, project
  state and native bypass, and works while the endpoint runs; plughost never snapshots by itself
  (chain/recovery.rs:18-23) [code].
- `Chain::recover(&self, slots: &[ChainSlot]) -> Result<Chain, Error>` starts a new helper with
  the same identity and timeouts and reapplies the saved audio configuration. The old chain is
  left as it was (chain.rs:167-188) [code].
- `Chain::recover_realtime(&mut self, slots) -> Result<Chain, Error>` does the same and moves the
  existing `RealtimeEndpoint` to the new chain through the mailbox, so the audio callback keeps
  its value; the old helper is not contacted (chain/recovery.rs:35-55) [code].

What survives: the slot list, each slot's project state at snapshot time, native bypass (only
where a VST3/CLAP bypass parameter exists, never for AU), the audio configuration, host identity,
timeouts and the endpoint with its counters. Edits after the snapshot, DSP history and tails, open
editors and unread diagnostics are lost (chain.rs:178-181; llms.txt:777-784) [code, documented].
Tests `a_crashed_chain_recovers_from_its_last_snapshot` and related ones
(tests/recovery.rs upstream, 4 tests) [CI-tested]. The guide reports 29 crashes recovered in a
median 177 ms and 10 hangs in 156 ms in a 0.1.4 local run (benches/README.md:230-234)
[documented].

### 9. Native editor support

`Chain::open_editor(slot)`, `close_editor(slot)` and `editor_open(slot) -> Result<bool, Error>`
send requests to the helper (chain.rs:397-414) [code]. The helper opens a top-level window of its
own per editor; editors are "not embedded in the application's windows"
(plughost-helper-0.2.0/src/editor.rs:1-2) [code]. The macOS path requires the helper's main
thread (`MainThreadMarker`) and an `NSWindow`; the Windows path uses a Win32 window with per-monitor
DPI awareness (editor.rs:4-40; editor/windows.rs:1-20) [code]. VST3, CLAP and AU editors are
implemented in `plughost-formats` (vst3/editor.rs, clap/plugin.rs:520-580, au/plugin/editor.rs)
for macOS and Windows; a CLAP editor that cannot embed floats (llms.txt:722-723) [code,
documented]. There is no `raw-window-handle`, `winit` or `egui` dependency, and the application
side has no way to pass a parent window: the in-process `HostedPlugin::open_editor` takes a raw
parent pointer only for direct `plughost-formats` users (plughost-formats-0.2.0/src/hosted.rs:179-183)
[code]. Opening an editor brings the helper to the front (llms.txt:729-731) [documented]. The
upstream example `examples/editor.rs` and an ignored test that "opens a window" exist and run in
CI's desktop session [CI-tested per verify scripts; individual test not reviewed].

### 10. macOS packaging

The helper must be a background `LSUIElement` `.app` signed with Hardened Runtime and two
entitlements, `com.apple.security.cs.disable-library-validation` and
`com.apple.security.cs.allow-unsigned-executable-memory` (README.md:36-41;
scripts/macos/helper.entitlements upstream) [documented, code]. The application passes the
executable inside `Contents/MacOS`, not the bundle (llms.txt:36-37) [documented]. The upstream
`package-app.sh` ad-hoc signs (`codesign --sign - --options runtime --entitlements ...`) and can
copy dylibs next to the executable (scripts/macos/package-app.sh:1-26) [code]. Distribution
signing and notarisation are the application's job; the script does neither (README.md:39-41)
[documented]. On Windows the helper is a plain `.exe` started with `CREATE_NO_WINDOW`
(scope.rs:11-15) [code]. Both platforms need the helper path at runtime for `Scanner` and `Chain`
[code].

### 11. Tests, CI and project health

**Tests.** The published sources hold 39 (`plughost`), 79 (`plughost-core`), 21
(`plughost-helper`) and 80 (`plughost-formats`) `#[test]` functions, 16 of them `#[ignore]` for
fixtures or benchmarks [code]. Upstream integration tests under `crates/plughost/tests/` are not
in the package; the ones fetched are `realtime.rs` (9), `recovery.rs` (4), `events.rs` (13),
`presets.rs` (2), `realtime_audit.rs` (2), plus custom-harness `shared_completion` and
`scan_scheduling` [code, upstream]. Other test files may exist [unverified]. Plugins under test are
repository fixtures (`delay` with about 27 fault variants such as crash, hang, stall and latency
change; `synth`; `presets`; `routing`), six VST3 SDK example plugins, and Apple system Audio Units;
no commercial plugins (scripts/build-test-plugins.sh:1-17; README.md:184-185) [code, documented].
Parser corpora (vstpreset, AU plist, moduleinfo, binary headers, cache, IPC framing) ship in the
crates; fuzzing via `cargo-fuzz` is local only [code, documented]. There is no proptest [code].

**CI matrix.** `macos-15` and `windows-2022` only, with the steps listed at the top of this
section; no Linux, Intel macOS or wasm job, and the benchmarks are local only (ci.yml:21-70) [code].

**Project metadata** (fetched 2026-10-25). The first release, 0.0.1, went to crates.io on
2026-09-30, and 0.2.0 is the 13th release in 25 days (0.0.1 to 0.0.5, 0.1.0 to 0.1.6, 0.2.0)
[crates.io, CHANGELOG.md]. All-time downloads are 170 (`plughost`), 339 (`plughost-core`), 234
(`plughost-formats`) and 143 (`plughost-helper`); no crate outside the family depends on any of
them; one crates.io user, `emulette`, owns them [crates.io]. Releases use trusted publishing from
GitHub Actions (run 40082580706, the tagged SHA) [crates.io]. The repository page shows 83
commits, 0 stars and 0 open issues [GitHub web]. The GitHub API and commit history were not
reachable from this environment, so contributor count and commit cadence are unverified.

### 12. Public API surface

From `plughost-0.2.0/src/lib.rs:61-101` and the files named [code]:

```rust
// Scanner (scan/mod.rs:200-384): builder methods host, directories, policy, jobs, stall_timeout
pub fn new(helper: &Path, cache: &Path) -> Scanner;
pub fn discover(&self) -> Catalog;   // and discover_controlled(&ScanControl, &dyn Fn(&ScanEvent) + Sync)
pub fn scan(&self, progress: &(dyn Fn(&BundleScan) + Sync)) -> Result<Catalog, Error>;
pub fn retry(&self, path: &Path) -> Result<BundleScan, Error>;
// Catalog { bundles: Vec<BundleScan>, audio_units: ScanOutcome }; plugins(), resolve(&PluginId)

// Chain (chain.rs, chain/*.rs)
pub struct SpawnOptions { pub host: HostIdentity, pub timeouts: Timeouts } // non_exhaustive
pub struct Timeouts { pub load: Duration, pub control: Duration, pub process: Duration }
pub struct ChainSlot { pub plugin: PluginRef, pub state: Option<PluginState>, pub bypass: Option<bool> }
pub fn spawn(helper: &Path, slots: &[ChainSlot], options: &SpawnOptions) -> Result<Chain, Error>;
pub fn main_bus_config(&mut self, sample_rate: f64, max_block_size: usize, input: Layout,
    outputs: &[Layout]) -> Result<RoutedChainConfig, Error>;
pub fn prepare_audio(&mut self, config: &RoutedChainConfig) -> Result<Vec<Vec<AudioBusInfo>>, Error>;
pub fn process_audio_f32(&mut self, context: &BlockContext, input: &[&[f32]],
    output: &mut [&mut [f32]], automation: &[AutomationEvent], events: &Events,
    produced: &mut Events) -> Result<(), Error>;                  // and _f64
pub fn realtime_endpoint(&mut self, config: &RealtimeConfig) -> Result<RealtimeEndpoint, Error>;
pub fn start_realtime(&mut self) -> Result<u32, Error>;           // also resync_realtime
pub fn stop_realtime(&mut self) -> Result<(), Error>;
pub fn realtime_status(&self) -> Option<RealtimeStatus>;
pub fn snapshot(&mut self) -> Result<ChainSnapshot, Error>;
pub fn recover(&self, slots: &[ChainSlot]) -> Result<Chain, Error>;
pub fn recover_realtime(&mut self, slots: &[ChainSlot]) -> Result<Chain, Error>;
pub fn save_state(&mut self, slot: usize, purpose: StatePurpose) -> Result<PluginState, Error>;
pub fn restore_state(&mut self, slot: usize, state: &PluginState, purpose: StatePurpose)
    -> Result<(), Error>;
pub fn open_editor(&mut self, slot: usize) -> Result<(), Error>;
pub fn helper_monitor(&self) -> HelperMonitor;                    // and canceller() -> ChainCanceller
// plus parameters, presets, buses, bypass, reset, latency, tail and take_* drains

// RealtimeEndpoint (realtime.rs, realtime/endpoint.rs)
pub struct RealtimeConfig { pub quantum: usize, pub latency: usize, pub fallback: Fallback }
// RealtimeConfig::new(quantum), .with_latency(1..=3), .with_fallback(Fallback::{Silence, Dry})
pub fn process_audio_f32(&mut self, context: &BlockContext, input: &[&[f32]],
    output: &mut [&mut [f32]], automation: &[AutomationEvent], events: &Events,
    produced: &mut Events) -> Result<RealtimeOutput, Error>;      // and _f64
// RealtimeOutput::{Processed, Fallback(FallbackReason::{Stopped, Priming, Missed, Resync})}

// Identity and state (plughost-core plugin.rs, state.rs)
pub fn PluginId::new(format: PluginFormat, class_id: impl Into<String>) -> PluginId; // serde, Hash
pub fn PluginRef::vst3(bundle: impl Into<PathBuf>, class_id: impl Into<String>) -> PluginRef;
pub fn PluginRef::clap(bundle: impl Into<PathBuf>, class_id: impl Into<String>) -> PluginRef;
pub fn PluginRef::audio_unit(class_id: impl Into<String>) -> PluginRef;            // no serde
pub fn PluginState::to_bytes(&self) -> Vec<u8>;
pub fn PluginState::from_bytes(bytes: &[u8]) -> Result<PluginState, InputError>;
```

`PluginId` serialises with serde as `{format: "vst3" | "audio-unit" | "clap", class_id}` and
canonicalises VST3 and AU IDs to upper case (plugin.rs:50-128; CHANGELOG.md:40-45) [code,
documented]. `PluginState` is capped at `MAX_STATE_BYTES` (128 MiB) and has no serde form; store
`to_bytes()` (state.rs:14-27, 91-104) [code]. Many public types are `#[non_exhaustive]` and the
in-process traits `HostedPlugin`, `BlockProcessor` and `Sample` are sealed (CHANGELOG.md:10-11,
41-44) [documented].

## Risk register

| Risk | Evidence | Severity | What would retire it |
|---|---|---|---|
| Does not compile on wasm32 or Linux; no stub backend | doorbell.rs:39-59, clock.rs:27-55, memory.rs:10-13, scan/bundle.rs:120-123; vst3/module.rs:235-236; release.yml:73-74 | High | Target-gated dependency plus `cfg` on every use, proven by a wasm32 and Linux `cargo check` |
| Project is about four weeks old, single owner, 0 stars, about 650 total downloads, no external reverse deps | crates.io API; GitHub page (83 commits) | High | Months of releases without breaking changes, more maintainers, outside users |
| API churn: 13 releases in 25 days, 0.2.0 broke state format, protocol and many signatures | CHANGELOG.md:8-60 | High | A 0.2.x line with only additive changes, or vendoring a pinned copy |
| Exact version lock between app, helper and protocol 34; mismatch fails at handshake | ipc.rs:21; helper.rs:164-175; README.md:34 | Medium | Build helper and app from one lockfile in one workspace, checked in CI |
| Callback makes one wake syscall per quantum, contrary to the "no syscalls" reading of the audit comment | realtime.rs:418; doorbell.rs:26-29, 78-81, 113-116 | Medium | Own measurement of the wake cost at p99.9, or upstream confirmation that RTSan intercepts it |
| A fault (size mismatch, overflow, failed quantum) mutes the stream until the control thread calls `resync_realtime` | endpoint.rs:288-329; chain/realtime.rs:172-185 | Medium | A supervising control loop that polls `realtime_status` and resyncs, with a test |
| Callbacks must be exactly `quantum` frames; variable device buffers force resyncs | endpoint.rs:291-295; endpoint.rs:56-60 | Medium | A fixed-size adapter in front of the endpoint at the cost of extra latency |
| Added latency of 1 to 3 quanta on top of plugin latency, per chain | realtime.rs:39-41; chain/realtime.rs:33-36 | Medium | Accepting it in the latency budget, or in-process hosting via plughost-formats |
| Strong link to `os_sync_wait_on_address` (macOS 14.4+) may stop the app from launching on older macOS | doorbell.rs:1-5, 70-73; README.md:9 | Medium | Raising the app's minimum macOS to 15, or a launch test on 14.x |
| Intel macOS unsupported and untested | README.md:135; ci.yml:26-32 | Medium | CI on x86_64-apple-darwin, or accepting arm64-only Mac support |
| Editors are separate helper windows only; no embedding, no raw-window-handle | plughost-helper editor.rs:1-2; chain.rs:397-414 | Medium | Accepting floating windows, or upstream embedding through a parent handle |
| macOS helper needs Hardened Runtime entitlements that weaken library validation; notarisation is the app's job | README.md:36-41; helper.entitlements | Medium | A signed and notarised helper build in the app's release pipeline |
| Large unsafe surface (about 600 blocks), mostly FFI in plughost-formats | unsafe counts above | Low (isolated in helper) | Fuzzing and Miri where possible; relying on process isolation for plugin faults |
| Realtime performance figures are for 0.1.3 and 0.1.4, local hardware, not CI | benches/README.md:225-240; ci.yml | Low | Running the `realtime` bench gate against 0.2.0 on target hardware |
| `realtime-audit` feature builds LLVM or downloads binaries at build time | rtsan-standalone-sys build.rs:82-151 | Low | Never enabling the feature outside a dedicated audit job |
| VST3 SDK provenance of the vst3 0.3.0 bindings is not stated | vst3-0.3.0/CHANGELOG.md | Low | Confirming upstream which SDK tag generated `bindings.rs` and that its licence text is MIT |

## Primary sources

- Crate sources from crates.io, read locally: `plughost-0.2.0/`, `plughost-core-0.2.0/`,
  `plughost-helper-0.2.0/`, `plughost-formats-0.2.0/` (files as cited above), plus
  `vst3-0.3.0/` (`Cargo.toml`, `README.md`, `CHANGELOG.md`) and `rtsan-standalone-sys-0.3.0/build.rs`.
- Upstream at tag `v0.2.0`, commit `e0ce719f7b693f2cc8009eec488d0fe77876f79c`, under
  https://github.com/emulette/plughost/blob/v0.2.0/ : `.github/workflows/ci.yml`,
  `.github/workflows/release.yml`, `llms.txt`, `CHANGELOG.md`, `Cargo.toml`,
  `rust-toolchain.toml`, `deny.toml`, `crates/plughost/Cargo.toml`,
  `crates/plughost/tests/{realtime_audit,realtime,recovery,events,presets}.rs`,
  `crates/plughost/benches/README.md`, `scripts/{verify-macos.sh,verify-windows.ps1,build-test-plugins.sh}`,
  `scripts/macos/{package-app.sh,helper.entitlements}`.
- Repository page https://github.com/emulette/plughost and its issues tab (commit count, stars,
  open issues), fetched 2026-10-25.
- crates.io API, fetched 2026-10-25: `https://crates.io/api/v1/crates/<name>`,
  `.../reverse_dependencies` and `.../owners` for the four crates, and the crate records of
  `vst3`, `clap-sys`, `clack-host` and `rtsan-standalone-sys`.
