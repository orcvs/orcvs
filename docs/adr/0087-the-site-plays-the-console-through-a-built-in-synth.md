# The site plays the console through a Built-in Synth

Status: accepted; when the audio context resumes is amended by [ADR 0089](0089-the-panel-plays-and-rewinds.md), which resumes it inside the reader's gesture. Settled with Toby on 2026-10-26 in `.scratch/orcvs-site/` (tickets 07 and 08).
The research is `docs/research/tone-generation.md`. [ADR 0088](0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md)
amends it: Control Change 1 to 8 set the synth's eight effects, which the sound module shows. [ADR 0096](0096-one-built-in-synth-channel-plays-a-303-style-voice.md)
amends it: channel `09` plays a 303-style voice in place of the kit, with its own Control Changes.
Ticket 16 amended when the audio context starts (2026-10-28).

The site shows Orcvs running, and an example nobody hears teaches half of what it shows. Most
readers have no MIDI device, and the console only sends MIDI. Learn Orca makes sound by patching
Orca's MIDI output into Enfer, a Tone.js synth beside it on the page.

## Decision

**The page's sound module holds a synth, the Built-in Synth, and the console plays it by message.** A
page that frames the console says it holds one by posting `{ type: "orcvs.synth", available }` to
the console, after the console's `orcvs.ready` (ADR 0086). While one is announced, the console
offers it under **O** as "Built-in Synth", listed after every MIDI port. It is chosen like any
destination, and only when no port exists is it picked automatically. A Linked or small console
always picks it. Choosing it sends nothing to a device: each message Playback hands the destination
is posted to the framing page as `{ type: "orcvs.midi", bytes, time }`, from the console's own
origin to that page only.

**A message sounds at its Tick's planned time, not when it arrives.** `time` is the Tick's planned
instant on the page's clock, `performance.timeOrigin` plus `performance.now()`, which both frames
share. The synth plays it a fixed delay after that time, so a late timer or a slow frame moves no
note until the lateness exceeds the delay. The delay is set within a 50 ms target. A message whose
time has already passed by more than the delay sounds at once.

**One synth core in Rust renders the sound in an AudioWorklet.** The core is a crate with no audio
I/O: it takes MIDI bytes and a time, and fills a buffer. On the web it is compiled to its own small
WASM module that runs inside an AudioWorklet on the browser's audio thread, so neither the page's
work nor the console's frames can starve it. The page passes each `orcvs.midi` on to the worklet's
port. The web build ships first. Native gets the same core later, through cpal, in its own ticket.

**What the synth answers.**

- **Channels `00`–`07` play tones:** `00` sine, `01` triangle, `02` saw, `03` square, `04` and `05`
  two-operator FM (a soft bell and a bright bass), `06` and `07` AM at harmonicity 1.5 and 2, after
  Pilot's AM voices. They share one short envelope. A Note Off releases a tone over 150 ms. One
  channel can sound a chord.
- **Channels `08`–`0F` play an 808-style kit,** synthesised from what the 808's circuits do, with no
  recordings. The Note's pitch class picks the instrument and its octave is ignored, in General
  MIDI's order from C: kick, rim, snare, clap, cowbell, low tom, closed hat, mid tom, cymbal, high
  tom, open hat, conga. A drum rings out its own decay and ignores Note Off. A closed hat chokes an
  open hat on its channel.
- **Velocity sets loudness.** There is no other level control: no volume, and no Control Change
  but All Sound Off and All Notes Off, which release every tone on the channel. Pitch Bend and
  every other message are ignored.
- **Sixteen voices sound at once.** A Note On past that takes the oldest voice.

**The audio context starts on the reader's first gesture in a console, and Tick 0 waits for it.** A
user activation in the console's frame activates the page that frames it. The console posts
`{ type: "orcvs.wake" }` on each click and key, and the page resumes its audio context on it, or on
the first `orcvs.midi` of a run if none came first. Starting takes a device over 100 ms, so the
page tells every console `{ type: "orcvs.audio", running }` as its context starts and stops, and a
console holds a start on the Built-in Synth until the audio runs, for at most 500 ms. Otherwise
Tick 0 would sound late and the Ticks after it on time (ticket 16).

**A page that cannot make sound still announces the synth,** with `available: false` and a reason.
The console lists it, and choosing it answers the reason as a connect failure, the way a MIDI
device that has gone answers. Playback runs on silently.

## Rejected alternatives

**A Tone.js iframe bridged from JavaScript, as Learn Orca's Enfer is.** It puts a second synth
language beside the console, Tone.js plays only in a browser, so native could never share it, and
its default schedule runs 100 ms ahead.

**Porting Enfer.** It has no licence, and its sound is 29 MB of sampled drum machines.

**The synth inside the console, through cpal's Web Audio host.** cpal renders on the page's main
thread from timers, in 2048-frame buffers by default. Measured in the timing prototype, notes
snapped to buffer edges up to 43 ms apart; stamping them inside the buffer fixed the spread but
cost 158 ms at the default size and 62 ms at 512 frames, and after a stalled frame cpal never
re-synchronised, so every later buffer stayed late. Its AudioWorklet host needs nightly Rust and
cross-origin isolation.

**Web Audio nodes driven from Rust through `web-sys`.** It gives the web a synth native cannot
reuse.

**A SoundFont player.** A General MIDI font is tens of megabytes for a site's examples.

**A volume or effects control now.** The system volume serves until the sound module grows an
effects panel, which belongs to the sound module rather than the console.

## Consequences

- The console has no audio code and gains one destination and two messages. A console that no
  page frames, natively or at `/play/` on its own, offers no Built-in Synth until native gets the
  core.
- Every console on a page plays through that page's one synth. Two consoles playing at once share
  its sixteen voices.
- `orcvs.midi` carries the bytes Playback hands any destination, so Stop's safety action reaches the
  synth as All Notes Off on every channel: tones release and drums ring out.
- `MidiConnection::send` carries no time today. Playback hands the planned Tick time to the
  destination with the message, which every other destination is free to ignore.
- The synth core's WASM and worklet script are built with the web console and shipped beside it in
  `console-web`, so the site's build still compiles no Rust.
- The effects panel, when it comes, sits in the sound module and changes the core's mix; whether
  Control Change also reaches it is that panel's decision.
