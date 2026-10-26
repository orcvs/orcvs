// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// The console is served from `public/play/`, which `mise run site_build` fills from Trunk with the
// same base path (`/play/`), so the site is built for the root of its domain.
export default defineConfig({
  integrations: [
    starlight({
      title: "Orcvs",
      description: "A grid-based environment for composing and performing compact musical expressions.",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/orcvs/orcvs" }],
      customCss: ["./src/styles/orcvs.css"],
      sidebar: [
        { label: "Learn", items: [{ autogenerate: { directory: "learn" } }] },
        { label: "Reference", items: [{ autogenerate: { directory: "reference" } }] },
        { label: "Open the console", link: "/play/" },
      ],
    }),
  ],
  vite: {
    // The console's font is read from `console/assets/`, outside this directory.
    server: { fs: { allow: [".."] } },
  },
});
