// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { satteri } from "@astrojs/markdown-satteri";
import { functionReference } from "./src/console/function-reference";

// The console is served from `public/play/`, which `mise run site_build` fills from Trunk with the
// same base: `SITE_BASE`, empty because the site is served from the root of its domain. Pages
// link to the console and to each other from the root.
export default defineConfig({
  base: process.env.SITE_BASE || "/",
  integrations: [
    starlight({
      title: "Orcvs",
      description: "A grid-based environment for composing and performing compact musical expressions.",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/orcvs/orcvs" }],
      // Source has no grammar to highlight; show it as plain text without a warning per block.
      expressiveCode: { shiki: { langAlias: { orcvs: "txt" } } },
      customCss: ["./src/styles/orcvs.css", "./src/console/console.css"],
      sidebar: [
        { label: "Learn", items: [{ autogenerate: { directory: "learn" } }] },
        { label: "Reference", items: [{ label: "Functions", link: "/reference/" }] },
        { label: "Open the console", link: "/play/" },
        { label: "About", link: "/about/" },
      ],
    }),
  ],
  markdown: { processor: satteri({ mdastPlugins: [functionReference] }) },
  vite: {
    // The console's font is read from `console/assets/`, outside this directory.
    server: { fs: { allow: [".."] } },
  },
});
