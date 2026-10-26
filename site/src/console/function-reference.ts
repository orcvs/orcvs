// Renders the repository's `docs/functions.md` as the site's `/reference/`. The file stays the one
// source: `orcvs/tests/function_reference_doc.rs` runs its examples against the language, so the
// site shows what that test checks. The mdast plugin below touches that file alone.

import { fileURLToPath } from "node:url";
import { defineMdastPlugin, type PluginFactoryContext } from "satteri";
import { consoleFigure } from "./embed";

const FUNCTION_REFERENCE = fileURLToPath(new URL("../../../docs/functions.md", import.meta.url));

const GITHUB_DOCS = "https://github.com/orcvs/orcvs/blob/main/docs/";

/** Where a link in `docs/functions.md` goes on the site. */
function siteLink(url: string): string {
  if (/^[a-z]+:|^#|^\//.test(url)) return url;
  if (url === "first-result.md") return "/learn/first-result/";
  return new URL(url, GITHUB_DOCS).href;
}

const plugin = defineMdastPlugin({
  name: "orcvs-function-reference",
  heading(node, context) {
    if (node.depth === 1) context.removeNode(node);
  },
  link(node, context) {
    context.setProperty(node, "url", siteLink(node.url));
  },
  code(node, context) {
    if (node.lang !== "orcvs" || node.meta) return;
    const html = consoleFigure({ source: `${node.value}\n`, label: "This example", preview: false });
    context.insertAfter(node, { raw: html, mdxExpressions: false });
  },
});

/**
 * For `docs/functions.md` alone: drops its `#` heading, which Starlight renders from the page
 * title; points its links at the site or the repository; and adds "Launch example" after each
 * example's starting Source, the `orcvs` block with no `tick` after it.
 */
export function functionReference(context: PluginFactoryContext) {
  return context.fileURL && fileURLToPath(context.fileURL) === FUNCTION_REFERENCE ? plugin : null;
}
