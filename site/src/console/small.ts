// The page's small console (ADR 0086): `/play/?small`, started on the first press of a Grid and
// sent each pressed Grid's Source by message, so every example opens in the one console.

/** How long the console has to say it is ready before the page gives up on it. */
const START_LIMIT_MS = 30_000;

let frame: HTMLIFrameElement | undefined;
let started: Promise<void> | undefined;
let failure: HTMLElement | undefined;

// The console posts `{ type: "orcvs.ready" }` once it listens for Sources, and drops what is
// sent before that.
function start(): Promise<void> {
  failure?.remove();
  const starting = document.createElement("iframe");
  starting.src = "/play/?small";
  starting.title = "Orcvs console";
  starting.className = "orcvs-small";
  frame = starting;
  return new Promise((resolve, reject) => {
    const ready = (event: MessageEvent) => {
      if (event.source !== starting.contentWindow || event.origin !== location.origin) return;
      if (event.data?.type !== "orcvs.ready") return;
      window.removeEventListener("message", ready);
      clearTimeout(limit);
      resolve();
    };
    const limit = setTimeout(() => {
      window.removeEventListener("message", ready);
      reject(new Error(`the console did not start within ${START_LIMIT_MS} ms`));
    }, START_LIMIT_MS);
    window.addEventListener("message", ready);
    document.body.append(starting);
  });
}

// A console that did not start is removed, so the next press starts another.
function fail(reason: unknown) {
  console.error(reason);
  frame?.remove();
  frame = undefined;
  started = undefined;
  failure = document.createElement("p");
  failure.className = "orcvs-small orcvs-small-failed";
  failure.setAttribute("role", "status");
  failure.textContent = "The console did not start.";
  document.body.append(failure);
}

for (const grid of document.querySelectorAll<HTMLElement>(".orcvs-grid[data-source]")) {
  grid.addEventListener("click", async () => {
    started ??= start();
    try {
      await started;
    } catch (reason) {
      if (started) fail(reason);
      return;
    }
    frame!.contentWindow!.postMessage({ type: "orcvs.open", source: grid.dataset.source }, location.origin);
    frame!.focus();
  });
}
