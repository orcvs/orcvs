# Trace: Site (SITE) — inventory at c8cc197

Owned ADRs: 0086, 0087 (site clauses), 0091, 0093, 0094, 0097, 0101. Engineering guidance read:
`.agents/skills/website/SKILL.md`. Topic boundary used here (proposed): SITE owns the console's
opening by URL and by message, the page↔console message protocol, the page's audio-context
lifecycle and message timing, the web settings element, Copy Link, and the console/site build and
deploy chain. PLAY owns the Built-in Synth's voices, effects, core and destination selection in
general; CONS owns settings keys, the top bar and the Panel.

## Source disposition

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| ADR 0086 | `/play/?small` opens Linked (neither restores nor stores), on an empty Source, Playback stopped, Grid and Panel without menu bar | retained — SITE-010 Small console opening | SITE | source-stated |
| ADR 0086 | Small console shows a bar holding only its notices, only while it has any | retained — SITE-011 Small console notice bar | SITE (CONS draws the bar) | source-stated |
| ADR 0086 | Message is `{ type: "orcvs.open", source }` posted by the framing page, from the console's origin; other window/origin/type or non-string `source` ignored | retained — SITE-012 Open-message contract | SITE | source-stated |
| ADR 0086 | `source` read as Source File by the `#source=` reader under the same byte limit; refusal keeps the open Source and raises a notice; acceptance replaces it with Playback stopped | retained — SITE-013 Sent Source acceptance | SITE | source-stated |
| ADR 0086 | Only the latest message waiting for a frame opens | retained — SITE-014 Latest message wins | SITE | source-stated |
| ADR 0086 (Consequences) | Console posts `{ type: "orcvs.ready" }` once it listens; the page sends nothing before; a console that never readies is the page's to remove | retained — SITE-015 Ready handshake | SITE | source-stated |
| ADR 0086 | A page that shows examples holds one small console and sends each example by message | retained — SITE-016 One small console per page | SITE | source-stated |
| ADR 0086 (Consequences) | Small console has no File menu, cannot save; its Source lasts as long as the page | retained — SITE-017 Small console keeps nothing | SITE | source-stated |
| ADR 0086 (Consequences) | `#source=` and `?example=` stay; `/play/` keeps them | retained — SITE-020 URL opening | SITE | source-stated |
| ADR 0086 | Rejected: console per example; new tab/window; any-origin messages | historical | reasoning | source-stated |
| ADR 0087 (doc) | Status: resume clause amended by 0089; CC effects amended by 0088; channel 08 acid voice / kit on 09–0F amended by 0099; kit GM keys by 2026-11-13 amendment | split below | site clauses here; synth clauses PLAY | source-stated |
| ADR 0087 | Page announces with `{ type: "orcvs.synth", available }`, after the console's `orcvs.ready` | retained — SITE-040 Synth announcement | SITE | source-stated |
| ADR 0087 | While announced, console lists "Built-in Synth" under **O** after every MIDI port; chosen like any destination; auto-picked only when no port exists | redirected — PLAY | destination selection | proposed |
| ADR 0087 | A Linked or small console always picks it | retained — SITE-041 Linked console picks the synth | SITE (link-opened consoles) | proposed (boundary) |
| ADR 0087 | Choosing it posts each message Playback hands the destination as `{ type: "orcvs.midi", bytes, time }`, from the console's origin to the framing page only | retained — SITE-042 MIDI-message protocol | SITE | source-stated |
| ADR 0087 | `time` is the Tick's planned instant on the page clock (`performance.timeOrigin + performance.now()`) | retained — SITE-042 | SITE | source-stated |
| ADR 0087 | Synth plays a fixed delay after `time` (50 ms target); a message late by more than the delay sounds at once | retained — SITE-043 Page plays at planned time plus delay | SITE (page host); PLAY may cite | proposed (boundary) |
| ADR 0087 | One Rust synth core with no audio I/O, own WASM in an AudioWorklet; page passes each `orcvs.midi` to the worklet port; native later through cpal | redirected — PLAY (core); page pass-through stays in SITE-043 | synth core | proposed |
| ADR 0087 | Channel voices, 808 kit, velocity-only level, Control Change handling, 16 voices | redirected — PLAY | synth behaviour (also amended by 0088, 0099, 2026-11-13) | source-stated |
| ADR 0087 | Audio context starts on the console's `orcvs.wake` (each click/key) or the first `orcvs.midi` of a run | superseded in part by ADR 0089 (gesture resumes the context itself, on the page or in a console frame; first message stays as fallback); `orcvs.wake` retained — SITE-044 Audio resumes in the gesture | SITE | source-stated |
| ADR 0087 | Page tells every console `{ type: "orcvs.audio", running }`; a console holds a start on the synth until audio runs, at most 500 ms | retained — SITE-045 Start waits for audio | SITE | source-stated |
| ADR 0087 | A page that cannot sound announces `available: false` with a reason; choosing it answers the reason as a connect failure; Playback runs on silently | retained — SITE-046 Unavailable synth | SITE | source-stated |
| ADR 0087 (Consequences) | Every console on a page plays through the page's one synth and shares its voices | retained — SITE-047 One synth per page | SITE | source-stated |
| ADR 0087 (Consequences) | An unframed console (native, or `/play/` alone) offers no Built-in Synth until native gets the core | retained — SITE-048 Unframed console has no synth | SITE | source-stated |
| ADR 0087 (Consequences) | Synth WASM and worklet built with the web console and shipped in `console-web`; site build compiles no Rust | retained — SITE-049 Synth ships with the console | SITE | source-stated |
| ADR 0087 (Consequences) | `orcvs.midi` carries Stop's safety action; `MidiConnection` gains planned time other destinations may ignore | redirected — PLAY | output policy | proposed |
| ADR 0087 | Rejected: Tone.js bridge, Enfer port, cpal web host, web-sys nodes, SoundFont; "no volume or effects control now" | historical (effects line overtaken by ADR 0088, PLAY) | reasoning | source-stated |
| ADR 0091 | Status: amends proposed ADR 0079's "web reads no settings file"; phone Cell amended by ADR 0102 | split below | — | source-stated |
| ADR 0091 | Web console reads `config.toml` text from one `<script type="application/toml" id="orcvs_config">` with native's reader, keys, bounds and notices; `data-media` applies it only when matched at startup; none/unmatched = defaults; read once | retained — SITE-050 Page settings element | SITE | source-stated |
| ADR 0091 | `[minimap] open` and `[diagnostics] open` are settings on native and web; defaults Minimap open, Diagnostics closed; View menu still toggles | redirected — CONS | settings keys and views | proposed |
| ADR 0091 | Shipped page holds a phone setting under `(max-width: 600px)`: 24×36 Cells, Minimap closed; Grid stays 256×256 | retained — SITE-052 Shipped phone setting (Cell size superseded by ADR 0102: 16×24) | SITE | source-stated |
| ADR 0091 (Consequences) | Page settings report problems through the settings notice naming the page | retained — SITE-051 Page settings notices | SITE | source-stated |
| ADR 0091 (Consequences) | A framing page passes nothing; it gets the phone setting from `/play/` | retained — SITE-053 Framing pages pass no settings | SITE | source-stated |
| ADR 0091 | Considered: URL parameters; Rust viewport layout; settings by message (arrive after start) | historical | reasoning | source-stated |
| ADR 0093 (doc) | Amended by ADR 0101 (previews take their own console; retention) | split below | — | source-stated |
| ADR 0093 | Console key: one hash over git objects of the `console-web.yml` path filter, printed by `scripts/console-key.sh` reading that filter | retained — SITE-070 Console key | SITE | source-stated |
| ADR 0093 | On `main` each build uploaded as `console-web-<key>.tar.gz` beside the rolling `console-web.tar.gz` | retained — SITE-073 Archive naming; SITE-077 Rolling console | SITE | source-stated |
| ADR 0093 | Rolling console is what previews take | superseded by ADR 0101 (previews wait for their key) | — | source-stated |
| ADR 0093 | Production build (`WORKERS_CI_BRANCH=main`) downloads its key, retry 20 s up to 15 min; failure names the key, deploys nothing | retained — SITE-075 Builds wait for their key (scope widened by 0101) | SITE | source-stated |
| ADR 0093 | No deploy hook; GitHub holds no Cloudflare credential; failed builds retried from Cloudflare's dashboard | retained — SITE-079 No deploy credential | SITE | source-stated |
| ADR 0093 | Site build never compiles the console (Toby 2026-10-26) | retained — SITE-081 Site never compiles the console | SITE | source-stated |
| ADR 0093 (Consequences) | Cloudflare build watch paths include the console's paths or are unset | retained — SITE-080 Build watch paths | SITE (dashboard) | source-stated |
| ADR 0093 (Consequences) | Keep newest 20 keyed archives | superseded by ADR 0101 (100 main / 50 branch) | — | source-stated |
| ADR 0093 | Considered: deploy hook; name commit; build console in site build | historical | reasoning | source-stated |
| ADR 0094 | `/` frames `/play/` with its own query and fragment; link names are the console's; site keeps no list | retained — SITE-030 `/` passes its address on | SITE | source-stated |
| ADR 0094 | A linked console opens stopped, neither reads nor writes saved Source; plain `orcvs.com` reopens it | retained — SITE-021 Linked console keeps saved work | SITE | source-stated |
| ADR 0094 (Consequences) | Link names are public: renaming a shipped example breaks links | retained — SITE-024 Link names are stable | SITE | source-stated |
| ADR 0094 (Consequences) | `/?small` opens the small console on `/`; address read once at load | retained — SITE-030 | SITE | source-stated |
| ADR 0094 | Considered: page per example; console writes address bar | historical | reasoning | source-stated |
| ADR 0097 | Top bar share icon `↗` beside the mode toggle, native and web, puts the site link to the running Source on the clipboard | retained — SITE-060 Copy Link control | SITE (CONS lays out the bar) | source-stated |
| ADR 0097 | Unchanged shipped example → `https://orcvs.com/?example=<name>`; else `#source=` with every byte but `[A-Za-z0-9-._~]` percent-encoded | retained — SITE-061 Link form | SITE | source-stated |
| ADR 0097 | Encoded text over the linked reader's limit is refused with a notice | retained — SITE-062 Over-limit link refused | SITE | source-stated |
| ADR 0097 | Link always names `https://orcvs.com/` whatever page or build runs the console (previews copy production links) | retained — SITE-063 Links name production | SITE | source-stated |
| ADR 0097 | Considered: File → Copy Link (shipped first, replaced); site-side share; address bar | historical | reasoning | source-stated |
| ADR 0101 | `console-web.yml` keeps the path filter, runs on push to any branch except `gh-pages` and the merge queue's; first job computes key, lists every release page, fails on API errors other than missing release; builds nothing when key published | retained — SITE-072 Console build runs per changed push | SITE | source-stated |
| ADR 0101 | Branch changing nothing in the console takes `main`'s archive of its key | retained — SITE-073 | SITE | source-stated |
| ADR 0101 | `main` → `console-web-<key>`, others → `console-preview-<key>`; `main` counts only its own; production takes only `main`'s; a branch counts either | retained — SITE-073 Archive naming | SITE | source-stated |
| ADR 0101 | Archive never replaced: `publish` re-lists, leaves existing, uploads without `--clobber`; builds of one archive run one at a time | retained — SITE-074 Archives immutable | SITE | source-stated |
| ADR 0101 | No push cancels another's run; every pushed commit's run finishes | retained — SITE-074 | SITE | source-stated |
| ADR 0101 | Every Cloudflare build (`WORKERS_CI_BRANCH` set) waits; production `main`'s archive, preview either; `dependabot/` takes the rolling console | retained — SITE-075, SITE-076 Dependabot and local builds | SITE | source-stated |
| ADR 0101 | Key's test fails when a workspace member, out-of-crate embed or Trunk hook script is missing from the filter | retained — SITE-071 Key covers every input | SITE | source-stated |
| ADR 0101 | Keep newest 100 of `main`'s and 50 of branches'; never prune `main`'s current key; pruned branch key rebuilt by running Console web on the branch | retained — SITE-078 Archive retention | SITE | source-stated |
| ADR 0101 (Consequences) | Dependabot console still built; branch run holds `contents: write`; forks no longer run the console build | retained — SITE-076 / SITE-072 notes | SITE | source-stated |
| ADR 0101 | Considered: label, workflow_dispatch only, pull_request trigger, run every push | historical | reasoning | source-stated |
| ADR 0089 (CONS-owned) | Gesture resumes audio; page asks for media playback (iOS) | retained into SITE-044 (amendment of 0087 site clause) | SITE cites CONS's ADR | source-stated |
| ADR 0102 (CONS-owned) | Phone Cell 16×24 | retained into SITE-052 | SITE cites | source-stated |
| CONTEXT.md "Built-in Synth" | "a page that frames the console offers it as a MIDI destination"; "listed after every MIDI port…, and a Linked or small console picks it" | retained — SITE-040/041 (offer, Linked pick); listing redirected PLAY | No Rust comment cites the entry | source-stated |
| CONTEXT.md "Built-in Synth" | "plays each message at its Tick's planned time plus a fixed offset" | retained — SITE-043 | No Rust comment cites | proposed (boundary) |
| CONTEXT.md "Sound Module" | "row of the site's page below the console" | retained — SITE-031 (placement); behaviour redirected PLAY (0088) | No Rust comment cites | proposed |
| CONTEXT.md "Source File" | Format a link and a message carry | redirected — LANG | reader owned by LANG | proposed |
| CONTEXT.md (absent) | "Linked" and "small console" have no entries though the Built-in Synth entry and 0086/0087/0094 use them | open — see Conflicts Q6 | glossary gap | proposed |
| `.scratch/orcvs-site/spec.md` (Loading a Source, Saved work) | Byte limit checked before decoding (16 KiB in code); unknown example or refused text opens the empty Source with a message; both `?example` and `#source` refused | retained — SITE-022 Link refusal (no ADR; 0086 refers to "the same byte limit") | SITE | proposed (.scratch ruling) |
| `.scratch/orcvs-site/spec.md` (Service worker) | Linked and small consoles register no service worker; `/play/` keeps it, scoped to `/play/` | retained — SITE-023 Embeds register no worker | SITE | proposed (.scratch ruling) |
| `.scratch/orcvs-site/spec.md` + issue 05 | A deploy replaces a returning reader's cached console | retained — SITE-082 Deploy replaces cached console | SITE | proposed (.scratch ruling) |
| `.scratch/site-as-console/spec.md` | `/` is `/play/` filling the window, console's own splash, Sound Module row below | retained — SITE-031 Home page is the console | SITE | proposed (.scratch ruling, Toby 2026-10-26) |
| `.scratch/site-as-console/spec.md` | Pages carry the repository's docs (`docs/first-result.md`, `docs/functions.md`), no site prose; each Source block drawn as a Grid; nothing opens a new tab or window | retained — SITE-032 Pages render repository docs | SITE | proposed (.scratch ruling) |
| `.scratch/site-as-console/spec.md` | Every page wears Grayscale Dark/Light and Monaspace Neon, following reader appearance | presentation-only | styling | proposed |
| `.scratch/site-as-console/spec.md` / `orcvs-site/13` | Help menu offers Learn and Reference | redirected — CONS | console menu | proposed |
| `.agents/skills/website/SKILL.md` | Embed, link, started, deploy contracts | evidence only (engineering guidance; restates 0086/0093/0094/0101) | — | source-stated (run record) |

## Evidence and gaps

Test assertions were read where marked "(read)".

| Rule | Code | Tests | Status |
| --- | --- | --- | --- |
| SITE-010 | `console/src/linked_source.rs::opening` (`Opening::Small`); `console/src/console.rs::Console::new` (`small`, `stores_source`, `linked`) | `console/src/linked_source.rs::small_opens_the_small_console` (read); `console/src/console/tests.rs::a_small_console_opens_each_message_it_is_sent_stopped` (read); `console/src/console/kittest_tests.rs::a_small_console_shows_no_menu_bar` (read) | built and covered (no-store asserted only for `#source=` Linked, not `?small`; same flag) |
| SITE-011 | `console/src/console/menu_bar.rs::show_notice_bar` | `kittest_tests.rs::a_small_console_shows_a_refused_source_s_notice` (read) | built and covered |
| SITE-012 | `console/src/web_startup.rs::listen_to_page` (source==parent, origin==own, type, string `source`) | none in Rust (wasm-only listener); `site/tests/reference.spec.ts::pressing an example opens it…` exercises the happy path | built, no verified test (ignore cases) |
| SITE-013 | `linked_source.rs::source_message`; `console/src/console/files.rs::open_sent_source`, `open` (new environment, stopped) | `linked_source.rs::a_message_over_the_limit_or_not_a_source_file_is_refused` (read); `tests.rs::a_small_console_keeps_its_source_when_a_message_is_refused` (read); `site/tests/lesson.spec.ts::pressing two Grids opens each…, stopped` (read) | built and covered |
| SITE-014 | `linked_source::Inbox` (single `Option`, overwritten) | none for a burst | built, no verified test |
| SITE-015 | `web_startup.rs::listen_to_page` posts `orcvs.ready` after listening; `site/src/console/small.ts::start` waits, 30 s limit, `fail` removes frame | `site/tests/reference.spec.ts::a console that does not start says so, and the next press starts another` (read) | built and covered |
| SITE-016 | `site/src/console/small.ts`, `docs.ts` | `lesson.spec.ts::pressing two Grids…` (asserts one iframe, read); `smoke.spec.ts::nothing on the site opens a new tab or window` (read) | built and covered |
| SITE-017 | `menu_bar.rs` (no File menu when `small`), `stores_source=false` | `a_small_console_shows_no_menu_bar` (read) | built and covered |
| SITE-020 | `linked_source.rs::opening`, `example_named`, `source_from`; `console/src/console.rs::start_at` | `linked_source.rs` tests `an_example_name_opens_that_example`, `source_text_opens_as_a_source_file`, `every_byte_round_trips_through_the_source_file_writer` (read); `smoke.spec.ts` link starts | built and covered |
| SITE-021 | `Console::new` (`stores_source`), `Console::save` | `console/src/console/storage_tests.rs::a_linked_console_neither_restores_nor_stores_the_source`; `tests.rs::a_linked_source_opens_stopped_and_frames_never_start_it` (read) | built and covered |
| SITE-022 | `linked_source.rs::SOURCE_TEXT_LIMIT` (16 KiB of text after `source=`), `opening` | `text_over_the_limit_is_refused_before_decoding`, `an_unknown_example_name_is_refused`, `an_example_and_source_text_together_are_refused`, `malformed_percent_encoding_is_refused`; `tests.rs::a_link_the_console_cannot_open_starts_empty_and_says_why` (read) | built and covered |
| SITE-023 | `console/index.html` inline script (`linked` check) | `site/tests/smoke.spec.ts` "starts and registers no service worker" ×3 and `/play/ … registers its service worker` (read) | built and covered |
| SITE-024 | `console/src/examples.rs::Example::link_name` | `examples.rs::each_example_links_by_its_name` (asserts beat, melody, tracker, sequencer, knobs) | built and covered |
| SITE-030 | `site/src/pages/index.astro` (iframe `src = "/play/" + search + hash`, once) | `smoke.spec.ts` "`/${link}` frames `/play/${link}`" for `?example=tracker` and `#source=` (read); `?small` on `/` untested | built and covered (partial) |
| SITE-031 | `index.astro`, `site/src/console/sound-module.ts` | `smoke.spec.ts::/ is the console, filling the window above the Sound Module` (read) | built and covered |
| SITE-032 | `site/src/console/docs.ts`, `grid.ts` | `lesson.spec.ts` heading-for-heading and Grid tests; `reference.spec.ts` same (read) | built and covered |
| SITE-040 | `synth/web/synth-host.js` (answers `orcvs.ready` with announcement, `tell` to all frames); `console/src/built_in_synth.rs::receive`/`announcement` | `built_in_synth.rs::the_synth_is_listed_after_the_ports_only_while_a_page_announces_it`; site `synth.spec.ts` end to end | built and covered |
| SITE-041 | `built_in_synth.rs::SynthAnnouncements::observe` (`linked` = any non-Stored opening) | `a_linked_console_picks_the_synth_over_a_port`, `pending_midi_access_still_lets_a_linked_console_pick_the_synth` | built and covered |
| SITE-042 | `built_in_synth.rs::BrowserPage::post` (time = timeOrigin + now − since_due; target origin = own) | `built_in_synth.rs` FakePage tests record `since_due`; `synth.spec.ts::a framed console plays the page's synth at a constant offset from its Ticks` (Ticks 125 ms apart ±2, read) | built and covered |
| SITE-043 | `synth-host.js` (`DELAY_MS = 50`, `contextTime`) | `synth.spec.ts` constant-offset test asserts `|at − planned − 50| < 2` (read) | built and covered; startup backlog differs (Conflicts Q3) |
| SITE-044 | `synth-host.js::resumeOnGestures` (pointerdown/touchend/keydown, capture, on page and each ready frame; `audioSession.type="playback"`); `web_startup.rs::wake_page_audio`; first-message fallback in host | `synth.spec.ts::a click in the console starts the page's audio before Space`, `on a phone › a tap … starts the page's audio before any note` (read) | built and covered |
| SITE-045 | host `tell(audio())`; `console/src/console/playback.rs::PlaybackStart` (`AUDIO_WAIT = 0.5`), `built_in_synth.rs::start_is_early` | `playback.rs` tests `an_early_request_is_held_rather_than_granted`, `a_held_start_is_granted_once_it_has_waited_long_enough`; `built_in_synth.rs::a_start_is_early_only_while_stopped_asleep_and_on_the_synth`; `synth.spec.ts` first-visit tests (`firstMessageFound == "running"`, read) | built and covered |
| SITE-046 | host `announcement` rejection; `WithBuiltInSynth::connect` → `MidiError(reason)` | `an_unavailable_synth_answers_its_reason_on_connect`, `a_linked_console_shows_why_an_unavailable_synth_cannot_play` | built and covered |
| SITE-047 | one `synth-host.js` per page; `small.ts::loadSynth` once | `synth.spec.ts` constant-offset test frames a second console beside the small one | built and covered |
| SITE-048 | `web_startup.rs::framing` → `None` ⇒ no listener, never announced; native builds no `BrowserPage` | none direct | built, no verified test. Native synth tracked by `.scratch/orcvs-site/issues/14-give-native-the-built-in-synth.md` (needs-triage) |
| SITE-049 | `console/Trunk.toml` post-build hook `scripts/stage-synth.sh` | site suites load `/play/synth-host.js` from the console archive | built and covered (indirect) |
| SITE-050 | `console/src/web_startup.rs::page_settings`; `console/src/config.rs::Config::start` (wasm), `from_page` | `config.rs::the_page_settings_are_read_as_the_file_is_and_named_in_a_notice` (read); `data-media` matching untested | built and covered (parsing); media gate built, no verified test |
| SITE-051 | `config.rs::from_page` source name "the page's `orcvs_config` element" | same test asserts notice names `orcvs_config` (read) | built and covered |
| SITE-052 | `console/index.html` element: `[cell] width=16 height=24`, `[minimap] open=false`, `data-media="(max-width: 600px)"` | none asserts the shipped element's content or effect | built, no verified test |
| SITE-053 | `index.astro` passes nothing | none | built, no verified test (structural) |
| SITE-060 | `console/src/console/menu_bar.rs::share_control`, `SHARE_GLYPH`; `files.rs::copy_link` | `kittest_tests.rs::copy_link_puts_the_link_to_the_running_source_on_the_clipboard` (read); `the_top_bar_centres_its_menus_and_opens_them_below_itself` (share beside toggle); `menu_bar_layout.rs::the_notices_sit_one_readout_group_gap_from_the_share_icon` | built and covered (native harness; web path same code) |
| SITE-061 | `linked_source.rs::link`, `percent_encoded` | `an_unchanged_example_links_by_its_name`, `any_other_source_links_as_text_that_opens_it_again` (read) | built and covered |
| SITE-062 | `link` limit check; `copy_link` → notice | `a_source_whose_link_is_over_the_limit_is_refused` (read; notice path not asserted) | built and covered (unit) |
| SITE-063 | `linked_source.rs::SITE` const | link tests assert `https://orcvs.com/` prefix | built and covered |
| SITE-070 | `scripts/console-key.sh` | `site/scripts/tests/console.sh` (key changes only with console paths; glob/empty filter fails) | built and covered |
| SITE-071 | path filter in `.github/workflows/console-web.yml` | `site/scripts/tests/console.sh` members/embeds/Trunk-hook checks | built and covered |
| SITE-072 | `console-web.yml` `on.push` (branches-ignore, paths), `key` job; `scripts/console-assets.sh` (404 vs other errors, `--paginate`) | none (workflow); tooling-contract gates only | built, no verified test |
| SITE-073 | `console-web.yml` `key` job (`accepted` regex), `publish` | `site/scripts/tests/console.sh` "Production never takes a console a branch published" (site side) | built and covered (site side); workflow side untested |
| SITE-074 | `publish` re-list + plain `gh release upload`; `build.concurrency` per archive, `cancel-in-progress: false` | none | code differs (benign): workflow comment says a third queued run "replaces the waiting one" — GitHub concurrency cancels a pending run; same archive is still published. No issue found |
| SITE-075 | `site/scripts/console.sh` (45 tries × 20 s, main-only for production, error names key) | `site/scripts/tests/console.sh` production/preview wait, 44 sleeps, failure names key | built and covered |
| SITE-076 | `console.sh` `dependabot/*` branch; `console-web.yml` `publish` skips dependabot | `site/scripts/tests/console.sh` "A local build, and a Dependabot branch's build, take the rolling console" | built and covered |
| SITE-077 | `console-web.yml` `latest` job | none | built, no verified test |
| SITE-078 | `latest` job prune loop (`web:100 preview:50`, excludes main's key) | none | built, no verified test |
| SITE-079 | no Cloudflare secret in `.github/workflows/` (grep) | none | built, no verified test |
| SITE-080 | Cloudflare dashboard, not in repo | none | not verifiable in repo; `.scratch/orcvs-site/issues/26-build-the-site-on-console-pushes.md` (ready-for-human) |
| SITE-081 | `site/scripts/console.sh` downloads/copies only | `.github/workflows/site.yml` builds with no Rust toolchain step | built, no verified test |
| SITE-082 | `console/assets/sw.js` network-first, `site/public/_headers` | `site/tests/deploy.spec.ts::a second deploy replaces the console a returning reader cached` (read) | built and covered. Preview-URL item of `.scratch/orcvs-site/issues/05-deploy-to-cloudflare-with-previews.md` still ready-for-human |

Minor: `.agents/skills/website/SKILL.md` says "up to 16 KiB of fragment"; the code counts the 16 KiB
after `source=`, not the whole fragment.

## Conflicts and open questions

**Q1. ADR 0091 (accepted) relies on keys from ADR 0079 (proposed).** 0091: "read by the same
reader as native's file, with the same keys, bounds and notices"; "the native default: 12 by 18
point Cells"; it "Amends ADR 0079's 'The web reads no settings file'". 0079 status: "proposed";
0079 defines `[cell] width/height`, 12×18 default, bounds 8–64. Code built both
(`console/src/config.rs`, `grid_viewport::CELL_SIZE`); ADR 0102 (accepted) sets the phone value
16×24 in the same keys. Recommendation: SITE-050/052 restate 0091 and 0102 as decided and point
to CONS for the `[cell]` key definitions, with CONS/main recording that the proposed 0079 is what
the code built. Consequence: until 0079 is accepted, the phone setting's keys have no accepted
definition, only an accepted user.

**Q2. `.scratch`-only rulings with no ADR.** The 16 KiB limit and decode-after-check, unknown
example / refused text opening the empty Source with a notice, refusing both parameters, embeds
skipping the service worker, deploys replacing a cached console, "the site is the console", and
pages rendering repository docs are decided only in `.scratch/orcvs-site/spec.md` and
`.scratch/site-as-console/spec.md`. ADRs 0086 and 0094 cite some of this as existing ("the same
byte limit"). The run record says binding decisions were recorded as ADRs. Recommendation: keep
them as SITE rules citing the `.scratch` spec as their basis and flag them as candidates for an ADR.
Consequence: the snapshot states rules no ADR backs. If main rules otherwise, SITE-022/023/031/032/082
drop to evidence, and the snapshot no longer says what a link may refuse.

**Q3. Startup backlog vs "sounds at once".** ADR 0087: "A message whose time has already passed by
more than the delay sounds at once." `synth/web/synth-host.js` (`START_MS = 500`, `due`) drops
messages that arrived while the context started and are more than 500 ms late. The behaviour comes
from the text of `.scratch/orcvs-site/issues/16-wake-the-synth-before-the-first-note.md` (resolved).
No test covers the drop. Recommendation: SITE-043 states 0087's rule for a running context and
notes the startup drop as built behaviour no ADR decides. Consequence: either an ADR amendment
records the drop, or the drop is a gap against 0087.

**Q4. Where 0087's destination and timing clauses belong.** The CONTEXT.md Built-in Synth entry
holds the Linked pick and the planned-time-plus-offset rule together with the voices. Proposed
split: Linked pick, message protocol and timing go to SITE (041–043), and listing order and
auto-pick go to PLAY. Recommendation: PLAY confirms. Consequence: otherwise a rule is stated twice
or not at all.

**Q5. ADR 0101 "No push cancels another's run".** The ADR says "every pushed commit's run
finishes". `console-web.yml` uses a concurrency group per archive with `cancel-in-progress: false`.
GitHub replaces a pending run in that group with a newer one, and the comment says so ("a third
replaces the waiting one, which builds the same archive"). The archive still gets published, so
the ADR's stated purpose holds. Recommendation: restate SITE-074 by its purpose: every pushed key
gets published and no run is cancelled while it is building. List the literal difference as a gap.

**Q6. "Linked" and "small console" are undefined in CONTEXT.md.** The CONTEXT.md Built-in Synth
entry and ADRs 0086, 0087 and 0094 use both words. The code defines them
(`linked_source.rs::Opening`): Linked means any URL- or `?small`-opened console, including one whose
link was refused. Recommendation: the requirement defines them by reference to 0086/0094, and the
glossary gap goes to `/domain-modeling`. No ADR change is needed.

## Fresh-reader questions

1. **"How do I send a friend a link that opens the Tracker example with sound?"** Use
   `https://orcvs.com/?example=tracker`, or press the top bar's `↗` (Copy Link) with the unchanged
   Tracker open. `/` frames `/play/` with the same query, inside the page that holds the Built-in
   Synth. The console opens stopped and leaves the friend's saved Source alone. The link name is the
   example's name in lowercase (`console/src/examples.rs::link_name`). Sources: ADR 0094, ADR 0097,
   `console/src/linked_source.rs::link`, `site/src/pages/index.astro`.
2. **"My branch changes `console/`. Why is the preview build stuck or failing with 'was not
   published within 15 minutes'?"** Every Cloudflare build waits for its own commit's console key.
   A preview takes `console-web-<key>` or `console-preview-<key>`. `console-web.yml` publishes the
   branch's archive only when the push changes a filtered path. If that run failed, or the key was
   pruned, run Console web on the branch and retry the build from Cloudflare. A `dependabot/`
   branch takes the rolling console. Sources: ADR 0093, ADR 0101, `site/scripts/console.sh`,
   `.github/workflows/console-web.yml`.
3. **Crosses CONS: "On a phone the Cells are too small and the Minimap covers the Grid. Where do I
   change that?"** Edit the `orcvs_config` element in `console/index.html`, which applies under
   `data-media="(max-width: 600px)"`. Today it sets `[cell] 16×24` and `[minimap] open=false`.
   Native's `config.toml` reader parses it, so the keys and bounds are CONS's. Pages that frame
   `/play/` pass nothing. Sources: ADR 0091, ADR 0102, ADR 0079 (proposed, key definitions),
   `console/src/web_startup.rs::page_settings`, `console/src/config.rs::from_page`.
4. **Crosses PLAY: "My MIDI device is connected, so why does an example on `/learn/` play through
   the Built-in Synth?"** The page's small console is Linked, and a Linked or small console always
   picks the Built-in Synth once the page announces it. An unlinked `/play/` picks the synth only
   when no port exists. What each channel plays is PLAY's. Sources: ADR 0087 (with 0088, 0099),
   `console/src/built_in_synth.rs::SynthAnnouncements::observe`, `docs/functions.md` § Built-in
   Synth.
5. **"I want the page to send the console a new message, say a tempo. What must it respect?"** The
   page sends nothing before the console's `orcvs.ready`. The console reads only messages whose
   source is its framing window and whose origin is its own, and ignores unknown types. Pages post
   with their own origin as target. Settings needed before the first frame cannot come by message:
   0091 rejected that. Sources: ADR 0086, ADR 0087, ADR 0091 (Considered options),
   `console/src/web_startup.rs::listen_to_page`, `site/src/console/small.ts`.
6. **"On an iPhone the first note of a run comes late or is silent. What should happen?"** The page
   resumes its audio context inside the reader's gesture, on the page or in the console's frame,
   and sets the media-playback audio session. The console posts `orcvs.wake` on each click or key
   and holds a start on the synth for up to 500 ms until `orcvs.audio` says the audio is running.
   The first `orcvs.midi` of a run is the fallback. Sources: ADR 0087, ADR 0089,
   `synth/web/synth-host.js`, `console/src/console/playback.rs::PlaybackStart`,
   `site/tests/synth.spec.ts`.

## Final rule mapping (drafting phase)

Document: `docs/requirements/site.md`. Each retained row above lands in the rule it names, with these merges and moves:

- SITE-017 merged into SITE-010 (small console keeps nothing).
- SITE-053 merged into SITE-052 (framing pages pass no settings).
- SITE-063 merged into SITE-061 (links name production).
- SITE-077 merged into SITE-073 (rolling console tracks `main`).
- SITE-022 split: the shared byte limit (ADR 0086, 0097) is binding. The 16 KiB value, the
  check-before-decode and the refusal handling (unknown example, unreadable text, both parameters)
  move to Open decisions (D3: no dated maintainer ruling; ticket 01 chose the value).
- SITE-023 (service worker) and SITE-082 (deploy replaces cached console) move to Open decisions
  (D3: undated spec decisions). Their IDs stay reserved.
- SITE-031 and SITE-032 are stated as "ruled 2026-10-26, no ADR" (`.scratch/site-as-console/spec.md`, Toby's dated direction).
- SITE-043 and SITE-074 keep the ADR wording. Q3 and Q5 are listed under "Accepted but not built, or built differently" (D9).
- SITE-050 and SITE-052 bind per ADR 0091 and 0102. The `[cell]` keys are pointed to CONS "Proposed, built" (D6).
- CONTEXT rows: the Built-in Synth offer and Linked pick land in SITE-040/041, the planned time plus offset in SITE-043, the Sound Module placement in SITE-031.
- Redirected rows to be stated elsewhere. PLAY: listing after ports and auto-pick when no port exists, the synth core in an AudioWorklet with native via cpal, voices and kit, `orcvs.midi` carrying the safety action, and `MidiConnection` planned time. CONS: the `[minimap] open` and `[diagnostics] open` keys, and the Help menu's Learn and Reference. LANG: the Source File reader.

## Audit fix (F1, F4)

- ADR 0088 and its 2026-11-14 amendment now belong to SITE (PLAY and SITE had each redirected them to the other). New rules:
  - SITE-033 Grid of Cells on `/` only
  - SITE-034 Channel light lasts 150 ms and shows no level
  - SITE-035 Chosen channel: `00` at load, follows the CC stream until the reader chooses, outlined, chosen by click or keys
  - SITE-036 Typing values; a digit past `7F` is refused; each change is sent as a CC on the chosen channel
  - SITE-037 Reset on load; `CU` and `DC` on `08`; Reset All Controllers shows `MD` as `00`
  - SITE-038 The `sound.magnitude` colour
- Checked against `site/src/console/sound-module.ts`, `sound-module.css` and `site/tests/sound-module.spec.ts`. Typing also ends following (`picked = true`). That is built, but no ADR states it, so no rule binds it.
- Redirected to PLAY: what each effect does to the sound and how the synth answers each controller (0088's Consequences, the amendment's first two bullets).
- ADR 0088 and 0099 are added to the marker.
- F4: SITE-043 now links to PLAY-540 for how late each Tick is. SITE-043, SITE-046 and SITE-048 already state the delay, the unavailable synth and the unframed console in full, so PLAY can link to them.
