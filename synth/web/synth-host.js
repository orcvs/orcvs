// The page's Built-in Synth. A page that frames consoles loads
// this beside them, from the same directory as the console:
//
//   <script src="/play/synth-host.js"></script>
//
// It answers each console's `{ type: "orcvs.ready" }` with
// `{ type: "orcvs.synth", available, reason }`, and plays each
// `{ type: "orcvs.midi", bytes, time }` a console posts a fixed delay after
// `time`, the message's planned instant as `performance.timeOrigin` plus
// `performance.now()`. A console's `{ type: "orcvs.wake" }`, posted on the
// reader's click or key, starts the audio context, and every console is told
// `{ type: "orcvs.audio", running }` whenever the context starts or stops, so
// a console can hold its first Tick until the audio runs. Only frames of the
// page's own origin are answered.
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

  // Safari resumes a context only inside the reader's gesture, and the message a console posts
  // after handling a tap arrives outside it. So every gesture, on this page or in a console's
  // frame, resumes the context while it is still the gesture. Capturing listeners run before the
  // console's own, which stop a touch from propagating. A gesture that is not an activation, such
  // as the end of a scroll, gets a refused resume and nothing else. Adding the same listener to a
  // target twice adds it once, so a console that says it is ready again adds nothing.
  function resume() {
    if (!context || context.state === "running") return;
    // iOS silences Web Audio under the ring/silent switch unless the page says it plays media. It
    // is said only once the reader acts, as it takes the device's audio from other apps.
    if (navigator.audioSession) navigator.audioSession.type = "playback";
    context.resume().catch(() => {});
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

  // Tells every frame of this origin `message`. Frames of another origin are not told: the target
  // origin keeps the message from them.
  function tell(message) {
    for (let index = 0; index < window.frames.length; index++) {
      window.frames[index].postMessage(message, window.location.origin);
    }
  }

  // A console that said it was ready before this script listened is told too.
  announcement.then(tell);

  const audio = () => ({ type: "orcvs.audio", running: context.state === "running" });
  ready.then(() => {
    context.addEventListener("statechange", () => tell(audio()));
    tell(audio());
  }, () => {});

  // The messages that arrived while the context starts.
  let starting;

  // Whether `message` is at most `START_MS` past its planned instant.
  function due(message) {
    return message.time + START_MS >= performance.timeOrigin + performance.now();
  }

  function play(message) {
    node.port.postMessage({ bytes: message.bytes, when: contextTime(message.time) });
  }

  // Starts the audio context, then plays the messages that arrived meanwhile and are still due.
  function start() {
    if (starting) return;
    starting = [];
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

  window.addEventListener("message", async (event) => {
    if (event.origin !== window.location.origin || !event.source) return;
    const { data } = event;
    if (data?.type === "orcvs.ready") {
      // The console's frame is of this origin, so its gestures can be heard here.
      resumeOnGestures(event.source);
      event.source.postMessage(await announcement, event.origin);
      if (node) event.source.postMessage(audio(), event.origin);
    } else if (data?.type === "orcvs.wake" && node && context.state !== "running") {
      // The reader's gesture in the console activates this page too, so it can start the context.
      start();
    } else if (data?.type === "orcvs.midi" && node && Array.isArray(data.bytes) && typeof data.time === "number") {
      // Where no gesture has resumed the context yet, the first message of a
      // run does. Messages that arrive while it starts wait for it, and those
      // still due sound as soon as it runs.
      if (context.state === "running") {
        play(data);
        return;
      }
      start();
      starting.push(data);
    }
  });
})();
