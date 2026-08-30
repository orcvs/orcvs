# Terminology for the playback-side module

## Recommendation

Call the module the **Playback Engine**.

In Orca, the Playback Engine owns playback lifecycle and musical time, accepts an ordered batch of already interpreted `Play Command`s for each tick, and dispatches that batch to output adapters such as MIDI. It does not parse or interpret Source.

This name is the closest fit even though no surveyed term names this exact combination perfectly. “Playback” says that the module realizes prepared musical instructions over time; “engine” signals that it coordinates timing and output behind one module interface. Keep its internal clock/scheduler and its MIDI adapter as subordinate modules rather than naming the whole module after either mechanism.

## What the nearby terms conventionally mean

| Candidate | Conventional scope | Fit for Orca |
| --- | --- | --- |
| **Playback engine** | The machinery that advances and realizes playback. JUCE's `AudioTransportSource` combines start/stop, current playback position, and reading the next audio block from a source; Ableton separately speaks of transport controls triggering playback. | **Best fit.** It covers lifecycle, timing, and dispatch without implying Source interpretation. Document the narrower Orca definition above. |
| **Transport** | Playback/record control state and timeline position: play, stop, record, seek, loop/cycle, tempo, and current position. Ableton calls play/stop/record the transport controls; VST exposes playing, recording, cycle, tempo, time signature, and project position as transport/process context; REAPER exposes play state, play position, cursor, and repeat state. | Too narrow for command dispatch. Use `Transport` for the Playback Engine's state/control submodule or interface. |
| **Scheduler** | A time-ordered queue that wakes work at logical or physical times. SuperCollider's `Clock` “keeps track of time and allows tasks to be scheduled”; `TempoClock` is explicitly a tempo-based scheduler in beats. | Too narrow for lifecycle plus adapter dispatch. Use `Scheduler` (or `Clock`) inside the Playback Engine. |
| **Sequencer** | A musical timeline or facility that arranges/launches stored musical material. Ableton describes Arrangement View as the fixed song timeline used by traditional sequencing programs, and separately describes step sequencing notes. | Misleading here: Orca Source interpretation produces each tick's commands; this module neither stores nor interprets a sequence. Reconsider only if it later owns tracks/clips/patterns or an editable event timeline. |
| **Audio engine** | Real-time audio processing and device/buffer flow. VST calls `IAudioProcessor` the processing part and processes audio in blocks; JUCE's `AudioDeviceManager` continuously streams through an audio callback and manages audio/MIDI I/O devices. Ableton even gives “Audio Engine” its own on/off control, separate from Transport. | Too audio-specific for a module currently dispatching MIDI commands, and likely to be confused with a future sample/DSP engine. Reserve it for real-time sample processing and audio devices. |
| **MIDI engine** | Not a stable cross-system role in the surveyed primary documentation. JUCE calls the relevant parts MIDI output/device facilities, while its device manager handles I/O. | Too adapter-specific. Prefer `MidiOutput` or `MidiAdapter` behind the Playback Engine. |
| **Runtime** | A general software execution term, not a DAW-specific role in the surveyed material. | Accurate only in the abstract and too broad: it could easily absorb Source interpretation, UI orchestration, or persistence. Avoid it as the domain name. |

## Suggested vocabulary

```text
Source interpretation
  -> Tick Plan { source writes, ordered Play Commands, diagnostics }
  -> commit source writes
  -> Playback Engine.submit(play_commands)
       Transport       (play/stop state and position)
       Clock/Scheduler (tick timing)
       MidiOutput      (adapter and delivery)
```

All `Play Command`s in one submitted batch are logically simultaneous; list order is retained for deterministic dispatch. This vocabulary leaves room for later output adapters without turning MIDI or audio into the architectural seam.

## Primary sources

- [Ableton Live 12 — Live Concepts](https://www.ableton.com/en/manual/live-concepts/) distinguishes transport controls (starting/stopping playback and recording) from other system concerns.
- [Ableton Live 12 — Session View](https://www.ableton.com/en/manual/session-view/) contrasts the Arrangement's fixed song timeline, characteristic of traditional sequencing programs, with clip launching.
- [JUCE `AudioTransportSource`](https://docs.juce.com/develop/classjuce_1_1AudioTransportSource.html) owns start/stop, playback position, and supplying the next audio block.
- [JUCE `AudioDeviceManager`](https://docs.juce.com/master/classjuce_1_1AudioDeviceManager.html) manages audio/MIDI I/O devices and continuously streams through audio callbacks.
- [VST 3 `ProcessContext`](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1ProcessContext.html) defines transport state alongside project time, tempo, time signature, cycle state, and MIDI clock position.
- [VST 3 API documentation](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical%2BDocumentation/API%2BDocumentation/Index.html) defines real-time audio processing in blocks as the processor's responsibility.
- [SuperCollider `Clock`](https://docs.supercollider.online/Classes/Clock.html) defines clock scheduling; [SuperCollider `TempoClock`](https://docs.supercollider.online/Classes/TempoClock.html) specializes it as a tempo-based scheduler in beats.
- [REAPER VST extensions](https://www.reaper.fm/sdk/vst/vst_ext.php) exposes transport-like play state, playback position, cursor position, and loop/repeat state.
