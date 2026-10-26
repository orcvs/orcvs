// The markup of an embedded console, shared by the `<Console>` component and the Function
// reference, which adds one after each example. `launch.ts` turns it into a running console when
// the reader presses "Launch example"; `console.css` styles it.

export interface Embed {
  /** A shipped example's link name, such as `cell-tracker`. */
  example?: string;
  /** Source File text to open, `""` for an empty Source. */
  source?: string;
  /** What the caption says the console will open. */
  label?: string;
  /** Show `source` above the buttons; off where the page already shows it. */
  preview?: boolean;
}

/** The `/play/` URL that opens `embed`'s Source (`console/src/linked_source.rs`). */
export function consoleLink({ example, source }: Embed): string {
  if ((example === undefined) === (source === undefined)) {
    throw new Error("an embedded console takes exactly one of `example` or `source`");
  }
  return example !== undefined
    ? `/play/?example=${encodeURIComponent(example)}`
    : `/play/#source=${encodeURIComponent(source ?? "")}`;
}

const escape = (text: string) =>
  text.replace(/[&<>"']/g, (character) => `&#${character.charCodeAt(0)};`);

/** The still preview, its buttons, the hidden failure message, and the caption. */
export function consoleFigure(embed: Embed): string {
  const link = escape(consoleLink(embed));
  const { example, source, preview = true } = embed;
  const label = embed.label ?? (source === "" ? "An empty Source" : (example ?? "This Source"));
  const shown = preview && source ? `<pre>${escape(source)}</pre>` : "";
  // No blank lines: Markdown ends an HTML block at the first one.
  return `<figure class="orcvs-console not-content" data-link="${link}">
  <div class="orcvs-console-preview">${shown}
    <div class="orcvs-console-actions">
      <button type="button" class="orcvs-console-launch">Launch example</button>
      <a href="${link}" target="_blank" rel="noopener">Open full console</a>
    </div>
  </div>
  <p class="orcvs-console-failure" role="alert" hidden>
    The console could not start here. <a href="${link}" target="_blank" rel="noopener">Open it in its own tab</a>.
  </p>
  <figcaption>${escape(label)}</figcaption>
</figure>`;
}
