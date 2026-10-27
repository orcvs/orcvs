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
  // How late a message that arrived while the audio context started can be
  // and still sound once it runs. Starting takes a device a few hundred
  // milliseconds at most; a message older than that is dropped, so a slow
  // start does not sound a backlog at once.
  const START_MS = 500;

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

  // iOS silences Web Audio under the ring/silent switch unless the page says it plays media.
  if (navigator.audioSession) navigator.audioSession.type = "playback";

  // Safari resumes a context only inside the reader's gesture, and the message a console posts
  // after handling a tap arrives outside it. So every gesture, on this page or in a console's
  // frame, resumes the context while it is still the gesture. Capturing listeners run before the
  // console's own, which stop a touch from propagating.
  function resume() {
    if (context && context.state !== "running") context.resume().catch(() => {});
  }
  function resumeOnGestures(target) {
    for (const type of ["pointerdown", "touchend", "keydown"]) {
      target.addEventListener(type, resume, { capture: true });
    }
  }
  resumeOnGestures(window);

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

  // The messages that arrived while the context starts.
  let starting;

  // Whether `message` is at most `START_MS` past its planned instant.
  function due(message) {
    return message.time + START_MS >= performance.timeOrigin + performance.now();
  }

  function play(message) {
    node.port.postMessage({ bytes: message.bytes, when: contextTime(message.time) });
  }

  window.addEventListener("message", async (event) => {
    if (event.origin !== window.location.origin || !event.source) return;
    const { data } = event;
    if (data?.type === "orcvs.ready") {
      // The console's frame is of this origin, so its gestures can be heard here.
      resumeOnGestures(event.source);
      event.source.postMessage(await announcement, event.origin);
    } else if (data?.type === "orcvs.midi" && node && Array.isArray(data.bytes) && typeof data.time === "number") {
      // Where the gesture's own resume has not run yet, the reader's gesture
      // that started Playback activates this page too, so the first message
      // of a run can resume the context. Messages that
      // arrive while it starts wait for it, and those still due sound as soon
      // as it runs.
      if (context.state === "running") {
        play(data);
        return;
      }
      if (starting) {
        starting.push(data);
        return;
      }
      starting = [data];
      context.resume().then(
        () => {
          for (const message of starting.filter(due)) play(message);
          starting = undefined;
        },
        (reason) => {
          starting = undefined;
          console.warn("The Built-in Synth cannot start:", reason);
        },
      );
    }
  });
})();
