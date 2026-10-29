---
name: website
description: The Orcvs site in site/. Use for its pages, lessons, the embedded console, the Function reference page, its build, tests or Cloudflare deploy. The egui skill covers the console's own presentation.
---

# website

The site is Astro + Starlight in `site/`, with the release WASM console served beside it at
`/play/`. `.scratch/orcvs-site/spec.md` holds the decisions and `docs/tooling.md` the workflow;
read both before changing routes, hosting or the build. A change to what the console draws or
how it handles input belongs to the `egui` skill; this skill covers how the site carries it.

## Steps

1. Read `site/package.json`, `site/astro.config.mjs`, the page or component the change reaches,
   and its test in `site/tests/`.
2. Make the change. Keep the repository's docs the one source: the lesson follows
   `docs/first-result.md` and `/reference/` renders `docs/functions.md`, so a fact about the
   language is edited there, never copied into `site/`.
3. Build and test with the commands below, then open each changed page in the pre-installed
   Playwright Chromium and take a screenshot, in the light and dark themes and at 390 px wide.
   Done when every site test passes and each screenshot shows the page as intended.
4. Owe the gates below, then report as `CLAUDE.md`'s completion evidence asks.

## Commands

Node must be the `mise.toml` pin: Astro 7 refuses older Node.

```sh
mise run site_build          # the published console into site/public/play/, then Astro into site/dist/
mise run site_test           # Playwright against site/dist/
```

To try a console change in the site, `mise run console_web_build` and then
`CONSOLE_WEB=../target/console-web mise run site_build`; the path is relative to `site/`, where the
build runs. Without mise, `npm ci && npm run build` in `site/` builds the site. A console already in
`site/public/play/` is kept; delete that directory to fetch the published one again.

## Contracts the code cannot show

- **The site is the console.** `/` is `/play/` filling the window, with its own splash. Every
  page wears the Grayscale Themes and Monaspace Neon (`site/src/styles/orcvs.css`), and carries
  the repository's docs, never prose written for the site (`.scratch/site-as-console/spec.md`).
- **Examples.** `site/src/console/docs.ts` draws each Source in `docs/first-result.md` and
  `docs/functions.md` as a Grid (`grid.ts`). Pressing a starting Source starts the page's one
  small console, `/play/?small`, and posts it `{ type: "orcvs.open", source }` (`small.ts`, ADR
  0086). Nothing opens a new tab or window.
- **Links.** `/play/?example=<link-name>` opens a shipped example (`808`, `cell-tracker`,
  `orca-tracker`, `orca-sequencer`, `orca-knobs`); `/play/#source=<percent-encoded Source File>`
  opens text, up to 16 KiB of fragment; `/play/?small` opens the small console. Each opens
  stopped, none reads or writes saved work, and none registers the service worker
  (`console/src/linked_source.rs`, `console/index.html`). `/` frames `/play/` with its own query
  and fragment, so a link to share is `/?example=<link-name>`, which also plays through the page's
  Built-in Synth (ADR 0094).
- **Started.** The canvas has no DOM. eframe gives `#the_canvas_id` a `width` attribute when it
  starts, which is what a test waits on; the small console also posts `{ type: "orcvs.ready" }` to
  its page once it listens for Sources, which is what `small.ts` waits on. What the console shows is compared as frame
  screenshots, and keys reach it as real input.
- **Markdown.** Astro 7 renders Markdown with Sätteri, not remark: a transform is a Sätteri mdast
  plugin in `markdown.processor`, as `site/src/console/docs.ts` is.
- **Serving.** `astro preview` moves itself to the background when it detects an agent; pass
  `--ignore-lock` to keep it in the foreground, and stop it with `npx astro preview stop`.
- **Deploy.** `site/tests/deploy.spec.ts` rewrites files in `site/dist/play/`, so it runs in its
  own Playwright project after the rest.
- **Build and deploy.** Cloudflare's GitHub app builds and deploys the site by running
  `npm run build` in `site/`, with the Node version in `site/.node-version`, after installing the
  dependencies. So a build step belongs in that script or its `prebuild` step.
  `.github/workflows/site.yml` runs `mise run site_build`, which is `npm ci && npm run build` in
  `site/`, only to test it.
- **Console build.** The site never compiles the console. `site/scripts/console.sh`, npm's
  `prebuild` step, downloads the build
  `.github/workflows/console-web.yml` publishes from `main` on the `console-web` release, whose
  URLs are relative, so the site needs no base path for it.

## Gates a site change owes

- Every site change: `mise run site_test`, after a build that includes the change.
- `console/`: the crate gates in `CLAUDE.md` and `mise run check_wasm`, then `mise run
  console_web_build` and `mise run site_build`.
- `.github/workflows/site.yml`, `.github/workflows/console-web.yml` or `mise.toml`: the tooling
  gates in `CLAUDE.md`.
- `.scratch/`: the roadmap gates in `CLAUDE.md`.

An outside skill, such as a design skill, is read and pinned to a commit before it is used,
never followed from an upstream branch.
