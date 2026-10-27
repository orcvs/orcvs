// The Built-in Synth's AudioWorklet processor. It runs the synth
// core's WebAssembly module on the browser's audio thread, so neither the
// page's work nor the console's frames can starve it.
//
// `processorOptions.module` is the compiled `synth.wasm`, which imports
// nothing. Each message on the port is `{ bytes, when }`: one MIDI message
// and the AudioContext time it sounds at. A time already rendered sounds at
// once.
class BuiltInSynth extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.synth = new WebAssembly.Instance(options.processorOptions.module, {}).exports;
    this.synth.synth_start(sampleRate);
    // The context frame the synth's own frame clock counts from. The synth
    // renders one frame for each the context renders from here on, and none
    // before its first `process`: a suspended context does not advance
    // `currentFrame` either, so the two clocks stay one offset apart.
    this.origin = currentFrame;
    this.port.onmessage = ({ data: { bytes, when } }) => {
      const frame = Math.round(when * sampleRate) - this.origin;
      this.synth.synth_schedule(frame, bytes[0] ?? 0, bytes[1] ?? 0, bytes[2] ?? 0);
    };
  }

  process(_inputs, [output]) {
    const frames = output[0].length;
    const at = this.synth.synth_render(frames);
    const samples = new Float32Array(this.synth.memory.buffer, at, frames);
    for (const channel of output) channel.set(samples);
    return true;
  }
}

registerProcessor("orcvs-built-in-synth", BuiltInSynth);
