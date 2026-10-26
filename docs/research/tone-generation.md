# Tone generation for the built-in synth: research

Research date: 2026-10-26, against `f5ad1c7` on `main`.
Each external claim cites a primary source from the Sources list at the end, as `[n]`. A claim
marked **UNVERIFIED** could not be confirmed from a primary source. A claim marked
**inference** follows from the cited sources but none of them states it. A claim marked
**measured** comes from the WASM size probe in section 3, which anyone can rebuild from the
description there.

The question, from Toby: what is the best option for tone generation in October 2026, what is
most featured, what is easiest to integrate, and what is the coolest. The constraints are settled:
the built-in synth is a MIDI destination behind the existing `MidiBackend` seam
(`orcvs/src/midi.rs`, `console/src/web_midi.rs`); native is the primary product; the WASM web build
must also make sound; plug-in hosting (VST/CLAP) never runs on WASM; the simplest code wins.

## Recommendation

Write a **small synth core in Rust by hand** (oscillators, an envelope, two-operator FM, noise
drums), and drive it with **cpal on both targets**: its platform hosts natively, and its
`wasm-bindgen` Web Audio host in the browser.

- **One synth, two targets.** cpal 0.18.2 has hosts for ALSA, CoreAudio, WASAPI and the browser
  [2][3]. The synth core is one function that turns MIDI bytes into voice state and another that
  fills an `f32` buffer. The same code makes the same sound natively and on orcvs.com. Every other
  shape needs either two synths or JavaScript beside the console.
- **It is the smallest download.** A 16-voice core with four waveforms, an envelope, a low-pass
  and noise compiled to 8.5 KB of WASM, 3.8 KB gzipped (measured). cpal's web host adds about 60 KB,
  about 22 KB gzipped, on top (measured). It needs no sample or SoundFont asset.
- **It adds one dependency.** On Linux and Windows cpal needs `alsa` 0.11 and `windows` 0.62 [2],
  which `Cargo.lock` already holds at those versions through `midir`, plus the new `dasp_sample`
  [2]. macOS adds `coreaudio-rs`, `mach2` and the `objc2-core-audio` family [2]. cpal is Apache-2.0, which `deny.toml`
  allows.
- **It is the simplest code that meets the constraints.** MIDI parsing is a dozen lines: Note On,
  Note Off and the channel nibble. A voice is a phase, an increment, an envelope level and a gate.
  Tests can assert on rendered samples or voice state with no audio device, below the cpal call.
- **There is room to grow.** If the ADR wants reverb, chorus, filters with resonance or richer
  drums, add **fundsp** for those parts (section 2.3). It costs about 115 KB gzipped (measured).

The risk is latency on the web. cpal's Web Audio host calls the synth on the **browser main
thread**, the same thread as egui, and queues 2048-frame buffers by default [5] (inference: about
43 ms at 48 kHz). That suits a lesson embed. It will not suit someone playing live in the browser.
cpal's low-latency `audioworklet` host needs nightly, `-Zbuild-std` with atomics, and
cross-origin isolation headers [3], and the repo treats nightly as optional and non-blocking
(`CLAUDE.md`). Section 4 lists what the ADR has to decide.

This changes ticket 08's wording. Ticket 08 and the spec say the destination turns bytes into "Web
Audio voices through `web-sys`" (`.scratch/orcvs-site/spec.md`, Built-in synth). With this
recommendation the `web-sys` calls belong to cpal, and native gets the same synth.

## 1. What Orcvs has today

- `MidiBackend` lists destinations and connects to one. A connection is a `MidiConnection` with
  one method, `send(&mut self, message: &[u8])`, and the trait requires `Send`
  (`orcvs/src/midi.rs:80-108`). A built-in synth is one more destination, and its connection
  pushes bytes to the synth instead of a port.
- The web backend keeps browser objects on the one main thread and reaches them through
  `spawn_local` and thread-local access (`console/src/web_midi.rs`). A web synth connection would
  use the same pattern, because `web-sys` handles are not `Send` (inference).
- `Cargo.lock` has `midir` 0.11.1, `alsa` 0.11.0, `windows` 0.62.2, `web-sys` 0.3.104 and
  `wasm-bindgen` 0.2.127. It has **no audio output crate**: no cpal, rodio, fundsp or ring buffer.
  `libm` 0.2.16 is locked.
- `deny.toml` allows Apache-2.0, BSL-1.0, BSD-2/3-Clause, ISC, MIT, Unicode-3.0 and Zlib, plus one
  MPL-2.0 exception for a test-only crate. Any LGPL, GPL, AGPL or MPL synth fails `mise run
  audit_deps` unless an exception is recorded.
- The toolchain is Rust 1.99.1 stable, with `wasm32-unknown-unknown` installed
  (`rust-toolchain.toml`). The console is single-threaded on the web and the site sets no
  cross-origin isolation headers (`docs/research/orcvs-site.md`, section 4).

**Reference points.** Pilot is MIT [39]. Its 16 channels are fixed Tone.js instruments: channels
0–5 are `AMSynth`, 6–9 are `FMSynth`, 10–11 are `MonoSynth` and 12–15 are `MembraneSynth`. All 16
feed one serial chain of 16 global effects, from bitcrusher through limiter [39]. Enfer plays 29 MB
of WAV kits through `Tone.Player` and `Tone.Sampler` [40] and has **no licence file** [40], so its
samples cannot be reused. Pilot is the useful model for *which sounds*: a channel picks a voice
type, and a few drum voices cover percussion.

## 2. The candidates

Versions and dates were current on 2026-10-26.

| | Latest | Licence | Native | Web | Sounds out of the box | MIDI-byte fit | WASM cost (gz) |
|---|---|---|---|---|---|---|---|
| Web Audio via `web-sys` | spec ED 2026-09-09 [35] | n/a | **No** | browser render thread | oscillators, gain, filters, delay, convolver, compressor [35] | hand-map | ~1.5 KB over a `web-sys` baseline (measured) |
| cpal + hand-written DSP | cpal 0.18.2, 2026-08-16 [1][4] | Apache-2.0 [1] | Yes | main thread, `AudioBufferSourceNode` queue [5] | what you write | hand-map | ~26 KB incl. 4 KB DSP (measured) |
| fundsp (+ cpal) | 0.23.0, 2026-01-07 [7] | MIT OR Apache-2.0 [8] | via cpal | via cpal | oscillators, FM, ADSR, Moog, SVF, Karplus-Strong, drums, reverbs, chorus, phaser [8][9] | hand-map | ~117 KB (measured) |
| rustysynth (+ cpal) + GM font | 1.3.8, 2026-10-20 [11] | MIT [11] | via cpal | via cpal | 128 GM programs + drum kit, reverb, chorus [12] | `process_midi_message` 1:1 [12] | ~53 KB code (measured) + font (32 MB for GeneralUser GS [16]) |
| oxisynth | 0.1.0, 2025-05-25 [14] | **LGPL-2.1** [14] | via cpal | via cpal | SF2/SF3 | 1:1 | not measured |
| Glicol | crates 0.13.5, 2024-04-23 [18] | MIT [19] | via cpal | AudioWorklet [19] | `bd`, `sn`, `hh`, saw/square/tri synths, plate reverb, sequencer [19] | own language | 1.17 MB npm WASM [19] |
| knyst | 0.5.1, 2024-11-25 [21] | MIT OR Apache-2.0 [21] | via cpal/JACK | not targeted [21] | graph, wavetables | hand-map | not measured |
| Faust → Rust (+ cpal) | compiler 2.88.0, 2026-09-08 [23] | generated code: your choice [25] | via cpal | via cpal | whole Faust library, incl. DX7 and physical models [25] | `gate`/`freq` params | UNVERIFIED |
| Tone.js bridged from JS | 15.1.22 stable; 15.5.62 `next`, 2026-10-25 [28] | MIT [28] | **No** | Web Audio nodes | 11 instruments, effects, Transport [28] | JS glue | ~80 KB gz JS [28] |
| Elementary Audio | core 4.0.1, 2024-12-10 [30] | MIT [30] | separate native runtime | WASM renderer [30] | functional DSP graph | JS glue | ~1 MB npm unpacked [30] |

### 2.1 Web Audio directly, through `web-sys`

- **How it sounds.** Each note is an `OscillatorNode` (`sine`, `square`, `sawtooth`, `triangle`
  or a `custom` periodic wave) into a `GainNode`. The envelope is `AudioParam` automation [35].
  Filters, delay, convolution reverb and compression are built-in nodes [35].
- **Thread model.** The page builds the graph on the *control thread*. The browser computes the
  audio on a separate *rendering thread* in 128-frame render quanta [35]. A slow egui frame
  delays when a note starts. It cannot make the audio glitch (inference from [35]).
- **Cost.** A 16-voice version measured 21.7 KB against 18.6 KB for a bare `AudioContext` with
  the same wasm-bindgen setup: about 3 KB, 1.5 KB gzipped (measured).
- **Native.** There is none. The native app would need a second synth. The
  `web-audio-api` crate (MIT, 1.7.0, 2026-08-08) implements the same API natively over cpal
  [33]. That would let one design run on both targets through two thin backends. But it pulls
  `symphonia`, `hrtf`, `rubato`, `creek`, `realfft` and more as non-optional dependencies [33],
  which is heavy for a few oscillators.
- **Verdict.** This is the best web-only answer. Toby has settled that native is primary, so on
  its own it covers only half the product.

### 2.2 cpal with hand-written oscillators

- **Maintenance.** 0.18.2 was released on 2026-08-16. 0.18.0 and 0.18.1 were released in June
  and 0.17.x ran from December 2025 to February 2026 [4]. It has about 9.2 million downloads in
  the last 90 days [1].
- **Web support.** The `wasm-bindgen` feature is "required for any WebAssembly audio support"
  [3]. That host calls the data callback from an `ended` event on the main thread. It copies the
  result into an `AudioBuffer` and schedules an `AudioBufferSourceNode` ahead [5]. The default
  buffer is 2048 frames [5], and `BufferSize::Fixed` changes it [4]. The `audioworklet` feature runs
  on a dedicated audio thread, but needs nightly, `-Zbuild-std`, atomics and `SharedArrayBuffer`
  headers [3][6]. wasm-bindgen's own worklet example states the same nightly and header
  requirements [36].
- **Native.** ALSA on Linux, CoreAudio on macOS and WASAPI on Windows are the defaults. JACK,
  PipeWire, PulseAudio and ASIO are opt-in features [2][3]. The callback runs on the platform's
  audio thread. MIDI bytes from the Playback Engine's task therefore need a handoff to it
  (inference). A single-producer single-consumer ring is the usual tool, such as `rtrb` (MIT OR
  Apache-2.0, 0.4.1, 2026-10-25 [44]). It is not locked today.
- **Size.** cpal with the 16-voice core: 82.7 KB of WASM and 22.4 KB of JS glue, 33.8 KB gzipped.
  The wasm-bindgen baseline alone is 18.6 KB / 7.7 KB gzipped (measured).
- **Features.** Only what you write. A useful set takes a few hundred lines (inference): sine,
  triangle, saw and square; a linear or exponential ADSR; two-operator FM; a one-pole or SVF
  low-pass; a pitched-sine kick and noise snare and hat, as Pilot's `MembraneSynth` channels and
  fundsp's `bassdrum` do [39][9].
- **Integration.** `send` parses the status byte. `0x9n` with velocity above zero starts a voice
  on channel `n`; `0x8n`, or `0x9n` with velocity zero, releases it. Everything else is ignored or
  mapped to a parameter. Voice stealing is one rule, such as "oldest voice".

### 2.3 fundsp

- **Maintenance.** 0.23.0 was released on 2026-01-07 [7]. The last commit on the default branch
  is 2026-03-03 [10]. The project has one main author (inference from [10]). It has about 62,000
  downloads in the last 90 days [7].
- **Features.** This is the richest Rust synthesis library surveyed. The README documents
  oscillators, an FM example, `adsr_live`, a Moog ladder, SVF and biquad filters, a
  Karplus-Strong `pluck`, three stereo reverbs, chorus, phaser and a `Sequencer` that mixes
  dynamically added voices with real-time-safe frontend/backend splitting [8]. `fundsp::sound`
  ships `bassdrum`, `snaredrum` and `cymbal` [9]. The graph notation checks channel counts at
  compile time: the probe's snare-into-stereo-reverb chain was rejected until an explicit
  `split::<U2>()` was added (measured).
- **Platforms.** It does no I/O; its own examples use cpal [8]. With `default-features = false`,
  0.23.0 **did not compile** for `wasm32-unknown-unknown`, failing on missing `vec!` and `Vec` in
  `resample.rs` (measured). With `features = ["std"]` it compiles, still without the default
  `files` (symphonia) and `fft` features.
- **Size.** FM, saw into Moog into chorus, `bassdrum`, and `snaredrum` into `reverb3_stereo`,
  inside a `Sequencer`: 289.7 KB, 117 KB gzipped (measured). That is about 30 times the
  hand-written core.
- **Integration.** Note On pushes a voice into the `Sequencer` and keeps its `EventId`. Note Off
  calls `edit_relative` to fade it out [8]. The backend renders inside the cpal callback.

### 2.4 SoundFont players

- **rustysynth** is MIT, version 1.3.8, released 2026-10-20 [11][13], and has "no dependencies other than
  the standard library" [11][12]. `Synthesizer::process_midi_message(channel, command, data1,
  data2)` takes MIDI as it arrives. The default polyphony is 64 and the block size is 64 [12]. The
  feature list includes the envelope generator, low-pass, vibrato and modulation LFOs, reverb and
  chorus [12]. It **rejects SF3** (compressed) fonts: the test `test_load_reject_sf3` checks that
  [12]. So any font it plays is uncompressed SF2. The code compiled to 138.8 KB, 53.5 KB gzipped
  (measured).
- **Fonts.** GeneralUser GS 2.0.3 is 32.3 MB as SF2. Its licence allows use "without restriction"
  and in software projects, but its author says he "cannot be 100% sure where all of the samples
  originated" [16]. Unison is described as CC0-1.0 by the `rameau_unison` crate, which ships it
  as a 7 MB SF3 [17]. The crate's own code is AGPL-3.0 [17], and rustysynth cannot read SF3 [12],
  so the SF2 would have to come from upstream. Its size and the CC0 claim are UNVERIFIED at the
  source, because the upstream host was not checked. Either font is a multi-megabyte download for
  a lesson page. That contradicts the site's goal that lessons load quickly
  (`.scratch/orcvs-site/spec.md`, story 6).
- **oxisynth** (0.1.0, 2025-05-25) reads SF3 through `lewton`, but it is **LGPL-2.1** [14].
  **xsynth** (0.4.0, 2026-04-04) is **LGPL-3.0** [15]. Both fail `deny.toml` as it stands.

### 2.5 Glicol

- It is a graph-oriented live-coding language whose engine is Rust [19]. It is MIT [19],
  although crates.io marks the licence "non-standard" because it uses `license-file` [18].
- **It looks stale.** The crates were last released on 2024-04-23 [18], the last commit on `main`
  is 2025-01-23 [19], and the npm package was last published on 2023-05-20 [20].
- Its web engine runs in an AudioWorklet. It uses `SharedArrayBuffer` when the page allows it and
  a message-port fallback otherwise [19]. The npm WASM is 1.17 MB [19]. The node set has drums
  (`bd`, `sn`, `hh`), simple synths and a plate reverb [19].
- For Orcvs, Glicol is a second language inside the first. Driving it from MIDI means generating
  Glicol code or reaching into its graph (inference).

### 2.6 knyst

- MIT OR Apache-2.0. The last release is 0.5.1 on 2024-11-25 [21], and every branch is at that
  date or older [22]. Its README says it "is not stable" and that its main target is "desktop
  multi-threaded environments" [21]. Its default features pull cpal and JACK [21]. The web is not
  a stated target. **Not a candidate.**

### 2.7 Faust, compiled to Rust

- Faust is a DSP language. Its compiler emits Rust with `-lang rust` and has Rust-specific
  options [24]. The compiler is LGPL [23], but its libraries carry an exception: you may
  distribute "the compiled code generated by the FAUST compiler ... under your own copyright and
  license" [25]. The generated Rust is therefore as clean as the console's own code (inference
  from [25]).
- **Maintenance.** The compiler is very active: tag 2.88.0 on 2026-09-08, and commits up to
  2026-10-25 [23]. The Rust build glue (`faust-build`, `faust-types`) was last released on
  2024-11-20 and lists no repository [26]. The faustdoc pages list community Rust integrations
  [24].
- **Features.** It has the deepest library surveyed: oscillators, envelopes, filters and reverbs,
  physical models (`physmodels.lib`), and a DX7 emulation inspired by Dexed (`dx7.lib`, marked
  "under development") [25].
- **Integration.** The generated struct is plain Rust with `compute` and parameter setters [24].
  It runs through cpal like the hand-written core. The cost is a code-generation step: either
  `build.rs` runs the Faust compiler, which is build-time execution that `CLAUDE.md` asks a
  rationale for, or a mise task regenerates committed `.rs` files. Its WASM size is UNVERIFIED,
  because no Faust compiler was installed here.

### 2.8 Bridged from JavaScript: Tone.js, Elementary, Strudel

- **Tone.js** is MIT. The stable release is 15.1.22 (2025-04-27), and the `next` tag is very
  active (15.5.62, 2026-10-25) [28]. Its `build/Tone.js` is 345 KB, 80 KB gzipped (measured from
  the npm tarball [28]). It has `Synth`, `AMSynth`, `FMSynth`, `DuoSynth`, `MonoSynth`,
  `MembraneSynth`, `MetalSynth`, `NoiseSynth`, `PluckSynth`, `PolySynth` and `Sampler` [28], plus
  the effect set Pilot chains [39]. For browser sound alone it is the most featured option. It has
  no native story, and it puts JavaScript beside the console, which the spec rules out
  ("needs ... no JavaScript beside the console", `.scratch/orcvs-site/spec.md`). Ticket 07 already
  lists the Enfer-style Tone.js iframe as a rejected alternative.
- **Elementary Audio** is MIT. `@elemaudio/core` 4.0.1 dates from 2024-12-10, the `next` tag is
  4.0.3 from 2025-09-18, and the last commit on `main` is 2024-12-21 [30][31]. It is slow-moving
  and needs JS glue.
- **Strudel / superdough** is **AGPL-3.0** [32]. That is incompatible with the deny list.

### 2.9 Also checked, not candidates

- **firewheel** (MIT OR Apache-2.0, 0.14.0, 2026-09-05) is a game audio graph engine headed into
  Bevy. It has a WASM backend but no synth voices beyond a test beep and noise generators [34].
- **kira** (0.12.5) and **rodio** (0.23.0) are playback engines built on cpal [37]. They solve
  "play this sound" rather than "turn MIDI into tones", and add a layer over cpal.
- **Cmajor** is GPLv3 or commercial [38] and has no Rust backend.
- **mimium** (MPL-2.0, 4.1.1) [42] and **tunes** (1.1.0, 1,648 downloads) [43] are small. crates.io
  searches for "synthesizer", "synth", "audio dsp" and "soundfont" on 2026-10-26 turned up nothing
  else with real uptake [41].

## 3. WASM size probe

The probe was a `cdylib` built for `wasm32-unknown-unknown` with Rust 1.99.1, with `lto = true`,
`codegen-units = 1`, `strip = "symbols"` and `panic = "abort"`. Every arm exports
`note(on, key, vel)` and `render(n)`, so nothing is dead-code eliminated. `wasm-bindgen` 0.2.129
`--target web` ran on each, and `wasm-opt` did not. The console runs `wasm-opt` at level 2
(`console/index.html`), so shipped numbers would be smaller. The console already carries std
formatting and wasm-bindgen, so for the Web Audio and cpal arms the fair cost is the delta over
the `wasm-bindgen + AudioContext` row.

| Arm | `.wasm` bytes | gzip -9 | JS glue |
|---|---|---|---|
| empty | 149 | 173 | 3,446 |
| hand-written: 16 voices, 4 waves, ADSR, one-pole LP, noise | 8,502 | 3,849 | 3,446 |
| wasm-bindgen + `web-sys` `AudioContext` only | 18,575 | 7,667 | 10,025 |
| Web Audio direct: 16 `OscillatorNode` + `GainNode` voices | 21,716 | 9,184 | 12,684 |
| cpal 0.18.2 `wasm-bindgen` host + hand-written core | 82,685 | 33,820 | 22,399 |
| rustysynth 1.3.8 (no font) | 138,800 | 53,463 | 3,446 |
| fundsp 0.23.0 `std` only: FM, saw→Moog→chorus, kick, snare→reverb, `Sequencer` | 289,742 | 117,056 | 3,446 |

## 4. Verdicts

**Most featured: fundsp.** Among options that run on both targets from one codebase, it has the
most building blocks [8][9]: FM, ADSR, a Moog ladder, Karplus-Strong, drum voices, three reverbs,
chorus, phaser and a real-time-safe voice sequencer. It is also permissively licensed and
maintained this year. Tone.js has more ready-made instruments and effects [28], but only in a
browser, and only through JavaScript. rustysynth with a General MIDI font has the most *ready
sounds*, 128 programs and a kit, but they are fixed samples, not synthesis.

**Easiest to integrate: rustysynth.** Its API is a MIDI message: `process_midi_message(channel,
command, data1, data2)` [12]. Channel selection, drums on channel 10, polyphony limits and voice
stealing come for free. It has no dependencies [12] and compiles to WASM unchanged (measured). The
catch is outside the code: a free GM font is tens of megabytes as SF2 [16], rustysynth cannot read
the smaller SF3 [12], and the most permissive fonts have provenance caveats [16][17]. For the web
alone, Web Audio through `web-sys` is easier still. The browser does the DSP on its own thread and
the code is node wiring, but native gets nothing.

**Coolest: Faust compiled to Rust.** An Orca-style grid, rewritten in Rust, playing a Yamaha DX7
operator stack [25] or a waveguide physical model [25] in a browser tab, with no plug-in and no
JavaScript. The DSP comes from a research language with a large audited library, compiled into
ordinary Rust that clippy can read. The licence exception makes the output Orcvs's own [25], and
the compiler is under active development (2.88.0 in September 2026, commits up to the day before
this research [23]). It loses on
simplicity: a code-generation step, a DX7 library its authors call "under development" [25], and
an unmeasured binary size. Honourable mention: Glicol, a Rust live-coding language with drum
voices built in [19], which would be the obvious sibling if it had shipped since 2024.

## 5. What the ADR (ticket 07) then needs to decide

1. **Engine.** Hand-written core plus cpal on both targets (recommended), with fundsp named as the
   escalation path. Record the rejected alternatives with reasons: Web Audio through `web-sys`
   alone (no native), a Tone.js iframe (JavaScript beside the console, no native), rustysynth
   (font size and provenance), Faust (code-generation step), Glicol and knyst (unmaintained), and
   LGPL or AGPL engines (`deny.toml`).
2. **Ticket 08's wording.** It currently says "Web Audio voices through `web-sys`". It needs to
   say cpal and its `wasm-bindgen` host, record cpal as a dependency with its rationale, and run
   `mise run audit_deps`. Native gets the destination too, presumably named the same, through
   `console_midi`'s native `MidiBackend`.
3. **Which messages it answers.** Note On and Off are certain. Choose whether the channel picks a
   voice type, as Pilot does with 16 fixed instruments [39], and which channels are drums. Choose
   whether Control Change sets anything, such as cutoff, decay or volume, or is ignored.
4. **Voices.** Choose the polyphony cap, for example 16, and the stealing rule (oldest, quietest,
   or same key first). Choose whether a channel is monophonic.
5. **The sounds.** Name a short palette to build first, for example sine, triangle, saw and square
   leads, one FM bell, kick, snare and hat. State that no third-party samples ship, so no font or
   sample licence enters.
6. **Threads and handoff.** On native, how bytes cross from the Playback Engine's task to cpal's
   audio thread: a lock-free SPSC ring, which would mean a new dependency such as `rtrb` [44], or a
   `std` channel drained without blocking. On the web, accept that the synth renders on the main
   thread [5]. Choose the buffer size, which trades latency against underruns when an egui frame
   runs long. Name the AudioWorklet host as the upgrade path if stable Rust ever builds it.
7. **Audio lifecycle.** The stream starts or resumes on the action that starts Playback. A
   browser allows an `AudioContext` to run only once it is "allowed to start" [35], and cpal's
   `play()` calls `resume()` [5]. Decide what happens to sounding voices on Stop: release them, or
   cut them off.
8. **Testing seam.** The synth core takes bytes and fills a buffer with no I/O. Tests drive it
   through `MidiConnection::send` and assert on the voice state or rendered samples below the cpal
   stream, so no shipped item takes a test-only parameter (`CLAUDE.md`).

## Sources

1. https://crates.io/api/v1/crates/cpal
2. cpal 0.18.2 `Cargo.toml` (features, per-target dependencies), from https://crates.io/api/v1/crates/cpal/0.18.2/download
3. cpal 0.18.2 `README.md` (platform table, optional features, WebAssembly), same archive
4. cpal 0.18.2 `CHANGELOG.md`, same archive
5. cpal 0.18.2 `src/host/webaudio/mod.rs` (`DEFAULT_BUFFER_SIZE`, `ended`-event scheduling, `resume`), same archive
6. cpal 0.18.2 `src/host/audioworklet/mod.rs` ("Requires atomics support"), same archive
7. https://crates.io/api/v1/crates/fundsp
8. fundsp 0.23.0 `README.md` and `Cargo.toml`, from https://crates.io/api/v1/crates/fundsp/0.23.0/download
9. fundsp 0.23.0 `src/sound.rs` (`bassdrum`, `snaredrum`, `cymbal`), same archive
10. https://github.com/SamiPerttu/fundsp (default-branch HEAD `5595840`, 2026-03-03)
11. https://crates.io/api/v1/crates/rustysynth
12. rustysynth 1.3.8 `README.md`, `src/synthesizer.rs`, `src/synthesizer_settings.rs`, `src/soundfont.rs`, from https://crates.io/api/v1/crates/rustysynth/1.3.8/download
13. https://github.com/sinshu/rustysynth (HEAD `8aa60af`, 2026-10-20)
14. https://crates.io/api/v1/crates/oxisynth and oxisynth 0.1.0 `Cargo.toml`
15. https://crates.io/api/v1/crates/xsynth-core
16. https://github.com/mrbumpy409/GeneralUser-GS (`documentation/LICENSE.txt`, `GeneralUser-GS.sf2` blob size, HEAD 2026-02-23)
17. https://crates.io/api/v1/crates/rameau_unison and rameau_unison 0.2.1 `Unison.LICENSE`, `Cargo.toml`
18. https://crates.io/api/v1/crates/glicol_synth, https://crates.io/api/v1/crates/glicol
19. https://github.com/chaosprint/glicol (`README.md`, `LICENSE`, `rs/synth/src/node/`, `js/npm/index.js`, `js/npm/glicol_wasm_bg.wasm`; `main` HEAD `0317db2`, 2025-01-23)
20. https://registry.npmjs.org/glicol
21. https://crates.io/api/v1/crates/knyst and knyst 0.5.1 `README.md`, `Cargo.toml`
22. https://github.com/ErikNatanael/knyst (branches `main` 2024-11-25, `refactor` 2023-12-30, `helper_thread` 2022-09-16)
23. https://github.com/grame-cncm/faust (`COPYING.txt`; tag `2.88.0`, 2026-09-08; HEAD 2026-10-25)
24. https://github.com/grame-cncm/faustdoc `mkdocs/docs/manual/options.md` (`-lang rust`, Rust-only options), `mkdocs/docs/manual/architectures.md` (Rust architectures and tools)
25. https://github.com/grame-cncm/faustlibraries `oscillators.lib` (GRAME licence and exception), `dx7/dx7.lib`, `physmodels.lib`
26. https://crates.io/api/v1/crates/faust-build, https://crates.io/api/v1/crates/faust-types
27. https://registry.npmjs.org/@grame/faustwasm
28. https://registry.npmjs.org/tone and the 15.1.22 tarball (`build/Tone.js`, `build/esm/instrument/`)
29. https://github.com/Tonejs/Tone.js (HEAD 2026-10-25)
30. https://registry.npmjs.org/@elemaudio/core, https://registry.npmjs.org/@elemaudio/web-renderer
31. https://github.com/elemaudio/elementary (`main` HEAD 2024-12-21)
32. https://registry.npmjs.org/@strudel/web, https://registry.npmjs.org/superdough
33. https://crates.io/api/v1/crates/web-audio-api and web-audio-api 1.7.0 `README.md`, `Cargo.toml`
34. https://crates.io/api/v1/crates/firewheel and firewheel 0.14.0 `README.md`, `Cargo.toml`
35. https://github.com/WebAudio/web-audio-api `index.bs` (Level 1.1 editor's draft, HEAD 2026-09-09): `OscillatorType`, "Control Thread and Rendering Thread", render quantum size, "allowed to start"
36. https://github.com/wasm-bindgen/wasm-bindgen `guide/src/examples/wasm-audio-worklet.md`
37. https://crates.io/api/v1/crates/kira, https://crates.io/api/v1/crates/rodio
38. https://github.com/cmajor-lang/cmajor `LICENSE.md`
39. https://github.com/hundredrabbits/Pilot (`LICENSE.md`, `desktop/package.json`, `desktop/sources/scripts/mixer.js`; HEAD `a77a4e0`, 2023-12-23)
40. https://github.com/neauoire/Enfer (no licence file; `media/` 29 MB; `scripts/kit.js`; HEAD `9326faf`, 2023-03-10)
41. https://crates.io/api/v1/crates?q=synthesizer, `?q=synth`, `?q=audio%20dsp`, `?q=soundfont` (sorted by recent updates, 2026-10-26)
42. https://crates.io/api/v1/crates/mimium-lang
43. https://crates.io/api/v1/crates/tunes
44. https://crates.io/api/v1/crates/rtrb
