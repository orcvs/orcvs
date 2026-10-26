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
mise run site_build   # release console into site/public/play/, then the Astro build into site/dist/
mise run site_test    # Playwright against site/dist/
```

Without mise, `bash site/build.sh` from the repository root builds. After a change to `site/` alone,
`npm run build` in `site/` is enough; rebuild the console only when `console/` changed.

## Contracts the code cannot show

- **Embedding.** `<Console example="…" />` or `<Console source="…" />` (`site/src/components/`)
  renders the markup in `site/src/console/embed.ts`; `launch.ts` swaps the preview for a
  same-origin iframe only when the reader presses Launch, so a page starts only the consoles its
  reader opens. Pressing Launch is the reader's click: the console takes the keyboard as it
  starts, and a click outside gives keys back to the page.
- **Links.** `/play/?example=<link-name>` opens a shipped example (`cell-tracker`,
  `orca-tracker`); `/play/#source=<percent-encoded Source File>` opens text, up to 16 KiB of
  fragment. Either opens stopped, neither reads nor writes saved work, and neither registers the
  service worker (`console/src/linked_source.rs`, `console/index.html`).
- **Started.** The canvas has no DOM. eframe gives `#the_canvas_id` a `width` attribute when it
  starts; that attribute is the only start signal. What the console shows is compared as frame
  screenshots, and keys reach it as real input.
- **Markdown.** Astro 7 renders Markdown with Sätteri, not remark: a transform is a Sätteri mdast
  plugin in `markdown.processor`, as `site/src/console/function-reference.ts` is.
- **Serving.** `astro preview` moves itself to the background when it detects an agent; pass
  `--ignore-lock` to keep it in the foreground, and stop it with `npx astro preview stop`.
- **Deploy.** `site/tests/deploy.spec.ts` rewrites files in `site/dist/play/`, so it runs in its
  own Playwright project after the rest. `.github/workflows/site.yml` builds once into the
  `site-<sha>` artifact, and its test, deploy and preview jobs all use that artifact, so a build
  step belongs in `site/build.sh` and nowhere later.

## Gates a site change owes

- Every site change: `mise run site_test`, after a build that includes the change.
- `console/`: the crate gates in `CLAUDE.md` and `mise run check_wasm`, then `mise run site_build`.
- `.github/workflows/site.yml` or `mise.toml`: the tooling gates in `CLAUDE.md`.
- `.scratch/`: the roadmap gates in `CLAUDE.md`.

An outside skill, such as a design skill, is read and pinned to a commit before it is used,
never followed from an upstream branch.
