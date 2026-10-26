// Starts an embedded console when its reader presses "Launch example": a same-origin iframe of
// `/play/` at the figure's link replaces the preview, so a page starts only the consoles its
// reader opens. The frame does not take focus; the console takes the keyboard itself as it starts.

// How long a launched console has to start before the page says it could not.
const START_TIMEOUT_MS = 20_000;

// eframe sets the canvas's `width` attribute when it starts; a page whose WASM never ran keeps
// the attribute-less 300 × 150 default.
function started(frame: HTMLIFrameElement): boolean {
  return !!frame.contentDocument?.getElementById("the_canvas_id")?.hasAttribute("width");
}

for (const figure of document.querySelectorAll<HTMLElement>(".orcvs-console")) {
  figure.querySelector(".orcvs-console-launch")?.addEventListener("click", () => {
    const frame = document.createElement("iframe");
    frame.src = figure.dataset.link!;
    frame.title = "Orcvs console";
    figure.querySelector(".orcvs-console-preview")!.replaceWith(frame);
    const failure = figure.querySelector<HTMLElement>(".orcvs-console-failure")!;
    const deadline = Date.now() + START_TIMEOUT_MS;
    const check = () => {
      if (started(frame)) return;
      if (Date.now() > deadline) {
        frame.remove();
        failure.hidden = false;
      } else {
        setTimeout(check, 250);
      }
    };
    check();
  });
}
