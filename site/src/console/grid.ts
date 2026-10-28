// A Source drawn as the console draws it: one Cell per character on the Grid, in the console's
// font and Theme colours (`grid.css`). A Function's spelling and a Bang take their roles' colours;
// every other Cell is ordinary, since only the language knows what an operand is.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const FUNCTION_REFERENCE = fileURLToPath(new URL("../../../docs/functions.md", import.meta.url));

/** Every Function spelling, read from the reference's entry headings, such as "### `.+` Add". */
const SPELLINGS = new Set(
  [...readFileSync(FUNCTION_REFERENCE, "utf8").matchAll(/^### `(..)`/gm)].map((match) => match[1]),
);
// A heading format the pattern no longer matches would draw every Function as an ordinary Cell.
if (SPELLINGS.size === 0) throw new Error(`no Function spelling found in ${FUNCTION_REFERENCE}`);

const escape = (text: string) => text.replace(/[&<>"']/g, (character) => `&#${character.charCodeAt(0)};`);

function cell(character: string, role: string): string {
  const shown = character === " " ? "" : escape(character);
  return `<span${role ? ` class="${role}"` : ""}>${shown}</span>`;
}

/** The Functions whose Payload, the Cells up to the next `!` on the row, is text. */
const PAYLOAD_SPELLINGS = new Set(["!u", "!o"]);

function row(text: string, columns: number): string {
  let cells = "";
  for (let x = 0; x < columns; ) {
    const pair = text.slice(x, x + 2);
    const role = pair === "**" ? "bang" : SPELLINGS.has(pair) ? "function" : "";
    const width = role ? 2 : 1;
    for (let i = 0; i < width; i++) cells += cell(text[x + i] ?? " ", role);
    x += width;
    if (PAYLOAD_SPELLINGS.has(pair)) {
      // The Payload is ordinary Cells, and the `!` that closes it is drawn with its Function.
      const closer = text.indexOf("!", x);
      const end = closer === -1 ? columns : closer;
      for (; x < end; x++) cells += cell(text[x] ?? " ", "");
      if (closer !== -1) cells += cell("!", "function");
      x = end + (closer === -1 ? 0 : 1);
    }
  }
  return cells;
}

/**
 * `source` as a Grid of Cells. A `sendable` Grid is a button that opens its Source in the page's
 * small console (`small.ts`).
 */
export function gridHtml(source: string, sendable: boolean): string {
  const rows = source.replace(/\n$/, "").split("\n");
  const columns = Math.max(...rows.map((text) => text.length), 1);
  const cells = rows.map((text) => row(text, columns)).join("");
  const style = `--columns:${columns}`;
  return sendable
    ? `<button type="button" class="orcvs-grid not-content" style="${style}" data-source="${escape(`${rows.join("\n")}\n`)}" title="Open in the console">${cells}</button>`
    : `<div class="orcvs-grid not-content" style="${style}">${cells}</div>`;
}
