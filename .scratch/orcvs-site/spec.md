# An Orcvs site that teaches by running the console

Status: needs-triage

## Problem Statement

Orcvs has no website. Someone who hears about it has only the repository, and the only way to try
it is to build the console. The docs that exist, `README.md`, `docs/first-result.md` and
`docs/functions.md`, are Markdown in the repo. They describe a console the reader cannot see
running.

Learn Orca (https://metasyn.srht.site/learn-orca/) shows what works for Orca: short prose beside
the real application, where each example loads into the running app with one click. Orcvs already
builds for the browser. Nothing publishes that build, and the console cannot be told which Source
to open.

The site will be built and kept up by agents, so its structure has to be conventional, its build
has to fail loudly on mistakes, and its pages have to be checkable in a real browser.

## Solution

A static site, built with Astro and its Starlight docs theme, lives in `site/` in this repository.
It has an introduction, lessons, a reference, and the full console at `/play/`. Lessons embed the
real console, the Trunk build of `console`, in a same-origin iframe. A reader clicks "Launch
example" and the console opens with that example loaded, stopped, ready for the reader to start
Playback. The first lesson is `docs/first-result.md`, which needs no MIDI device.

The console gains one input for this: it reads the URL once at startup and can open a shipped
example by name or a short Source carried in the fragment. When embedded, it skips the service
worker and never touches the reader's saved Source.

CI builds the console and the site together into one static output and deploys it. Nothing built is
committed.

The research behind this spec is `docs/research/orcvs-site.md`. A second guide Toby supplied on
2026-10-25 contributed the routes, the launch-on-click rule, the saved-work rule, the CI path
filter, the agent workflow and the acceptance criteria.

## User Stories

1. As a newcomer, I want a site that says what Orcvs is and shows it running, so that I can decide
   whether to try it without building anything.
2. As a newcomer, I want the first lesson to be "Your first result", so that I get something to
   happen in a few minutes without a MIDI device.
3. As a learner, I want each example in a lesson to open in the real console with one click, so
   that I learn by running Orcvs, not by reading about it.
4. As a learner, I want an example to open stopped, so that nothing plays until I start Playback.
5. As a learner, I want to edit an example while it runs, so that I see how a change alters the
   result, as the first lesson asks.
6. As a learner, I want lessons to load quickly, so the page doesn't start a console for every
   example before I ask for one.
7. As a learner, I want "Open full console" beside every embed, so that I can carry on at full
   size.
8. As a performer, I want `/play/` to be the whole console with no site chrome, so that I can use
   it like the native app.
9. As a performer, I want my saved Source in `/play/` to survive any lesson I open, so that trying
   an example never loses my work.
10. As a performer, I want a link to an example to open stopped and never send MIDI by itself, so
    that a link someone sends me cannot play notes on my devices.
11. As a reader, I want keys to scroll the lesson until I click the console, and to reach the
    console after I click, so that the page and the console never fight over the keyboard.
12. As a reader on a phone, I want the lesson to stay readable and offer the full console rather than
    a cramped canvas, so that the site works on small screens.
13. As a reader whose browser fails to start the console, I want the lesson text to stay and a clear
    message where the console would be, so that a WASM failure never blanks the page.
14. As a reader, I want search across lessons and reference, so that I can find a Function by name.
15. As a reader, I want the site to say that the language is pre-release and that spellings may
    change, so that I am not surprised when they do.
16. As a reader, I want the site to look like the console, with its Grid, font and Theme colours,
    so that what I read matches what I run.
17. As a maintainer, I want the Function reference to come from the repo's checked docs, not a
    copy, so that the site cannot drift from the language.
18. As a maintainer, I want every deploy to build the console and site from pinned tools in CI, so
    that the published console always matches `main`.
19. As a maintainer, I want a deploy to replace the old console for returning readers, so that no
    one is stuck on a cached build.
20. As a maintainer, I want site-only changes to run site checks without the full Rust compile, and
    console changes to rebuild the embed, so that each change pays only for what it touches.
21. As a maintainer, I want browser smoke tests of the composed site in CI, so that a broken embed
    fails a pull request.
22. As an agent maintaining the site, I want a repository skill that names the site's commands,
    structure, embed contract and checks, so that I change the site the way the repo expects.
23. As an agent maintaining the site, I want to look at the built site in a browser and take
    screenshots, so that I check what I built rather than only its source.

## Implementation Decisions

- **Framework.** Astro with Starlight, versions pinned in `site/package.json` and its lockfile.
  Content is Markdown or MDX under `site/src/content/docs/`. No React, no server, no CMS. Astro
  7.4.2 requires Node 22.23.4 or newer, so the Node pin in `mise.toml` moves from 22.23.1 to
  22.23.4, and the roadmap gate runs on it too.
- **Routes.** `/` introduction; `/learn/` lessons; `/reference/` the Function reference and
  language notes; `/play/` the full console; `/download/` native builds once releases exist;
  `/about/` project status and links.
- **The console in the site.** The unmodified Trunk build is served from `/play/`. Lessons embed it
  in a same-origin iframe through one MDX component, `<Console example="…" />`, that renders a
  still preview with a "Launch example" control and an "Open full console" link. The iframe is
  created only on launch, so a page with many examples starts at most the ones the reader opens.
  The component shows a message in place of the console if it fails to load.
- **Loading a Source.** The console reads the URL once at startup. `?example=<name>` opens a
  shipped example from `console/src/examples.rs`. `#source=<percent-encoded text>` opens a short
  Source for prose snippets. The text is read as a Source File through the same reader as opening a
  file, and rejected whole if it does not parse. A byte limit is checked before decoding; the
  ticket sets it. An unknown example or a rejected Source opens the empty Source with a message.
  There is no `postMessage` bridge. One is added only if measured reload time makes launching
  examples feel slow.
- **Nothing plays by itself.** A Source opened from the URL opens with Playback stopped. Only a
  reader's action starts it. This matters because once Web MIDI access is granted, the console
  selects the first MIDI destination by itself.
- **Saved work.** `?example=` or `#source=` puts the console in embed mode. In embed mode it
  neither restores nor stores the Source in eframe storage, so the Source the reader keeps in
  `/play/` is never read over or written over.
- **Service worker.** In embed mode `index.html` does not register `sw.js`. `/play/` keeps it, with
  its scope limited to `/play/`, and a deploy must replace the cached console: the ticket picks
  between file hashing (`console/Trunk.toml` turns it off today) and a versioned worker, and proves
  that an old build is not served after a deploy.
- **Keyboard.** No console change is planned. The first embed ticket checks in a real browser that
  before a click, keys scroll the page and do not move the Cursor; after one click, arrow keys move
  the Cursor and the page does not scroll; and Tab or a click outside gives keys back to the page.
  A failure becomes its own ticket, with that condition as its test.
- **Build.** One CI job builds in this order, with no built files committed:
  1. Install the tools pinned in `mise.toml`.
  2. In `console/`, `trunk build --release --locked --public-url <base>/play/ --dist ../site/public/play`.
  3. In `site/`, the Astro build with `base` set to `<base>`. It copies `public/play/` unchanged.
  4. A check that fails if `public/play/index.html` does not name `<base>/play/` as its base.
  5. Serve the output locally and run the smoke tests against it.
  6. Upload and deploy.

  `<base>` is one workflow variable, so the two base paths cannot disagree. The existing
  feature-on and feature-off WASM builds in `check_wasm` stay as they are.
- **CI paths.** Today a pull request whose diff is only Markdown skips compiling (`test.yml`,
  `changes` job). Site files get their own path filter and a site job that runs the Astro build and
  smoke tests without the Rust gates, unless `console/` or the toolchain also changed. The new
  workflow is covered by the tooling-contract gates: `check-tooling-contract.sh`, `actionlint` and
  `zizmor`.
- **Reference content.** `docs/functions.md` stays the source of the Function reference, already
  checked by `orcvs/tests/function_reference_doc.rs`. The site renders it from the repo at build time
  and does not keep a copy. Lessons cite `CONTEXT.md` vocabulary.
- **Look.** The site uses the console's font and the built-in Themes' colours, real screenshots and
  real embeds, and states the language's pre-release status on the home page and in the reference.
  No generic landing-page gradients, feature cards or decorative animation.
- **Agent tooling.** A `website` skill in `.agents/skills/website/SKILL.md`, symlinked from
  `.claude/skills/` per `docs/agents/skills.md`, records the commands, layout, embed contract and
  which checks a site change owes. Agents check pages in the pre-installed Playwright Chromium.
  Outside skills, such as a design skill, are reviewed and pinned before use, not pulled from an
  upstream branch.
- **Hosting.** Not decided. GitHub Pages needs no second provider and is enough, because the
  console needs no special headers. Cloudflare (Workers static assets, which Cloudflare now steers
  new projects to over Pages) adds preview deploys per pull request and a `_headers` file. Toby
  decides before the deploy ticket.
- **Sound.** Not in the first milestone. A built-in synth MIDI destination through `MidiBackend`,
  like Learn Orca's Enfer bridge, is a later ticket with its own ADR. It can sound only after a
  reader's gesture, which the stopped-on-load rule already supplies.

## Testing Decisions

- A good test checks what a reader sees and does: which Source opens, whether Playback is running,
  whether the saved Source survived. It never inspects private state.
- **Console, URL input.** Tests at the boundary for the URL reader: a known and an unknown example
  name, a valid Source, malformed percent-encoding, text over the limit, text that is not a valid
  Source File, and both parameters at once. The parse rule gets a property test that any text it
  accepts round-trips through the Source File writer. Prior art: the Source File tests in `orcvs`
  and the console's startup tests.
- **Console, embed mode.** Tests that a Source opened from the URL opens stopped; that embed mode
  neither restores nor stores the Source, with `persistence` on and off; and that the WASM build
  still compiles both ways (`check_wasm`).
- **Site.** Playwright smoke tests against the composed output served locally: `/`, `/learn/`,
  `/reference/` and `/play/` load with no console errors; "Launch example" in the first lesson opens
  the console with that example; a failed console load leaves the lesson text and shows the
  message; and the keyboard conditions above. The egui canvas exposes no DOM to assert on, so the
  console's own behaviour stays tested by the Rust and browser suites. Site tests use screenshots
  and real input only for what the page shows.
- **Deploy.** A test that a second deploy replaces the cached `/play/` console.

## Out of Scope

- A built-in synth, which is a later ticket with an ADR (Implementation Decisions, Sound).
- A `postMessage` bridge, unless reload time is measured and found too slow.
- Native downloads, until there are release builds to link.
- Rewriting the console or any of the language in JavaScript.

## Further Notes

- Milestone 1 is "Your first result" live on the site with the real console. It is done when a
  clean checkout builds one static output; `/`, `/learn/` and `/play/` load from it; the first lesson
  matches `docs/first-result.md`; a reader can launch its example, start Playback and edit it with no
  MIDI device; `/play/` works on its own; the saved Source survives the lesson; a deploy replaces the
  cached console; and the site checks run in CI beside the existing Rust and WASM gates.
- Tickets are written from this spec once Toby has triaged it.
