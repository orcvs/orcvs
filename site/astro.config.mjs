// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { satteri } from "@astrojs/markdown-satteri";
import { repositoryDocs } from "./src/console/docs";

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
      customCss: ["./src/styles/orcvs.css", "./src/console/grid.css"],
      sidebar: [
        { label: "Console", link: "/" },
        { label: "Learn", link: "/learn/" },
        { label: "Reference", link: "/reference/" },
      ],
    }),
  ],
  markdown: { processor: satteri({ mdastPlugins: [repositoryDocs] }) },
  vite: {
    // The console's font is read from `console/assets/`, outside this directory.
    server: { fs: { allow: [".."] } },
  },
});
