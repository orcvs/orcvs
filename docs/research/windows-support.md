# Windows support for Orcvs: research

Research date: 2026-10-24, against `86f40c6` on `main`.
Every external claim cites a primary source in the Sources list at the end. Anything marked
**UNVERIFIED** could not be confirmed from a primary source.

## Summary

- **What Windows support should mean for v1.** The native console builds, starts, plays, opens
  and saves files, and sends MIDI to a real or loopback port on Windows 11 x64. Ship an MSVC
  build only. OSC/UDP and audio are already outside v1 scope
  (`.scratch/v1-release/definition-of-done.md:87`), so Windows adds nothing to them now. The notes
  below cover what will matter when they arrive.
- **MIDI is mostly fine, and the code already covers Windows.** midir 0.11 uses WinMM by default.
  It works for output, but has no virtual ports on Windows, so today's loopback test cannot run
  there. Windows MIDI Services (in-box on Windows 11 from February 2026) makes every port
  multi-client and adds loopback endpoints that can be created by script. Both reach WinMM apps
  like midir without code changes.
- **The real risk is timing, not IO.** tokio's timer runs at millisecond granularity, and Windows
  can be coarser, especially when the window is minimized. Playback ticks on
  `tokio::time::sleep_until`, so expect more tick jitter on Windows. Measure it before claiming
  parity.
- **GitHub has Windows runners, and they are free for this repo.** `orcvs/orcvs` is public, so
  `windows-2025` (4 vCPU, 16 GB) and `windows-11-arm` cost nothing. They are slow for Rust, and
  they have no MIDI device and no GPU.
- **Recommended CI.** (A) Add a Windows-target clippy cross-check to the Linux PR job now. I ran
  it here: shipped code passes clean in about 50 s with no extra tools. (B) Add a
  merge-queue-only `windows` job, shaped like the `macos` job, that runs nextest.
- **Local testing from macOS.** Use a Windows 11 Arm64 VM in VMware Fusion Pro (free) or UTM,
  with Microsoft's 90-day Enterprise evaluation ISO. Build `aarch64-pc-windows-msvc` natively
  inside the VM, and pass a USB MIDI device through. For a quick loop without a VM, run
  `cargo xwin test` under Wine in the `messense/cargo-xwin` Docker image; it proves logic only.
  Microsoft's old free dev-VM images expired in July 2024.

## 1. What Windows support needs to mean

### What already exists
- midir is already target-gated to include Windows (`console/Cargo.toml:137-138`), and
  `console_midi.rs` names `target_os = "windows"` in its native arm.
- `console/src/main.rs:2` already sets `windows_subsystem = "windows"` for release builds.
- `docs/tooling.md:157-165` records that the storage redirect does not work on Windows, and
  `:572-573` that the virtual-port loopback test is not compiled there.
- No CI job runs on Windows, and no Windows build has ever been tested. Ticket
  `native-midi/01` ticked "behave exactly as before on … Windows" with no Windows evidence.

### Proposed definition (v1)
| Area | Windows v1 bar |
|---|---|
| Target | `x86_64-pc-windows-msvc` on Windows 11. `aarch64-pc-windows-msvc` best-effort (free runner exists). No GNU target: rustup defaults to MSVC and GNU is for MinGW interop only. |
| Build and tests | Workspace clippy and nextest pass on a Windows runner, with default and `--no-default-features`. |
| Window and paint | eframe/glow starts on a machine with a real GPU driver. |
| Files | rfd Open/Save work, and persistence restores from `%APPDATA%\<app_id>\data`. |
| MIDI out | Discovery, selection, send, all-notes-off, and the "port busy" error on a real device or a Windows MIDI Services loopback. |
| Timing | Measured tick jitter on Windows against macOS, with the window both visible and minimized. |
| Distribution | Unsigned zip acceptable for v1 (SmartScreen warns). Signing is a separate decision. |
| Out of scope | OSC/UDP and audio (already deferred past v1), virtual MIDI ports, WinRT backend. |

## 2. IO on Windows

### MIDI (midir 0.11.1, the locked version)
- **Backends.** WinMM is the default. WinRT (`Windows.Devices.Midi`) needs `features = ["winrt"]`.
  Both depend on `windows >=0.59, <=0.62`. Stay on WinMM: nothing in Orcvs needs WinRT, and
  Microsoft now makes WinRT MIDI 1.0 port names match WinMM's anyway.
- **No virtual ports.** `VirtualInput` and `VirtualOutput` live in `src/os/unix.rs` behind
  `#[cfg(unix)]`, documented as "supported on every platform but Windows". This confirms
  `docs/tooling.md`.
- **Output behaviour.** WinMM sends immediately with `midiOutShortMsg`. On `MIDIERR_NOTREADY` it
  retries with 1 ms sleeps, up to 500 times. These loops were unbounded and could hang `send()`
  until 0.11.1, fixed in PRs #207 and #208 after a Microsoft review, so keep midir at 0.11.1 or
  later. There is no scheduled send, so Orcvs's own clock sets the timing.
- **Exclusive ports.** `midiOutOpen` can return `MMSYSERR_ALLOCATED`, and midir maps it to a
  connect error. On older Windows, or in MIDI Services' "Legacy API mode", a port another app
  holds is busy. The console must show that as an error the performer can recover from.
- **Windows MIDI Services** (MIDI 2.0, rolling out in-box to retail Windows 11 from February
  2026):
  - WinMM and WinRT calls are redirected to the new service, and "every MIDI 1.0 port and MIDI
    2.0 endpoint is multi-client, regardless of the driver or API used".
  - Loopbacks can be created by script: `midi loopback create --save-to-config` or
    `New-MidiLoopback -SaveToConfiguration`. Changing the configuration needs administrator
    rights, and the SDK tools are a separate install
    (`winget install Microsoft.WindowsMIDIServicesSDK`).
  - Microsoft says loopbacks create MIDI 1.0 API ports. That WinMM can see them is implied, not
    stated: **UNVERIFIED**.
  - midir has no MIDI Services backend, and I found no Rust crate for `Windows.Devices.Midi2`.
    Only the in-box redirection benefits Orcvs.
  - A Windows loopback test is possible: send to a pre-created Basic Loopback and read it back
    with `MidiInput::connect`. That needs a Windows 11 client machine. Whether MIDI Services
    exists on Server 2025 runners is **UNVERIFIED** and unlikely.
- **Third-party virtual ports** are a poor fit for CI. loopMIDI is GUI-only, documented for
  Windows 7–10. The teVirtualMIDI SDK needs a kernel driver and its licence terms are not
  published (**UNVERIFIED**). LoopBe1 is non-commercial only and shows popups.

### OSC/UDP (post-v1)
- rosc only encodes and decodes, so the transport is `std::net` or `tokio::net::UdpSocket`.
- **WSAECONNRESET.** If nothing is listening on the target port, Windows reports the ICMP "port
  unreachable" as a `ConnectionReset` error on a later `send_to` or `recv_from` on the same
  socket. Neither Rust std nor mio sets `SIO_UDP_CONNRESET`. Treat `ConnectionReset` on a UDP
  output socket as non-fatal. The alternative, the ioctl, is an `unsafe` FFI call under the
  repo's unsafe policy.
- Broadcast needs `set_broadcast(true)`, or the send fails with `WSAEACCES`.
- **Firewall.** The first time an app listens for inbound traffic, Windows prompts the user. Only
  an administrator's "allow" creates an allow rule. A send-only output is unlikely to prompt; an
  OSC input will.

### Audio (if ever added)
- cpal uses WASAPI by default. Its `asio` feature needs the Steinberg SDK (downloaded at build
  time unless `CPAL_ASIO_DIR` is set), libclang, and Visual Studio's `vcvarsall.bat`. That is
  build-time execution and a licence the repo's dependency policy would have to record.

### Other stack concerns
- **Timer resolution (biggest risk).**
  - tokio's `sleep` has millisecond granularity, and Windows may be coarser.
  - Since Windows 10 2004, `timeBeginPeriod` applies per process. On Windows 11, a minimized or
    occluded process loses its raised timer resolution unless it opts out with
    `SetProcessInformation(ProcessPowerThrottling, PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION)`.
  - mio has no high-resolution timer path. Playback waits on `tokio::time::sleep_until`
    (`orcvs/src/playback.rs:1317`).
- **OpenGL.**
  - eframe/glow uses WGL first and EGL as a fallback, so it needs a real GPU driver.
  - VMs and CI have only Microsoft's GL 1.1 software driver, which cannot run egui_glow.
    mesa-dist-win's llvmpipe (GL 4.5, `GALLIUM_DRIVER=llvmpipe`) is the workaround.
  - The console's kittest UI tests do not render (only `release-capture` does), so CI tests
    should not need GL.
- **rfd.** COM `IFileDialog` on the UI thread, which is how the console already calls it.
- **Release builds have no console window**, so tracing to stderr is lost. Log to a file if
  Windows diagnostics are needed.
- **Line endings.** The repo has no `.gitattributes`, and Git for Windows commonly checks out
  CRLF. 8 `include_str!`/`include_bytes!` sites and `console/tests/fixtures` could then compare
  CRLF against LF. Adding `* text=auto eol=lf` avoids this; a Windows CI job would catch it.
- **Signing.** Unsigned binaries get the "Windows protected your PC" warning, and EV
  certificates no longer bypass SmartScreen. Microsoft's Artifact Signing service costs from
  $9.99 a month and works in CI. Smart App Control may block unsigned executables.

## 3. GitHub Actions Windows runners

| Label | Image | This repo (public) |
|---|---|---|
| `windows-2025` (= `windows-latest` since Sep 2025) | Server 2025 x64 | 4 vCPU, 16 GB, free |
| `windows-2022` | Server 2022 x64 | same |
| `windows-11-arm` | Windows 11 Arm64 | 4 vCPU, 16 GB, free |

- **Cost.** Free and unlimited for public repos. For reference, private rates are Linux $0.006,
  Windows $0.010 and macOS $0.062 per minute.
- **Preinstalled on `windows-2025`:** Rust 1.98, VS 2022 C++ tools, LLVM 20, Git, MSYS2. **Not
  installed:** mise, cargo-nextest, Mesa. The repo pins Rust 1.99.1 in `rust-toolchain.toml`,
  and rustup installs it.
- **Speed.** Windows runners are known to be slow: runner-images #7320 was closed with no fix.
  Mitigations:
  - `samypr100/setup-dev-drive` puts `CARGO_HOME`, `RUSTUP_HOME` and the workspace on a ReFS Dev
    Drive.
  - nextest is slow on Windows because process creation is slow, and it starts one process per
    test.
  - Budget 40 minutes or more, like the `macos` job.
- **No MIDI device and no GPU** on the images.
- **mise task bodies are POSIX shell.** On Windows, mise runs inline scripts with `cmd /c`. A
  Windows job needs `shell = "bash -c"` (Git Bash is on the image) or `run_windows`, and should
  run only the cargo part. actionlint, zizmor, node and the tooling contract check repository
  files and already run on Linux.

### Options for this repo's tiers
**A. Windows-target clippy on the Linux PR job (recommended now; added as `mise run check_windows`).** Verified here in this
container:

| Command | Result |
|---|---|
| `cargo clippy --workspace --locked --target x86_64-pc-windows-msvc -- -D warnings` | passes, 49 s cold, no extra tools |
| `… --all-targets …` | fails: criterion's `alloca` build script needs `lib.exe` or clang-cl to compile C for MSVC |

So A covers shipped code today. Covering tests and benches as well needs cargo-xwin or clang-cl
with the MSVC SDK on the runner, or dropping `--all-targets` for this check. I could not test
cargo-xwin here because the sandbox proxy blocks Microsoft's download host; that is a container
limit, not a tool limit. A catches `cfg(windows)` breakage in midir, rfd and winit. It runs
nothing.

**B. Merge-queue-only `windows` job, shaped like `macos`.**
- Runs `if: github.event_name != 'pull_request'` on `windows-2025`: Dev Drive, rust-cache with
  `save-if` on main, mise-action, then `cargo install cargo-nextest` or taiki-e/install-action.
- A new bash-shelled mise task runs the default and `--no-default-features` clippy and nextest
  pairs.
- The job goes into `ci` `needs:` and `scripts/check-ci-results.sh` with the macOS
  skipped-on-PR rule.
- Do a manual-dispatch run first to measure its wall time.

**C. Scheduled weekly Windows workflow, outside `ci`.** The fallback if B slows the merge queue
too much.

**Not recommended:**
- Self-hosted runners: GitHub says they "should almost never be used for public repositories".
- AppVeyor: adds nothing hosted runners lack, and splits CI across two systems.

## 4. Testing Windows locally from macOS or Linux

There is no single canonical path. There are two loops, and they prove different things.

**Quick loop, no VM: cross-compile plus Wine.**
- `cargo xwin build|test --target x86_64-pc-windows-msvc` uses Microsoft's CRT and SDK, which
  means accepting their licence. `test` runs under Wine.
- The easiest route is the `messense/cargo-xwin` Docker image, which bundles Wine.
- Homebrew's Wine cask is disabled on Apple Silicon, so use Docker or Linux.
- It proves logic and compile only. Wine's MIDI maps to CoreMIDI or ALSA and its GL maps to the
  host's, so neither says anything about Windows.
- The GNU target with mingw-w64, or `cross test --target x86_64-pc-windows-gnu`, is an
  alternative, but it is not the target we would ship.

**Real loop: a Windows 11 VM.**
- **Images.** Microsoft's free dev VMs expired 15 July 2024 and had no Arm version. Use the
  Windows 11 Enterprise 90-day evaluation ISO, available for x64 and Arm64. After expiry it
  shuts down hourly. For a VM you keep, use the retail Arm64 ISO and a Pro licence.
- **Apple Silicon hypervisors:**
  - **VMware Fusion Pro**: free for all use since November 2024, runs Windows 11 Arm, DX11.
  - **UTM**: free, Virtualize → Windows, SPICE guest tools.
  - **Parallels**: paid, and the only one Microsoft names as authorized (M1–M3).
- **Linux hosts:** quickemu, or `dockur/windows` (needs KVM, not macOS).
- **Arm vs x64.** Windows 11 on Arm runs x64 binaries under Prism emulation. Build
  `aarch64-pc-windows-msvc` natively in the VM, and leave native x64 to the CI runner.

**Setup steps on a Mac:**
1. Install VMware Fusion Pro or UTM.
2. Download the Windows 11 Arm64 Enterprise evaluation ISO.
3. Create the VM with 4+ cores, 8–16 GB RAM and 80 GB disk, then install the guest tools.
4. In the VM, install the tools:
   ```
   winget install Microsoft.VisualStudio.2022.BuildTools   # add "Desktop development with C++" (ARM64)
   winget install Rustlang.Rustup Git.Git jdx.mise
   cargo install cargo-nextest --locked
   ```
   The workload ID is **UNVERIFIED** for 2026; check it in the VS installer.
5. Create a Dev Drive, clone the repo onto it, and point `CARGO_HOME` there.
6. Run `cargo nextest run --workspace --locked`, then `cargo run -p console`.
7. For MIDI, pass a USB MIDI interface through to the VM, or install the Windows MIDI Services
   SDK tools and create a loopback.
8. GL in an Arm VM goes through Microsoft's OpenGL-on-D3D12 compatibility pack, or mesa llvmpipe.
   Whether glow gets a context in each hypervisor is **UNVERIFIED**; try it first.

## Suggested next steps (tickets, not done)
1. Add option A to `check_pull_request`, scoped to shipped code, with a recorded rationale for the
   new rustup target.
2. Add a `.gitattributes` with `* text=auto eol=lf`.
3. Do a one-off manual-dispatch Windows job to find real breakage and wall time, then decide
   between B and C.
4. Write a Windows tick-jitter measurement, and decide whether Playback needs the
   timer-resolution opt-out.
5. Write a Windows MIDI loopback test variant against a MIDI Services Basic Loopback, run by hand
   in the VM.
6. Leave signing and distribution for later.

## Sources
- midir 0.11.1 crate source (README, CHANGELOG, `src/backend/mod.rs`, `src/os/unix.rs`, `src/backend/winmm/mod.rs`): https://static.crates.io/crates/midir/midir-0.11.1.crate , https://github.com/Boddlnagg/midir , PRs https://github.com/Boddlnagg/midir/issues/207 https://github.com/Boddlnagg/midir/issues/208
- midiOutOpen: https://learn.microsoft.com/en-us/windows/win32/api/mmeapi/nf-mmeapi-midioutopen
- Windows MIDI: https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/midi , https://blogs.windows.com/windowsexperience/2026/02/17/making-music-with-MIDI-just-got-a-real-boost-in-windows-11/ , https://midi.org/midi-2-0-coming-to-windows-11 , https://microsoft.github.io/MIDI/sdk-reference/Transports/Loopback/ , https://microsoft.github.io/MIDI/sdk-reference/Transports/Virtual/ , https://microsoft.github.io/MIDI/kb/common-midi-tasks/
- loopMIDI / virtualMIDI / LoopBe1: https://www.tobias-erichsen.de/software/loopmidi.html , https://www.tobias-erichsen.de/software/virtualmidi.html , https://www.nerds.de/en/loopbe1.html
- Winsock: https://learn.microsoft.com/en-us/windows/win32/winsock/winsock-ioctls , https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-sendto , https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-recvfrom , https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-setsockopt ; Rust std sockets: https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/sys/net/connection/socket/windows.rs ; mio 1.1.0: https://static.crates.io/crates/mio/mio-1.1.0.crate ; rosc: https://docs.rs/rosc
- Firewall: https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/rules
- cpal: https://github.com/RustAudio/cpal
- Timers: https://docs.rs/tokio/latest/tokio/time/fn.sleep.html , https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod , https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessinformation
- eframe 0.36.2 (`file_storage.rs`, `glow_integration.rs`): https://static.crates.io/crates/eframe/eframe-0.36.2.crate ; glutin-winit 0.5.0: https://static.crates.io/crates/glutin-winit/glutin-winit-0.5.0.crate ; mesa-dist-win: https://github.com/pal1000/mesa-dist-win , https://github.com/ssciwr/setup-mesa-dist-win
- rfd 0.17.2: https://static.crates.io/crates/rfd/rfd-0.17.2.crate
- windows_subsystem: https://doc.rust-lang.org/reference/runtime.html ; rustup on Windows: https://rust-lang.github.io/rustup/installation/windows.html
- SmartScreen / signing: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation
- gitattributes: https://git-scm.com/docs/gitattributes
- Runners: https://docs.github.com/en/actions/reference/runners/github-hosted-runners , https://docs.github.com/en/billing/reference/actions-runner-pricing , https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md , https://github.com/actions/runner-images/blob/main/images/windows/Windows11-Arm64-Readme.md , https://github.com/actions/runner-images/issues/7320 , https://github.blog/changelog/2025-07-31-github-actions-new-apis-and-windows-latest-migration-notice/ , https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/
- Dev Drive / nextest: https://github.com/samypr100/setup-dev-drive , https://nexte.st/docs/installation/windows/ ; rust-cache: https://github.com/Swatinem/rust-cache
- mise: https://mise.jdx.dev/tasks/toml-tasks.html , https://mise.jdx.dev/installing-mise.html
- Cross-compile: https://github.com/rust-cross/cargo-xwin , https://github.com/cross-rs/cross , https://doc.rust-lang.org/cargo/reference/config.html , https://formulae.brew.sh/cask/wine-stable
- Self-hosted security: https://docs.github.com/en/actions/reference/security/secure-use ; AppVeyor: https://www.appveyor.com/pricing/
- VMs: https://developer.microsoft.com/downloads/virtual-machines , https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise , https://www.microsoft.com/en-us/software-download/windows11arm64 , https://support.microsoft.com/windows/options-for-using-windows-11-with-mac-computers-with-apple-m1-and-m2-chips-cd15fd62-9b34-4b78-b0bc-121baa3c568c , https://blogs.vmware.com/cloud-foundation/?p=15956 , https://knowledge.broadcom.com/external/article/315609/features-of-windows-11-vm-on-vmware-fusi.html , https://docs.getutm.app/guides/windows/ , https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation , https://devblogs.microsoft.com/directx/announcing-the-opencl-and-opengl-compatibility-pack-for-windows-10-on-arm/ , https://github.com/quickemu-project/quickemu , https://github.com/dockur/windows
