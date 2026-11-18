// Renders the repository's own docs as the site's pages: `docs/first-result.md` as `/learn/` and
// `docs/functions.md` as `/reference/`. The files stay the one source, so the site carries no
// prose of its own: `orcvs/tests/function_reference_doc.rs` runs the reference's examples against
// the language, and the site shows what that test checks. The mdast plugin touches those files
// alone.

import { fileURLToPath } from "node:url";
import { defineMdastPlugin, type PluginFactoryContext } from "satteri";
import { gridHtml } from "./grid";

const DOCS = fileURLToPath(new URL("../../../docs/", import.meta.url));

type HoldsSource = (lang: string | null | undefined) => boolean;

/** Each rendered doc: its route on the site, and the code blocks in it that hold a Source. */
const RENDERED_DOCS: Record<string, { route: string; holdsSource: HoldsSource }> = {
  "first-result.md": { route: "/learn/", holdsSource: (lang) => lang === "text" },
  "functions.md": { route: "/reference/", holdsSource: (lang) => lang === "orcvs" },
};

/** The rendered doc a repository file name names, if it names one. */
function renderedDoc(name: string) {
  return Object.hasOwn(RENDERED_DOCS, name) ? RENDERED_DOCS[name] : undefined;
}

const GITHUB_DOCS = "https://github.com/orcvs/orcvs/blob/main/docs/";

/** Where a link in a rendered doc goes on the site. */
function siteLink(url: string): string {
  if (/^[a-z]+:|^#|^\//.test(url)) return url;
  const hash = url.indexOf("#");
  const file = hash === -1 ? url : url.slice(0, hash);
  const fragment = hash === -1 ? "" : url.slice(hash);
  const doc = renderedDoc(file);
  return doc ? doc.route + fragment : new URL(url, GITHUB_DOCS).href;
}

function plugin(holdsSource: HoldsSource) {
  return defineMdastPlugin({
    name: "orcvs-docs",
    heading(node, context) {
      if (node.depth === 1) context.removeNode(node);
    },
    link(node, context) {
      context.setProperty(node, "url", siteLink(node.url));
    },
    // A Source is drawn as a Grid. A starting Source, a block with nothing after its language,
    // opens in the page's Small Console when pressed; any meta, such as the reference's `tick 0`
    // or the lesson's `result`, marks a Grid that is drawn but does not open.
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
  const doc = name && renderedDoc(name);
  return doc ? plugin(doc.holdsSource) : null;
}
