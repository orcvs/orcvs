# A site for Orcvs that agents build and maintain: research

Research date: 2026-10-25, against `5182d65` on `main`.
Each external claim cites a primary source from the Sources list at the end, as `[n]`. A claim
marked **UNVERIFIED** could not be confirmed from a primary source. A claim marked
**inference** follows from the cited sources but none of them states it.

## Recommendation

Build the site with **Astro and its Starlight docs theme**, host it on **GitHub Pages**, and embed
the console the way Learn Orca embeds Orca: as **a same-origin iframe of the unmodified Trunk
build**, served from the site's own `public/play/` directory.

- **Agents work well with it.** Pages are plain Markdown or MDX files. Starlight checks the
  frontmatter against a schema, and the TypeScript config means an agent gets a build error
  instead of a silent mistake [10][11]. Astro publishes its docs in agent-readable form at
  `docs.astro.build/llms.txt` [12]. Starlight brings sidebar, Pagefind search and code highlighting
  with no configuration [16][17][7], so an agent writes content instead of building a site.
- **The embed needs no bundler work.** Astro copies `public/` into the build untouched [14], so
  Vite never touches the wasm-bindgen output. One MDX component, for example
  `<Console example="cell-tracker" />`, can wrap the iframe and the "load this example" links, so
  every page embeds the console the same way.
- **The toolchain cost is small, with one Node bump.** The repo already pins Node in `mise.toml`
  and has a `package.json` for the roadmap script, so Starlight adds npm packages, not a new
  runtime. Astro 7.4.2 declares `engines.node >=22.23.4` [5]. The repo pins Node 22.23.1, so
  adopting Astro moves the `mise.toml` pin to 22.23.4 (released 2026-10-20 [101]), a patch release
  on the same line, which the roadmap gate then runs on too. Starlight 0.42.7 declares no `engines`
  of its own [7].
- **GitHub Pages is enough.** The console is single-threaded, so it needs no cross-origin isolation
  headers, which GitHub Pages cannot set [77][86] (inference for eframe from [20][81]). Web MIDI in
  a same-origin iframe needs no `allow` attribute [93][94].

The runner-up is **mdBook**. It is a Rust binary with a small TOML config, and it has built-in
search [24]. It loses on two counts that matter here. It has no component system, so each embed
would be hand-written raw HTML, which agents copy and let drift. It also has no `llms.txt` [25].
Choose mdBook instead only if keeping Node out of the site build matters more than those two.

Learn Orca's sound comes from a second iframe, Enfer, that takes Orca's MIDI messages directly
(section 1). Orcvs can do the same through its existing `MidiBackend` seam, without the site's
help. That is a console feature with its own ADR, not a framework question; section 5 describes it.

## 1. How Learn Orca embeds Orca

Learn Orca's source repository on sourcehut refused automated fetches, and its old GitHub copy is
now an empty "moved" stub [4]. These findings come from the files deployed on
`metasyn.srht.site`.

- **The app runs in a same-origin iframe.** `js/main.js` reads the running Orca from
  `document.getElementById('orca-iframe').contentWindow`, and an Enfer synth from `enfer-iframe`
  the same way [1]. Reading another frame's globals only works when both are same-origin, so Orca
  is served from the Learn Orca origin (inference). The iframe `src` values are UNVERIFIED.
- **The page drives the app through its JavaScript API.** `loadOrca(text)` calls Orca's
  `orca.reset()`, `toggleGuide(false)` and `orca.writeBlock(2, 2, text)` [1]. There is no
  `postMessage` protocol.
- **Examples are one JavaScript object.** `js/examples.js` sets `window.examples` to a map from
  `category/name` to the patch text. The page builds its menus from it, and each link calls
  `loadExample(key)` [1][2]. Clicking any `pre.orca` code block in the prose loads that block too
  [1].
- **Sound comes from a patched MIDI output.** `patchOrcaToEnfer()` replaces Orca's MIDI output
  device with an object whose `send` forwards each message to `enferClient.io.onMessage` [1]. So
  the page plays sound with no MIDI device and no MIDI permission prompt. What Enfer is inside is
  UNVERIFIED: that it is a Tone.js sample synth by hundred rabbits, pinned at a particular commit,
  that maps note-on channels to drum kits and control changes to mixer knobs. `main.js` names no
  iframe `src` and no commit [1], and Enfer's repository could not be reached from here. Section 5
  depends only on the bridge, which `main.js` shows.
- **The site is hand-written.** No static site generator markers were visible [1][3]. It is served
  by sourcehut pages [3].

**What this means for Orcvs.** An eframe app has no JavaScript globals for a parent page to reach
into. It exposes only what wasm-bindgen exports, and eframe's documented web pattern is a
`#[wasm_bindgen]` `WebHandle` wrapping `WebRunner` [20]. To load an example, the console needs one
of: a URL query or hash read at startup, an exported wasm-bindgen function, or a `postMessage`
listener. Section 5 picks one.

## 2. How the WASM build works today

- `mise run check_wasm` runs clippy for `wasm32-unknown-unknown`, then `trunk build --locked` in
  `console/`, with and without default features (`mise.toml`, task `check_wasm`). Trunk is pinned
  to 0.21.14 (`mise.toml`). `mise run test_wasm` runs the browser regressions in headless Firefox
  with `wasm-pack`.
- `console/index.html` drives the build. `<link data-trunk rel="rust" data-wasm-opt="2" />` builds
  the console, `<base data-trunk-public-url />` writes the served base path, and `copy-file` links
  ship `sw.js`, `manifest.json` and the icons. `console/Trunk.toml` turns off file hashing.
- `console/src/main.rs` finds `<canvas id="the_canvas_id">` and starts `eframe::WebRunner` on it.
  The page's CSS makes the canvas fill the window.
- The page registers `sw.js`, which caches the app for offline use. Appending `#dev` to the URL
  skips it [100].
- Nothing publishes the build today. The only Pages use is the benchmark series on `gh-pages`
  (`.github/workflows/bench.yml`).

Two consequences for an embed:

- **The output is already a static directory.** `trunk build` writes `index.html`, the JS glue and
  the `.wasm` file. Any static site generator can copy it as-is (section 3). Building with
  `--public-url /<site-base>/play/` makes the `<base>` tag point at the right path (Trunk's
  `build.public_url` [75]).
- **The service worker would serve stale builds inside the site.** Once a reader has loaded the
  embed, the cached app keeps loading after a deploy. The embed should not register the worker; see
  section 5.

## 3. The candidates

Versions were current on 2026-10-25.

| | Latest | Runtime | Content | Copied as-is | Search | Components in Markdown | Official `llms.txt` |
|---|---|---|---|---|---|---|---|
| Astro + Starlight | astro 7.4.2, starlight 0.42.7 [5][6] | Node | MD, MDX, Markdoc; schema-checked frontmatter [10] | `public/` [14] | Pagefind, built in [17] | Yes, MDX [18] | Astro yes [12]; Starlight by community plugin [13] |
| mdBook | 0.5.4 [21] | Rust binary | MD plus `SUMMARY.md` | any non-MD file in `src/` [26] | Built in [24] | No | No [25] |
| Zola | 0.23.6 [32] | Rust binary | MD, TOML frontmatter | `static/` [36] | Index only, you write the UI [38] | Tera shortcodes [37] | No [35] |
| VitePress | 1.6.4 stable; 2.0 in alpha [41][42] | Node | MD compiled to Vue | `public/` [45] | MiniSearch [49] | Yes, Vue [48] | Yes [44] |
| Docusaurus | 3.10.2 [51] | Node | MDX | `static/` [55] | Algolia; local only by plugin [57] | Yes, React | No [54] |
| Eleventy | 3.1.7 [59] | Node | MD plus templates | passthrough copy [64] | None | Shortcodes [62] | No [63] |
| Hugo | 0.168.0 [65] | Go binary | MD; raw HTML off by default [71] | `static/` [69] | None | Shortcodes [70] | No [68] |
| Trunk alone | 0.22.0 [72] | Rust binary | HTML only | `copy-dir` [75] | None | No | n/a |

What decided the ranking:

- **Every candidate can host the embed.** Each one copies a directory into its output unchanged,
  so the Trunk build drops in without integration work. Embedding is not what separates them.
- **Agent-friendliness separates them.** The useful properties are: content in plain files, a
  build that fails loudly on a mistake, a docs theme that already exists, and documentation an
  agent can read. Starlight has all four. VitePress has all four but its next major version is in
  alpha [42], and Vue components are a second framework to learn. Docusaurus has the largest
  config surface and no local search [57].
- **The Rust-native generators cost more agent work.** mdBook has search and a docs layout but no
  components. Zola, Hugo and Eleventy have neither a docs theme nor search UI, so an agent would
  build and then maintain both.
- **Trunk alone is not a site generator.** Its guide describes a build driven by one target HTML
  file, with one Rust `main` link in it [75][102]. Neither the 0.21.14 guide nor the current one has
  a section on building several pages [103] (that it cannot is an inference from that absence).
  The deciding point holds either way: Trunk renders no Markdown and has no navigation or search
  [75], so every page would be hand-written HTML.
- **Training data is UNVERIFIED.** GitHub stars are the only proxy found: Hugo 89.7k, Docusaurus
  66.2k, Astro 61.7k, mdBook 22.1k, VitePress 18.3k, Zola 17.4k, Starlight 9.2k [67][52][8][23][43]
  [33][9].

Risks of the recommendation:

- Starlight is pre-1.0 [6]; expect occasional breaking upgrades. Pin it in `package.json` and the
  lockfile.
- Astro needs Node 22.23.4 or newer [5], three patch releases past the repo's pin.

## 4. Hosting

- **GitHub Pages works and costs nothing for a public repo.** It cannot set custom headers; the
  request for COOP and COEP is still open with "No ETA" [77]. The site does not need them.
- **No headers are needed.** Cross-origin isolation is needed only for `SharedArrayBuffer`, wasm
  threads and high-precision timers [86]. The console uses none. egui's own web demo runs on
  GitHub Pages without the headers [81] (inference).
- **`.wasm` is served as `application/wasm`.** GitHub's archived docs say Pages takes MIME types
  from mime-db [78], which has listed `.wasm` since 2017 [79]. Current github.com behaviour is
  UNVERIFIED in GitHub's docs, but egui's demo depends on it [81].
- **Deploy with GitHub's Pages actions.** Astro's guide uses `withastro/action@v6` and
  `actions/deploy-pages@v5` [19]. Current majors are `configure-pages` v6,
  `upload-pages-artifact` v5 and `deploy-pages` v5 [83][84][85]. Astro needs `site` and, for a
  project page, `base: '/orcvs'` [19].
- **CI builds the console and the site in one job, and nothing built is committed.** A committed
  WASM build would go stale against `main`. The deploy job runs on pushes to `main`:
  1. Install the `mise.toml` tools, so Trunk, Rust and Node are the pinned ones.
  2. In `console/`, run
     `trunk build --release --locked --public-url /orcvs/play/ --dist ../site/public/play`.
     `--release` matters: `check_wasm` builds the debug profile, which is not what readers should
     download. `--public-url` and `--dist` are Trunk's `public_url` and `dist` settings [75][104].
  3. In `site/`, run the Astro build with `site` set to the Pages URL and `base: '/orcvs'` [19]. It
     copies `public/play/` into its output unchanged [14].
  4. Upload the output with `upload-pages-artifact` and publish it with `deploy-pages` [83][84].

  The base path is written twice, and the two must agree: Trunk's public URL is Astro's `base`
  followed by `/play/`. If they differ, every asset URL in the embed breaks. Both values should come
  from one variable in the workflow, and a check after step 3 should fail the job if
  `public/play/index.html` does not name `/orcvs/play/` as its base. Adding the workflow also brings
  in the tooling-contract gates in `CLAUDE.md` (`check-tooling-contract.sh`, `actionlint`,
  `zizmor`).
- **Move to Cloudflare or Netlify only if headers become necessary.** Both read a `_headers` file
  [87][88][90]. Threads in the console would be that trigger.

## 5. What the console needs to be embedded

These are console changes. Each would become a ticket, and the sound target an ADR, once the
framework is chosen.

1. **Load a Source from the URL, stopped.** Learn Orca's parent page calls into Orca's globals [1].
   eframe has none [20]. The smallest Orcvs equivalent is reading the URL once at startup:
   `?example=<name>` names a shipped example (`console/src/examples.rs`), and a fragment such as
   `#source=<percent-encoded text>` carries a prose snippet. The site changes examples by changing
   the iframe's `src`. That reloads the WASM from the browser cache, which is simpler than a
   `postMessage` protocol. A wasm-bindgen export is the alternative if reload time proves too slow;
   measure it first.

   A URL is input anyone can craft, so the ticket has to fix the boundary:
   - **Nothing plays until the reader acts.** A Source from the URL loads with Playback stopped,
     and only the reader starts it. This matters because the console asks for Web MIDI access at
     startup (`console/src/web_midi.rs`) and, once access is granted, selects the first destination
     every frame until one is selected (`MidiDeviceSelection::observe_frame`, `console/src/midi.rs`).
     A link that started Playback by itself could send notes to a visitor's device.
   - **The text is parsed as a Source File, and rejected whole if it does not parse.** It goes
     through the same reader as opening a file, with a byte limit checked before decoding. The limit
     is the ticket's to set. Lessons should prefer `?example=` and short snippets: the fragment never
     reaches the server, but browsers cap URL length (the cap is UNVERIFIED), so a long Source
     belongs in `examples/`.
   - **The embed never overwrites saved work.** The embed and the full console share an origin, so
     they share eframe's storage, and the `persistence` feature restores the Source from it. A
     Source loaded from the URL must neither replace nor be saved over the reader's stored Source.
   - **Tests at the boundary.** `CLAUDE.md` asks for boundary or property tests at a parser
     boundary. The ticket needs them for malformed percent-encoding, text over the limit, text that
     is not a valid Source File, an unknown example name, and both parameters given at once.
2. **Skip the service worker in the embed.** The same parameter, or a separate `?embed`, stops
   `index.html` registering `sw.js` (section 2).
3. **Check keyboard focus; build nothing unless the check fails.** Since eframe 0.28 the canvas
   takes keyboard input only while focused [91][97], and an iframe keeps its events from the host
   page (inference). So this is probably no work. The first embed ticket carries it as acceptance
   conditions, checked in a real browser:
   - Before the reader clicks the embed, arrow keys and Space scroll the lesson page, and the
     console's Cursor does not move.
   - After one click in the embed, arrow keys move the Cursor and the lesson page does not scroll.
   - Tab or a click outside the embed returns keys to the page.

   Only if one of these fails is there console work, and the failing condition is its test.
4. **A built-in sound target.** Learn Orca plays sound because its bridge hands MIDI bytes
   straight to a synth [1]. Orcvs already has the seam for this: `MidiBackend` (`orcvs/src/midi.rs`)
   lists destinations and connects to one, and the web console's backend is
   `console/src/web_midi.rs`. A second web backend, or an extra destination in the existing one,
   could offer a destination such as "Built-in synth" whose connection turns note-on and note-off
   bytes into Web Audio voices. The embed would select it by default. It needs no MIDI permission,
   and Web MIDI devices would still be listed beside it.

   It still needs a user gesture. Starting Web Audio sound outside a user input event is subject to
   the browser's autoplay rules, and by default Firefox lets an audio context play only after
   sticky activation [105]. The `autoplay` Permissions Policy defaults to `self`, so a same-origin
   iframe is not blocked by policy [105]. Whether a click on the lesson page counts as activation
   inside the iframe is UNVERIFIED. So the embed is silent until the reader has interacted with
   it. The first point above already makes the reader start Playback, so creating or resuming the
   audio context on that action fits it. Learn Orca has the same constraint, because Enfer is Web
   Audio too (inference).

   How many voices, which sounds, and whether native gets it too are design decisions for an ADR.
   A Tone.js iframe like Enfer is possible, but bridging JavaScript from eframe costs more than Web
   Audio from Rust through `web-sys` (inference).

## Sources

1. https://metasyn.srht.site/learn-orca/js/main.js
2. https://metasyn.srht.site/learn-orca/js/examples.js
3. https://metasyn.srht.site/learn-orca/
4. https://github.com/metasyn/learn-orca (README: moved to sourcehut); https://git.sr.ht/~metasyn/learn-orca (not fetchable)
5. https://registry.npmjs.org/astro
6. https://github.com/withastro/starlight/releases/latest
7. https://registry.npmjs.org/@astrojs/starlight/latest
8. https://github.com/withastro/astro
9. https://github.com/withastro/starlight
10. https://starlight.astro.build/
11. https://docs.astro.build/en/guides/configuring-astro/
12. https://docs.astro.build/llms.txt
13. https://starlight.astro.build/resources/plugins/
14. https://docs.astro.build/en/basics/project-structure/
15. https://docs.astro.build/en/reference/directives-reference/
16. https://starlight.astro.build/guides/sidebar/
17. https://starlight.astro.build/guides/site-search/
18. https://starlight.astro.build/guides/components/
19. https://docs.astro.build/en/guides/deploy/github/
20. https://docs.rs/eframe/latest/eframe/
21. https://github.com/rust-lang/mdBook/releases/latest
22. https://github.com/rust-lang/mdBook/releases
23. https://github.com/rust-lang/mdBook
24. https://rust-lang.github.io/mdBook/format/configuration/renderers.html
25. https://rust-lang.github.io/mdBook/llms.txt (not found)
26. https://rust-lang.github.io/mdBook/guide/creating.html
32. https://github.com/getzola/zola/releases/latest
33. https://github.com/getzola/zola
35. https://www.getzola.org/llms.txt (not found)
36. https://www.getzola.org/documentation/getting-started/directory-structure/
37. https://www.getzola.org/documentation/content/shortcodes/
38. https://www.getzola.org/documentation/content/search/
41. https://registry.npmjs.org/vitepress
42. https://github.com/vuejs/vitepress/releases/latest
43. https://github.com/vuejs/vitepress
44. https://vitepress.dev/llms.txt
45. https://vitepress.dev/guide/asset-handling
48. https://vitepress.dev/guide/using-vue
49. https://vitepress.dev/reference/default-theme-search
51. https://github.com/facebook/docusaurus/releases/latest
52. https://github.com/facebook/docusaurus
54. https://docusaurus.io/llms.txt (not found)
55. https://docusaurus.io/docs/static-assets
57. https://docusaurus.io/docs/search
59. https://github.com/11ty/eleventy/releases/latest
62. https://www.11ty.dev/docs/languages/markdown/
63. https://www.11ty.dev/llms.txt (not found)
64. https://www.11ty.dev/docs/copy/
65. https://github.com/gohugoio/hugo/releases/latest
67. https://github.com/gohugoio/hugo
68. https://gohugo.io/llms.txt (not found)
69. https://gohugo.io/getting-started/directory-structure/
70. https://gohugo.io/content-management/shortcodes/
71. https://gohugo.io/configuration/markup/
72. https://github.com/trunk-rs/trunk/releases
75. https://trunk-rs.github.io/trunk/guide/print.html
77. https://github.com/orgs/community/discussions/13309
78. https://docs.github.com/en/enterprise/2.14/user/articles/mime-types-on-github-pages
79. https://raw.githubusercontent.com/jshttp/mime-db/master/HISTORY.md
81. https://raw.githubusercontent.com/emilk/egui/main/.github/workflows/deploy_web_demo.yml
83. https://github.com/actions/deploy-pages/releases
84. https://github.com/actions/upload-pages-artifact/releases
85. https://github.com/actions/configure-pages/releases
86. https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated
87. https://developers.cloudflare.com/workers/static-assets/headers/
88. https://developers.cloudflare.com/pages/configuration/headers/
90. https://docs.netlify.com/manage/routing/headers/
91. https://raw.githubusercontent.com/emilk/egui/main/crates/eframe/CHANGELOG.md
93. https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Permissions-Policy/midi
94. https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Permissions_Policy
97. https://github.com/emilk/egui/releases (0.28.0 notes)
99. https://raw.githubusercontent.com/emilk/egui/main/crates/eframe/src/epi.rs (`WebOptions`)
100. https://raw.githubusercontent.com/emilk/eframe_template/main/README.md
101. https://nodejs.org/dist/index.json
102. https://raw.githubusercontent.com/trunk-rs/trunk/main/guide/src/commands/index.md
103. https://raw.githubusercontent.com/trunk-rs/trunk/v0.21.14/guide/src/SUMMARY.md ; https://raw.githubusercontent.com/trunk-rs/trunk/main/guide/src/SUMMARY.md
104. https://raw.githubusercontent.com/trunk-rs/trunk/main/guide/src/configuration/index.md
105. https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay

Source numbers follow the research notes they came from, so unused numbers are skipped.
