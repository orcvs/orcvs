# UDP and OSC output for Orcvs: research

Research date: 2026-10-26, against `ffded64` on `main`.
Every external claim cites a primary source in the Sources list at the end. Anything marked
**UNVERIFIED** could not be confirmed from a primary source. Upstream code is pinned to Orca
`223e45c`, Orca-c `9df9786` (the commits `.scratch/language-map/orca-bang-reference.md` already
uses), and Pilot `a77a4e0`.

## Summary

- **Orca's model is small.** Both implementations scan a payload east of the operator, lock
  those Cells, queue one event per banged operator during the frame, and flush the queue after
  the whole grid has run. There is no scheduling, no retry, and almost no error reporting.
- **JS and C disagree on the details.** Orca JS scans `;` and `=` east to the first `.`, at most
  36 Cells for `;` and 35 for `=`. Orca-c caps `;` at 16 Cells, and its `=` takes an explicit
  base-36 length operand instead of scanning. JS sends UDP and OSC to separate ports
  (49161, 49162) on one IP. Orca-c sends both to a single host and port that the user sets in
  a menu, and it has no default.
- **OSC arguments are always int32.** Each payload glyph becomes its base-36 value. The address
  is `/` plus one glyph. Orcvs's two-Cell Numbers (00–FF) fit int32 with no change of meaning.
- **UDP payloads are raw glyph text.** Pilot, the main UDP receiver, parses plain-text commands
  and splits them on `;`.
- **The transport should be `std::net::UdpSocket`.** It needs no new dependency, and its `send`
  is one syscall. tokio's `net` feature is not enabled in this workspace. It would add `mio`
  and `socket2`, and it is a compile error on wasm32. On `wasm32-unknown-unknown`, std's
  `UdpSocket` compiles but `bind` returns `ErrorKind::Unsupported`. I measured that under node.
- **rosc 0.11.4 is the only mature encoder, and it has stalled.** The last release and last
  commit were both March 2025. With default features it adds 9 crates to the lock, or 4 with
  `default-features = false`. nannou_osc 0.20.0 is a thin `UdpSocket` wrapper over rosc.
  `tokio-osc` and `rust-osc` do not exist, and `osc` 0.1.0 is an empty placeholder.
- **Encoding by hand is cheap.** An int32-only message encoder is about 40 lines (OSC 1.0
  "OSC Syntax"). Orca-c ships exactly that, in about 60 lines of C.
- **Timetags exist, but most receivers ignore them.** OSC 1.1 deliberately leaves timetag
  semantics unspecified.

## 1. Orca baseline

### Orca JS (`desktop/sources/scripts`)
| Aspect | Behaviour | Source |
|---|---|---|
| `;` payload | Runs every frame (passive). It reads Cells `x+1..x+36` and locks each one before testing for `.`, so the terminating `.` is locked too. It stops at the first `.`. Past the grid edge, `glyphAt` returns `''`, so the scan simply runs out. | `core/library.js:704-728`, `core/orca.js:72-80,176-178` |
| `;` send | Only when a `*` is orthogonally adjacent (`hasNeighbor('*')`). It pushes the raw glyph string, `*` included. | `core/library.js:719-722`, `core/operator.js:92-98` |
| `=` payload | The path is the one glyph at `x+1`. Arguments are read from `x+2..x+36` up to the first `.`, with the same lock-before-test order. An empty path or `.` sends nothing. | `core/library.js:672-702` |
| `=` arguments | Each glyph becomes `orca.valueOf(g)`: 0–35, case-insensitive. `.` and `*` give 0; any other non-base-36 glyph gives **-1** (`indexOf` miss). node-osc 4.1.9 encodes an integer JS number as `i` (int32). | `core/orca.js:160-162`, `core/io/osc.js:32-41`, node-osc `lib/Message.js:30-37` |
| Address | `'/' + glyph`, so the address space is one level of 36 names. | `core/library.js:696` |
| Timing | `client.run` calls `io.clear()`, then `orca.run()` (operators `push`), then `io.run()`, which sends the whole stack after the grid has run. The send is async `dgram` fire-and-forget. A forced run from the cursor sends immediately. | `client.js:155-160`, `core/io/udp.js:23-38`, `cursor.js:120` |
| Defaults | IP `127.0.0.1`. UDP out 49161, UDP in 49160, OSC out 49162. The sockets are `udp4`, so IPv4 only. | `core/io.js:10`, `core/io/udp.js:40,49`, `core/io/osc.js:9`, README "Default Ports" |
| Selection | Commander `udp:out;in`, `osc:port`, `ip:addr`. Ports below 1000 are refused, and the IP must be dotted-quad or contain `.local`. `Alt+O` and `Alt+U` open the prompt. | `commander.js:26-35`, `core/io.js:48-53,67`, `client.js:121-122` |
| Presets | `osc.js` lists `tidalCycles: 6010, sonicPi: 4559, superCollider: 57120, norns: 10111`, but only `options.default` is ever read. `osc:sonicPi` parses to `null` and is refused, so the presets are dead code. | `core/io/osc.js:9,43-45`, `commander.js:26,85` |
| Errors | `console.warn` in the send callback. Nothing reaches the user. | `core/io/udp.js:35-37`, `core/io/osc.js:38-40` |
| README drift | The README says `;hello` goes to port 49160. The code and the README's own port table say 49161. | README "UDP" vs "Default Ports" |

**UDP input.** A second `udp4` socket listens on 49160. Each datagram is passed whole to
`commander.trigger`, which reads `cmd:args`. An unknown command is warned and dropped
(`core/io/udp.js:49-70`, `commander.js:134-143`). This is how other programs drive Orca with
`play`, `bpm:`, `write:` and so on.
`resources/listener.js` is a test harness that listens on 49161 and 49162 and sends `run` to
49160.

### Orca-c (`sim.c`, `osc_out.c`, `tui_main.c`)
| Aspect | Behaviour | Source |
|---|---|---|
| `;` payload | `n = min(width - x - 1, 16)`. It scans east and locks each Cell until the first `.`, which is not locked. The scan and lock happen before the bang check. | `sim.c:323-339`, `vmio.h:40` |
| `=` operands | `PORT(0,1)` is the path glyph and `PORT(0,2)` is a **length** (base-36, capped at 35). Arguments sit at `x+3..x+3+len`. Every port is locked whether or not the operator is banged. | `sim.c:350-372`, `vmio.h:31` |
| `=` arguments | `index_of`: 0–35, and 0 for any non-base-36 glyph. This differs from JS, which gives -1. Sent as int32 big-endian. | `sim.c:26-34,363-365`, `osc_out.c:132-168` |
| Address | `{'/', glyph, '\0'}` | `tui_main.c:1151` |
| Timing | `orca_run` appends `Oevent`s to a list. After the VM run, `send_output_events` sends each one synchronously with `sendto`. | `tui_main.c:1324-1336,1146-1165` |
| Destination | One `Oosc_dev` serves both OSC **and** raw UDP, so `;` and `=` share a host and port. `getaddrinfo(host, port)` prefers IPv4; a null host gives loopback. | `osc_out.c:18-89`, `tui_main.c:1160-1164` |
| Selection | Main menu → "OSC Output…" with address, port, and enable/disable. Saved to `orca.conf` under `$XDG_CONFIG_HOME`, or `~/.config` when unset. There is no default port: output stays off until a port is set. The CLI has no host or port flag, only `--strict-timing` and `--osc-midi-bidule`. | `tui_main.c:2104-2116,2446,2775-2783,3036-3052`, `sysmisc.c:8,260`, README usage |
| Errors | The `sendto` result is discarded (`// TODO handle this in UI`). A socket setup failure shows "OSC Networking Error". | `osc_out.c:97-108`, `tui_main.c:2783-2785` |
| Extras | Sends `/orca/bpm`, `/orca/started` and `/orca/stopped` itself, plus MIDI-over-OSC for Plogue Bidule. | `tui_main.c:1457-1458,1839-1843,965-976` |
| Input | None. No `recvfrom` or `bind` in any `.c` file. | grep of the tree |

### What receivers expect (brief)
- **Pilot** listens for UDP on 49161 (hard-coded, `// TODO - make this configurable`). It splits
  each datagram on `;`, so `03C;13E` is a chord. Commands are base-36 glyph text such as
  `04Cf` (channel, octave, note, velocity) (`README.md:20`, `desktop/sources/scripts/listener.js:20`,
  `mixer.js:118`). Orca JS's UDP default port is Pilot's port.
- **SuperCollider (sclang)** listens on `NetAddr.langPort`, normally 57120. It maps `i` to
  Integer (32-bit) and `f` to Float, and an unknown tag becomes a Char that is read like a string
  (`OSC_communication.schelp:18-20,84-112`).
- **Sonic Pi** listens on 4560 and prefixes received addresses with `/osc*`
  (`12.1-Receiving-OSC.md:5,30`). Orca's `sonicPi: 4559` preset does not match the 4560 that
  Sonic Pi's current tutorial documents.
- **TouchDesigner** (OSC In CHOP or DAT, user-set port, int and float args as channels):
  **UNVERIFIED**. docs.derivative.ca was unreachable from this environment.
- **No standard port.** OSC has no IANA-registered port (OSC 1.1 paper §5), so every receiver
  picks its own.

## 2. Rust building blocks

### Fit with the existing code
- The Playback Engine runs as a task that sleeps with `tokio::time::sleep_until`
  (`orcvs/src/playback.rs:1199-1210,1317-1318`). It calls `OutputAdapter::submit(&[OutputCommand])`
  **synchronously** once per executed Tick (`playback.rs:89-92,915-921`).
- midir is target-gated to native desktop (`console/Cargo.toml:137-138`). The console opens the
  connection on its own thread and installs it into Playback (`console/src/console_midi.rs:1-6`).
- `OutputCommand` is MIDI-only today (`playback.rs:60-85`).
- tokio is locked at 1.53.3. `orcvs` enables `macros, rt, sync, time`, and the console adds
  `rt-multi-thread` (`orcvs/Cargo.toml:60`, `console/Cargo.toml:165`). `net`, `mio` and `socket2`
  are not in `Cargo.lock`. The workspace MSRV is 1.99 on toolchain 1.99.1.

### `std::net::UdpSocket` (Rust 1.99.1)
- `connect` fixes the peer, after which `send` needs no address. `send_to` sends only to the
  *first* address the `ToSocketAddrs` input resolves to, and fails if the IP family differs
  from the bound socket's. A first `connect` to loopback can make later non-loopback
  `connect`s fail on some platforms (`library/std/src/net/udp.rs:190-194,670-683`).
  This means the bind address has to follow the destination: binding `0.0.0.0:0` for an IPv4
  target and `[::]:0` for an IPv6 one.
- `set_nonblocking(true)` makes a full send buffer return `WouldBlock` instead of blocking
  (`udp.rs:788-822`). On Linux and macOS a UDP send to a local buffer rarely blocks.
  Whether a blocking `send_to` can stall the Tick on any platform is **UNVERIFIED**; measure it.
- A send larger than 65507 bytes may be refused (`udp.rs:184,689`). An Orca-sized payload is far
  below that.
- Windows: the `WSAECONNRESET`-on-unreachable-port behaviour and `set_broadcast` are in
  `windows-support.md` §2, "OSC/UDP (post-v1)". Do not treat `ConnectionReset` as fatal.
- **wasm32-unknown-unknown.** std routes every non-listed target to `connection/unsupported.rs`,
  where `UdpSocket::bind` returns `unsupported()` (`sys/net/connection/mod.rs:1-36`,
  `unsupported.rs:182-183`). I built a probe with 1.99.1 for wasm32 and ran it under node.
  `UdpSocket::bind` returned `ErrorKind::Unsupported`, while `"127.0.0.1:9000".to_socket_addrs()`
  succeeded. So the code compiles cleanly and fails at runtime. A `cfg(not(target_arch = "wasm32"))`
  gate, the same shape as the midir and rfd gates, keeps it out of the web build altogether.

### `tokio::net::UdpSocket`
- `net` pulls in `mio` (`os-poll`, `net`), `socket2`, and Windows `windows-sys` features
  (tokio 1.53.3 `Cargo.toml:96-106`). Without `tokio_unstable`, any tokio feature beyond
  `sync, macros, io-util, rt, time` **fails to compile on WASM** (`src/lib.rs:425-437`), so
  `net` would have to be target-gated.
- It offers `try_send_to`, which never awaits (`src/net/udp.rs:1272`). Since `submit` is
  synchronous and the std socket already gives a non-blocking send, there is no evidence it
  would add anything.

### OSC crates (crates.io API, 2026-10-26)
| Crate | Latest | Released | Licence | MSRV | Deps | Notes |
|---|---|---|---|---|---|---|
| `rosc` | 0.11.4 | 2025-03-23 | MIT/Apache-2.0 | 1.67.1 | default: byteorder, nom 7 (+minimal-lexical, memchr), time 0.3 (+deranged, num-conv, powerfmt, time-core). That is 10 crates, 9 of them new to this lock. With `default-features = false`: byteorder, nom, minimal-lexical, memchr. | `OscType` covers i f s b t h d c r m T/F N I and arrays. `OscBundle` has `OscTime`, with `TryFrom<SystemTime>` under `std`. `encode`, `encode_into<O: Output>`, `encode_tcp`. Supports `no_std` + `alloc`. The encoder does not validate the address (`verify_address` is separate). The repo's last commit is `711945b`, dated 2025-03-23. |
| `nannou_osc` | 0.20.0 | 2026-06-22 | MIT OR Apache-2.0 | none declared (edition 2024) | rosc 0.11 | `Sender` wraps `std::net::UdpSocket` with `bind_to`, `connect`, `send` and `send_to`. Re-exports rosc. Releases are years apart (0.19.0 was 2024-01-17). |
| `async-osc` | 0.2.0 | 2021-02-20 | MIT OR Apache-2.0 | — | async-std stack | Unmaintained. |
| `tinyosc` | 0.0.3 | 2016-03-26 | MIT | — | — | Abandoned. |
| `osc` | 0.1.0 | 2023-11-16 | MIT OR Apache-2.0 | — | none | Placeholder: `lib.rs` is the cargo-new `add` template. |
| `osc-types10/11` | 0.1.0-alpha.2 | 2025-10-22 | MIT OR Apache-2.0 | 1.70 | — | Alpha, no stable release. |
| `open-sound-control` | 0.9.0 | 2026-01-31 | MIT | — | — | One release, 14 recent downloads. Not evaluated further. |
| `tokio-osc`, `rust-osc` | — | — | — | — | — | Do not exist on crates.io (API 404). |

**rosc on wasm32.** A probe using `rosc = "=0.11.4"` (default features) together with
`std::net::UdpSocket` builds for `wasm32-unknown-unknown` with 1.99.1.
`encode` of `/a` with `Int(1)` gives the 12 bytes `2f 61 00 00 2c 69 00 00 00 00 00 01`.
`SystemTime::now()` is not supported on wasm32-unknown-unknown, so a timetag built from it at
runtime would fail there. Whether it panics or errors is **UNVERIFIED**. This is moot if the
code is native-gated.

### Hand-encoding (OSC 1.0)
- Atomic types: int32 and float32 are big-endian. An OSC-string is ASCII plus a null, padded
  with nulls to a multiple of 4 bytes. A packet's size is always a multiple of 4. One UDP
  datagram carries one packet ("Atomic Data Types", "OSC Packets").
- A message is the address pattern (an OSC-string starting with `/`), then the type tag string
  (`,` followed by one tag per argument, padded the same way), then the arguments
  ("OSC Messages", "OSC Type Tag String"). For Orcvs's case, every tag would be `i`.
- Address names may not contain space, `#`, `*`, `,`, `/`, `?`, `[`, `]`, `{` or `}`
  ("OSC Address Spaces and OSC Addresses"). An Orcvs glyph used as an address segment must
  avoid these, and `*` matters because it is Orca's bang glyph.
- A bundle is `#bundle`, then an 8-byte timetag (NTP format: 32-bit seconds since 1900 plus a
  32-bit fraction, where the value 1 means "immediately"), then size-prefixed elements
  ("OSC Bundles", "Temporal Semantics and OSC Time Tags").
- OSC 1.1 keeps `i f s b` required and adds `T F N I t`. It requires SLIP framing on streams
  only (§3.5, §4). It **declines to specify timetag semantics**, citing three incompatible uses
  in the wild, including "simply ignored" (§3.2). Desktop OSes cannot guarantee low-jitter
  delivery (§3.6). There is no published 1.1 specification document, only the NIME 2009 paper
  (`spec-1_1.md`).
- Size: Orca-c's int32-only encoder is `osc_out.c:110-168`, about 60 lines of C including bounds
  checks. A Rust equivalent writing into a `Vec<u8>` or a stack buffer is about 40 lines plus
  tests, adds no crate, and every byte can be pinned in a unit test against the spec's examples.
  A rosc dev-dependency could serve as a decode oracle in tests without entering the shipped graph.

### Timetags for Tick-accurate delivery
A bundle stamped with a Tick's due time plus a fixed latency would let a receiver that honours
timetags (scsynth's documented use case) remove sender jitter. This needs a map from the
engine's monotonic `ClockInstant` to wall-clock NTP time, and it gains nothing on receivers that
execute on receipt. Per OSC 1.1 §3.2, that may be most of them. Which of Sonic Pi, sclang and
TouchDesigner honour bundle timetags is **UNVERIFIED** here.

## Implications for the open questions
- **Payload extent.** Orca has two precedents. One scans east to `.` with a cap (JS 36 or 35,
  C 16), locks every Cell it reads, and treats `.` as both terminator and filler. The other is
  an explicit length operand (Orca-c `=`). Both lock the payload even when unbanged. The C
  probe already shows a `*` inside a locked payload is preserved and still bangs
  (`orca-bang-reference.md:61-66`). With two-Cell values, a scan would step in pairs, so `..`
  would be the natural terminator, and a fixed or declared arity is what the Parser can see
  without evaluating.
- **OSC address and argument typing.** Orca's address is one glyph, so 36 flat names, and every
  argument is int32. Two-Cell Numbers map onto int32 with nothing lost. A Note has no OSC
  type, so it needs a defined number. Multi-segment paths, floats and strings have no Orca
  precedent and would need the deferred text or message value (ADR 0008). Address glyphs must
  avoid the OSC reserved set.
- **UDP payload.** Orca sends glyph text verbatim, and Pilot consumes exactly that. Orcvs has no
  text value yet, so `!u` either waits for one or defines a byte encoding of Cells.
- **Destination configuration.** The precedents are one global IP with per-protocol ports (JS,
  not persisted), or a single host and port shared by UDP and OSC (C, persisted, off by
  default). There is no standard OSC port. The receivers above want 49161, 57120 and 4560.
  JS's presets show demand for receiver presets but never shipped working ones.
- **Delivery path and timing.** Both Orcas collect events during the frame and flush them
  after it. That is the shape of Tick Plan → `OutputAdapter::submit` per Tick. A synchronous
  `std::net::UdpSocket::send` in a native-gated adapter matches midir's synchronous send and
  needs no runtime change. Both Orcas swallow send errors, while Orcvs already has
  `PlaybackDiagnostic::OutputFailure` for surfacing them. Bundles with timetags are an opt-in
  refinement whose value depends on the receiver.
- **Inbound is out of scope.** Only Orca JS has UDP input, a commander bridge on 49160, and
  Orca-c has none. Leaving it out avoids the Windows firewall prompt that listening triggers
  (`windows-support.md` §2).

## Sources
- Orca JS @ `223e45c84db14cb77fb1d31e29a3ad94652080d8`: https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/desktop/sources/scripts/core/library.js , …/core/io/udp.js , …/core/io/osc.js , …/core/io.js , …/core/orca.js , …/core/operator.js , …/client.js , …/commander.js , …/cursor.js , https://github.com/hundredrabbits/Orca/blob/223e45c84db14cb77fb1d31e29a3ad94652080d8/README.md , …/resources/listener.js , …/desktop/package-lock.json (node-osc 4.1.9)
- node-osc 4.1.9: https://registry.npmjs.org/node-osc/-/node-osc-4.1.9.tgz (`lib/Message.js`, `lib/Client.js`)
- Orca-c @ `9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc`: https://github.com/hundredrabbits/Orca-c/blob/9df9786e2ad3c01955cdf4cdd5ae1fffad8fa5cc/sim.c , …/vmio.h , …/osc_out.c , …/tui_main.c , …/sysmisc.c , …/gbuffer.h , …/README.md
- Pilot @ `a77a4e059147d02fbbfb676e2417974a587dbaed`: https://github.com/hundredrabbits/Pilot/blob/a77a4e059147d02fbbfb676e2417974a587dbaed/README.md , …/desktop/sources/scripts/listener.js , …/desktop/sources/scripts/mixer.js
- OSC 1.0 spec (canonical https://opensoundcontrol.stanford.edu/spec-1_0.html , unreachable from here; read from its source repo): https://github.com/CNMAT/OpenSoundControl.org/blob/fb7b3b48ba9ac64eae030e3333f9a980f4f8fd59/spec-1_0.md ; OSC 1.1: …/spec-1_1.md and …/files/2009-NIME-OSC-1.1.pdf
- SuperCollider: https://github.com/supercollider/supercollider/blob/65ff08335e3f660f74f446ec2889526946d9b17d/HelpSource/Guides/OSC_communication.schelp
- Sonic Pi: https://github.com/sonic-pi-net/sonic-pi/blob/df6f17e4625b0451fa2af8accf2e1e641c7ad458/etc/doc/tutorial/12.1-Receiving-OSC.md
- Rust std @ 1.99.1 (`213cfbe`): https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/net/udp.rs , …/library/std/src/sys/net/connection/mod.rs , …/library/std/src/sys/net/connection/unsupported.rs , …/library/std/src/sys/pal/unsupported/common.rs
- tokio 1.53.3: https://static.crates.io/crates/tokio/tokio-1.53.3.crate (`Cargo.toml`, `src/lib.rs`, `src/net/udp.rs`)
- rosc 0.11.4: https://static.crates.io/crates/rosc/rosc-0.11.4.crate , https://github.com/klingtnet/rosc , https://crates.io/api/v1/crates/rosc
- nannou_osc 0.20.0: https://static.crates.io/crates/nannou_osc/nannou_osc-0.20.0.crate ; osc 0.1.0: https://static.crates.io/crates/osc/osc-0.1.0.crate
- crates.io API (metadata, keyword `osc`, 404s for `tokio-osc` and `rust-osc`): https://crates.io/api/v1/crates?keyword=osc , https://crates.io/api/v1/crates/{async-osc,tinyosc,osc-types10,open-sound-control,tokio-osc,rust-osc}
- Local probes (scratch only, not committed): `cargo tree` for rosc with and without default features; a wasm32 build of rosc + `std::net`; a wasm32 cdylib calling `UdpSocket::bind`, run under node.
