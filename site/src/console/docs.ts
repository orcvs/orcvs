// Renders the repository's own docs as the site's pages: `docs/first-result.md` as `/learn/` and
// `docs/functions.md` as `/reference/`. The files stay the one source, so the site carries no
// prose of its own: `orcvs/tests/function_reference_doc.rs` runs the reference's examples against
// the language, and the site shows what that test checks. The mdast plugin touches those files
// alone.

import { fileURLToPath } from "node:url";
import { defineMdastPlugin, type PluginFactoryContext } from "satteri";
import { gridHtml } from "./grid";

const DOCS = fileURLToPath(new URL("../../../docs/", import.meta.url));

/** Each rendered doc, and the code blocks in it that hold a Source. */
const SOURCE_BLOCKS: Record<string, (lang: string | null | undefined) => boolean> = {
  "first-result.md": (lang) => lang === "text",
  "functions.md": (lang) => lang === "orcvs",
};

const GITHUB_DOCS = "https://github.com/orcvs/orcvs/blob/main/docs/";

/** Where a link in a rendered doc goes on the site. */
function siteLink(url: string): string {
  if (/^[a-z]+:|^#|^\//.test(url)) return url;
  if (url === "first-result.md") return "/learn/";
  if (url === "functions.md") return "/reference/";
  return new URL(url, GITHUB_DOCS).href;
}

function plugin(holdsSource: (lang: string | null | undefined) => boolean) {
  return defineMdastPlugin({
    name: "orcvs-docs",
    heading(node, context) {
      if (node.depth === 1) context.removeNode(node);
    },
    link(node, context) {
      context.setProperty(node, "url", siteLink(node.url));
    },
    // A Source is drawn as a Grid. A starting Source, a block with no `tick` after its language,
    // opens in the page's small console when pressed.
    code(node, context) {
      if (!holdsSource(node.lang)) return;
      context.replaceNode(node, { type: "html", value: gridHtml(node.value, !node.meta) });
    },
  });
}

/**
 * For the rendered docs alone: drops each `#` heading, which Starlight renders from the page
 * title; points links at the site or the repository; and draws each Source as a Grid.
 */
export function repositoryDocs(context: PluginFactoryContext) {
  const path = context.fileURL && fileURLToPath(context.fileURL);
  const name = path?.startsWith(DOCS) ? path.slice(DOCS.length) : undefined;
  return name && name in SOURCE_BLOCKS ? plugin(SOURCE_BLOCKS[name]) : null;
}
