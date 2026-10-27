// The page's Built-in Synth. A page that frames consoles loads
// this beside them, from the same directory as the console:
//
//   <script src="/play/synth-host.js"></script>
//
// It answers each console's `{ type: "orcvs.ready" }` with
// `{ type: "orcvs.synth", available, reason }`, and plays each
// `{ type: "orcvs.midi", bytes, time }` a console posts a fixed delay after
// `time`, the message's planned instant as `performance.timeOrigin` plus
// `performance.now()`. Only frames of the page's own origin are answered.
(() => {
  // How long after its planned instant a message sounds. A message that
  // reaches the synth later than this sounds at once.
  const DELAY_MS = 50;

  const here = document.currentScript.src;
  let context;
  let node;

  // The synth, ready to play once the audio context runs, or why this page
  // cannot make sound.
  const ready = (async () => {
    if (typeof AudioWorkletNode === "undefined" || typeof WebAssembly === "undefined") {
      throw new Error("this browser has no AudioWorklet or WebAssembly");
    }
    context = new AudioContext({ latencyHint: "interactive" });
    const [module] = await Promise.all([
      WebAssembly.compileStreaming(fetch(new URL("synth.wasm", here))),
      context.audioWorklet.addModule(new URL("synth-worklet.js", here)),
    ]);
    node = new AudioWorkletNode(context, "orcvs-built-in-synth", {
      numberOfInputs: 0,
      outputChannelCount: [2],
      processorOptions: { module },
    });
    node.connect(context.destination);
  })();
  ready.catch((reason) => console.warn("The Built-in Synth cannot play:", reason));

  // The AudioContext time a page instant `time` falls at, from the context's
  // latest pairing of a time it is playing with the page's clock.
  function contextTime(time) {
    const playing = context.getOutputTimestamp();
    const planned = time - performance.timeOrigin + DELAY_MS;
    // A context that has not played yet pairs nothing; its render time less
    // its latency is the nearest stand-in.
    if (!playing.performanceTime) {
      const playingNow = context.currentTime - context.baseLatency - (context.outputLatency || 0);
      return playingNow + (planned - performance.now()) / 1000;
    }
    return playing.contextTime + (planned - playing.performanceTime) / 1000;
  }

  const announcement = ready.then(
    () => ({ type: "orcvs.synth", available: true }),
    (reason) => ({ type: "orcvs.synth", available: false, reason: String(reason.message ?? reason) }),
  );

  // A console that said it was ready before this script listened is told too. Frames of another
  // origin are not: the target origin keeps the message from them.
  announcement.then((answer) => {
    for (let index = 0; index < window.frames.length; index++) {
      window.frames[index].postMessage(answer, window.location.origin);
    }
  });

  window.addEventListener("message", async (event) => {
    if (event.origin !== window.location.origin || !event.source) return;
    const { data } = event;
    if (data?.type === "orcvs.ready") {
      event.source.postMessage(await announcement, event.origin);
    } else if (data?.type === "orcvs.midi" && node && Array.isArray(data.bytes) && typeof data.time === "number") {
      // The reader's gesture that started Playback activates this page too,
      // so the first message of a run can resume the context.
      if (context.state !== "running") {
        context.resume().catch((reason) => console.warn("The Built-in Synth cannot start:", reason));
      }
      node.port.postMessage({ bytes: data.bytes, when: contextTime(data.time) });
    }
  });
})();
