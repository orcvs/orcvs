// The page's small console (ADR 0085): `/play/?small`, started on the first press of a Grid and
// sent each pressed Grid's Source by message, so every example opens in the one console.

let frame: HTMLIFrameElement | undefined;
let started: Promise<void> | undefined;

// eframe sets the canvas's `width` attribute once the console runs and listens.
function start(): Promise<void> {
  frame = document.createElement("iframe");
  frame.src = "/play/?small";
  frame.title = "Orcvs console";
  frame.className = "orcvs-small";
  document.body.append(frame);
  return new Promise((resolve) => {
    const check = () => {
      if (frame?.contentDocument?.getElementById("the_canvas_id")?.hasAttribute("width")) resolve();
      else setTimeout(check, 100);
    };
    check();
  });
}

for (const grid of document.querySelectorAll<HTMLElement>(".orcvs-grid[data-source]")) {
  grid.addEventListener("click", async () => {
    started ??= start();
    await started;
    frame!.contentWindow!.postMessage({ type: "orcvs.open", source: grid.dataset.source }, location.origin);
    frame!.focus();
  });
}
