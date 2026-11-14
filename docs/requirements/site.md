# Site requirements

> Snapshot: checked against `c8cc197` (2026-11-14); incorporates ADRs through 0109.
> ADRs accepted after 0109 bind where they differ, until this document is updated.
> Source ADRs: 0054, 0086, 0087, 0089, 0091, 0093, 0094, 0097, 0101, 0102. Reviewed, not applicable or wholly superseded: none wholly superseded; 0079 (proposed, owned by [console.md](console.md)).

This document owns the web console's life on a page: opening a Source by link or by message, the
messages between a console and the page that frames it, the page's audio, the web settings
element, Copy Link, and the build and deploy chain that puts a console on the site. The Built-in
Synth's voices, effects, listing order and automatic selection belong to [playback.md](playback.md).
Settings keys, the top bar and the Panel belong to [console.md](console.md). The Source File format
belongs to [language.md](language.md), and Ticks to [evaluation.md](evaluation.md). Terms are defined in
[CONTEXT.md](../../CONTEXT.md), and decisions are argued in [the ADRs](../adr/).

Here a console is **Linked** when its URL opens it by `?example=`, `#source=` or `?small`, whether
or not that link could be read. The **small console** is the Linked console at `/play/?small`.

## Opening a Source by link

### SITE-020 A console's URL opens a shipped example or Source File text, read once at startup

`/play/?example=<link-name>` opens the shipped example with that link name. `/play/#source=<text>`
opens percent-encoded Source File text. In that text `+` stays `+`, and `%` must be followed by two
hex digits. The console reads its URL once, at startup. A URL that names neither form opens the
stored Source.

Reason: a link is how a reader is sent one Source, and `/play/` keeps both forms beside the small
console's messages.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md) (Consequences), [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `console/src/linked_source.rs` `opening`, `source_from`, `percent_decoded`; `console/src/console.rs` `Console::start_at`; tests `console/src/linked_source.rs` `an_example_name_opens_that_example`, `source_text_opens_as_a_source_file`, `a_plus_is_a_plus_not_a_space`, `every_byte_round_trips_through_the_source_file_writer` (assertions inspected).

### SITE-021 A Linked console opens stopped and never reads or writes saved work

A Linked console opens with Playback stopped, so a link cannot start Playback by itself. It
neither restores the stored Source nor stores its own, so a link never replaces saved work, and
plain `orcvs.com` opens that work again. Edits in a Linked console are not saved.

Reason: a link someone sends must not play by itself or overwrite the reader's own Source.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md), [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/console.rs` `Console::new` (`stores_source`), `Console::save`; tests `console/src/console/storage_tests.rs` `a_linked_console_neither_restores_nor_stores_the_source`, `console/src/console/tests.rs` `a_linked_source_opens_stopped_and_frames_never_start_it` (assertions inspected).

### SITE-022 A link's text, a sent Source and a copied link share one byte limit

The `#source=` reader has one byte limit (`console/src/linked_source.rs` `SOURCE_TEXT_LIMIT`).
SITE-013 applies the same limit to a Source sent by message, and SITE-062 applies it to a copied
link.

Reason: one limit means a link the console copies always opens, and a message can carry no more
than a link.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md), [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `source_from`, `source_message`, `link`; tests `text_over_the_limit_is_refused_before_decoding`, `a_message_over_the_limit_or_not_a_source_file_is_refused`, `a_source_whose_link_is_over_the_limit_is_refused` (assertions inspected).

### SITE-024 Link names are the console's, and are public

A shipped example's link name is the one the console gives it. The site keeps no list of examples.
A link name appears in links in the wild, so renaming a shipped example breaks those links.

Reason: the site would otherwise keep a second copy of the example list beside the console's.
Not: a page per example on the site.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `console/src/examples.rs` `Example::link_name`; test `console/src/examples.rs` `each_example_links_by_its_name`.

## The small console

### SITE-010 The small console opens empty, stopped and keeps nothing

`/play/?small` opens Linked on an empty Source with Playback stopped. It shows the Grid and the
Panel without the menu bar. It has no File menu and cannot save, so its Source lasts as long as
the page.

Reason: a page shows many examples beside one running console, and none of them may touch the
reader's work.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md), [ADR 0089](../adr/0089-the-panel-plays-and-rewinds.md) (every console shows the Panel).
Evidence: `console/src/linked_source.rs` `opening` (`Opening::Small`); `console/src/console.rs` `Console::new`; tests `console/src/linked_source.rs` `small_opens_the_small_console`, `console/src/console/kittest_tests.rs` `a_small_console_shows_no_menu_bar` (assertions inspected).

### SITE-011 The small console shows a notice bar only while it has notices

While the small console has notices, it shows a bar holding only those notices.

Reason: a refused Source has to say why the open one stayed.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/console/menu_bar.rs` `show_notice_bar`; test `console/src/console/kittest_tests.rs` `a_small_console_shows_a_refused_source_s_notice` (assertions inspected).

### SITE-012 The console opens only `orcvs.open` messages from its framing page and origin

The open message is `{ type: "orcvs.open", source }`. It is posted by the page that frames the
console, from the console's own origin. The console ignores a message from another window or
origin, a message of another type, and a message without a string `source`.

Reason: otherwise any page that frames the console could replace its Source.
Not: accepting messages from any origin.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/web_startup.rs` `listen_to_page`; test `site/tests/reference.spec.ts` "pressing an example opens it in the page's small console" (happy path only).

### SITE-013 A sent Source opens stopped, or is refused with a notice

The console reads `source` as a Source File, the same way it reads `#source=` text and under the
same byte limit (SITE-022). If the reader refuses it, the open Source stays and a notice is
raised. If the reader accepts it, it replaces the open Source with Playback stopped, so a message
cannot start Playback.

Reason: a message is a link without a reload, and keeps a link's guarantees.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/linked_source.rs` `source_message`; `console/src/console/files.rs` `open_sent_source`; tests `console/src/console/tests.rs` `a_small_console_opens_each_message_it_is_sent_stopped`, `a_small_console_keeps_its_source_when_a_message_is_refused`, `site/tests/lesson.spec.ts` "pressing two Grids opens each in the one small console, stopped" (assertions inspected).

### SITE-014 Only the latest message waiting for a frame opens

If several Sources arrive before the next frame, only the last one opens.

Reason: a burst of presses should open what the reader pressed last, once.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/linked_source.rs` `Inbox`.

### SITE-015 A console says `orcvs.ready` once it listens, and the page sends nothing before

A framed console posts `{ type: "orcvs.ready" }` to its page once it listens for messages. The page
sends nothing before that message. If a console never says it is ready, removing it is the page's
job.

Reason: the console listens only once its WASM runs, so anything posted earlier would be lost.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md) (Consequences).
Evidence: `console/src/web_startup.rs` `listen_to_page`; `site/src/console/small.ts` `start`, `fail`; test `site/tests/reference.spec.ts` "a console that does not start says so, and the next press starts another" (assertions inspected).

### SITE-016 A page that shows examples holds one small console

A page with examples holds one small console and sends it each example by message.

Reason: a console per example pays a WASM start for each one.
Not: a console per example, or opening examples in a new tab or window.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `site/src/console/small.ts`; test `site/tests/lesson.spec.ts` "pressing two Grids opens each in the one small console, stopped" (asserts one frame; assertions inspected).

## The site's pages

### SITE-030 `/` frames `/play/` with its own query and fragment

`/` frames `/play/`, passing on its whole query and fragment, inside the page that holds the
Built-in Synth and the Sound Module. So `orcvs.com/?example=<link-name>` opens what
`/play/?example=<link-name>` opens, and `/?small` opens the small console. The address is read
once, when the page loads. Editing the fragment of an open page changes nothing until a reload.

Reason: a link to the site has to name an example and still play through the page's synth.
Not: the console writing its link into the address bar, because a reload would drop the reader's edits.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `site/src/pages/index.astro`; tests `site/tests/smoke.spec.ts` "`/${link}` frames `/play/${link}`" (assertions inspected).

### SITE-031 The home page is the console

`/` is the console filling the window, with the console's own splash. The Sound Module is the row
below it.

Reason: "The site should be the console."
Sources: ruled 2026-10-26, no ADR (`.scratch/site-as-console/spec.md`); [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `site/src/pages/index.astro`; test `site/tests/smoke.spec.ts` "/ is the console, filling the window above the Sound Module" (assertions inspected).

### SITE-032 Pages carry the repository's docs, draw each Source as a Grid, and open no new window

Site pages render the repository's own docs, `docs/first-result.md` and `docs/functions.md`, and add
no prose written for the site. Each Source block is drawn as a Grid of Cells, and pressing it sends
that Source to the page's small console (SITE-016). Nothing on the site opens a new tab or window.

Reason: "I do not want your copy", "Function source code references should use a grid", "Launching
examples in a new window is bad".
Sources: ruled 2026-10-26, no ADR (`.scratch/site-as-console/spec.md`).
Evidence: `site/src/console/docs.ts`, `grid.ts`; tests `site/tests/lesson.spec.ts` "the lesson is docs/first-result.md, heading for heading", `site/tests/reference.spec.ts` "every example is a Grid, and every starting Source opens in the console", `site/tests/smoke.spec.ts` "nothing on the site opens a new tab or window" (assertions inspected).

## The Built-in Synth on a page

### SITE-040 A page announces its synth with `orcvs.synth` after the console's `orcvs.ready`

A page that holds the Built-in Synth answers a console's `orcvs.ready` with
`{ type: "orcvs.synth", available }`. Listing and choosing the synth as a destination are stated in
[playback.md](playback.md).

Reason: the console has no audio code. It learns from the page that a synth is there.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` (ready handler, `tell`); `console/src/built_in_synth.rs` `receive`; test `console/src/built_in_synth.rs` `the_synth_is_listed_after_the_ports_only_while_a_page_announces_it`.

### SITE-041 A Linked console picks the Built-in Synth once it is announced

When the page announces the synth, a Linked console, the small console included, picks it as its
destination even if a MIDI port exists.

Reason: an example someone opens from a link or a page should be heard without any setup.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/built_in_synth.rs` `SynthAnnouncements::observe`; tests `a_linked_console_picks_the_synth_over_a_port`, `pending_midi_access_still_lets_a_linked_console_pick_the_synth`.

### SITE-042 Each message goes to the page as `orcvs.midi` stamped with its Tick's planned instant

While the synth is chosen, the console posts each message Playback hands the destination as
`{ type: "orcvs.midi", bytes, time }`. It posts only to the framing page, with the console's own
origin as the target. `time` is the Tick's planned instant on the clock both frames share,
`performance.timeOrigin` plus `performance.now()`, not the moment the message is sent. Stop's safety
action reaches the page the same way as any other message.

Reason: a late timer or a slow frame must not move a note.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/built_in_synth.rs` `BrowserPage::post`; test `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks" (Ticks 125 ms apart within 2 ms; assertions inspected).

### SITE-043 The page plays a message a fixed delay after its planned instant

The page plays each `orcvs.midi` a fixed delay after its `time`. The delay is set within a 50 ms
target. A message whose time has passed by more than the delay sounds at once. The page passes
each message on to the synth core, which [playback.md](playback.md) describes.

Reason: no note moves until lateness exceeds the delay.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` `DELAY_MS`, `contextTime`; test `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks" (asserts sounding at planned + 50 ms within 2 ms; assertions inspected).

### SITE-044 The reader's gesture resumes the page's audio

The page resumes its audio context inside each pointer, touch or key gesture, whether the gesture
is on the page or in a console's frame. When it does, it also asks for media playback, so iOS's
ring/silent switch does not mute it. A framed console posts `{ type: "orcvs.wake" }` to its page on
each click and key. If no gesture has resumed the context, the first `orcvs.midi` of a run resumes
it.

Reason: Safari resumes audio only inside a gesture, and the console's first message arrives after
the gesture that caused it.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md) as amended by [ADR 0089](../adr/0089-the-panel-plays-and-rewinds.md).
Evidence: `synth/web/synth-host.js` `resume`, `resumeOnGestures`, `start`; `console/src/web_startup.rs` `wake_page_audio`; tests `site/tests/synth.spec.ts` "a click in the console starts the page's audio before Space", "on a phone › a tap in the console's frame starts the page's audio before any note" (assertions inspected).

### SITE-045 A start on the synth waits up to 500 ms for the page's audio

The page tells every console `{ type: "orcvs.audio", running }` whenever its context starts or
stops. A Playback start on the Built-in Synth while the audio is not running is held until the
audio runs, for at most 500 ms.

Reason: starting audio takes a device over 100 ms. Without the hold, Tick 0 would sound late and the
Ticks after it on time.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/console/playback.rs` `PlaybackStart`, `AUDIO_WAIT`; `console/src/built_in_synth.rs` `SynthAnnouncements::start_is_early`; tests `console/src/console/playback.rs` `a_held_start_is_granted_once_it_has_waited_long_enough`, `site/tests/synth.spec.ts` "a first visit opens Knobs, and its first Tick sounds on time after …" (assertions inspected).

### SITE-046 A page that cannot make sound still announces the synth, unavailable

A page without audio announces `available: false` with a reason. The console still lists the
synth. Choosing it answers the reason as a connect failure, and Playback runs on silently.

Reason: an unavailable synth should explain itself, the way a MIDI device that has gone does.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` `announcement`; `console/src/built_in_synth.rs` `WithBuiltInSynth::connect`; tests `an_unavailable_synth_answers_its_reason_on_connect`, `a_linked_console_shows_why_an_unavailable_synth_cannot_play`.

### SITE-047 Every console on a page plays through that page's one synth

A page holds one Built-in Synth, and every console it frames plays through it. Two consoles playing
at once share its voices.

Reason: the synth belongs to the page, not to a console.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md) (Consequences).
Evidence: `site/src/console/small.ts` `loadSynth`; `synth/web/synth-host.js` `tell`; test `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks".

### SITE-048 A console no page frames offers no Built-in Synth

A native console, or `/play/` opened on its own, offers no Built-in Synth until native gets the
synth core.

Reason: the synth lives in the page that frames the console.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md) (Consequences).
Evidence: `console/src/web_startup.rs` `framing`, `listen_to_page`.

### SITE-049 The synth ships beside the web console, and the site compiles no Rust

The synth core's WASM and its worklet and page scripts are built with the web console and published
in the same archive, so the site build compiles no Rust.

Reason: the site build takes a built console and never compiles one (SITE-081).
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md) (Consequences).
Evidence: `console/Trunk.toml` post-build hook `scripts/stage-synth.sh`.

## Web settings

### SITE-050 The web console reads `config.toml` text from its page, once, at startup

`console/index.html` may hold one `<script type="application/toml" id="orcvs_config">` element. The
web console reads its text with native's settings reader, using the same keys, bounds and notices.
A `data-media` attribute holds a CSS media query, and the element applies only if the page matches
it when the console starts. Without the attribute, the element always applies. With no element, or
one whose media does not match, the console runs on the defaults. The keys themselves are stated in
[console.md](console.md).

Reason: the page that runs the console is what a deployment controls, and settings are needed
before the first frame.
Not: URL parameters per setting, a layout chosen inside the console by viewport, or settings sent
by message.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md).
Evidence: `console/src/web_startup.rs` `page_settings`; `console/src/config.rs` `Config::start`, `Config::from_page`; test `console/src/config.rs` `the_page_settings_are_read_as_the_file_is_and_named_in_a_notice` (assertions inspected).

### SITE-051 Problems in the page's settings are reported in the settings notice, naming the page

A key the page's element cannot supply is reported through the settings notice. The notice names
the page's `orcvs_config` element rather than a file.

Reason: the reader has to know where the bad value came from.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md) (Consequences).
Evidence: `console/src/config.rs` `Config::from_page`; test `the_page_settings_are_read_as_the_file_is_and_named_in_a_notice` (assertions inspected).

### SITE-052 The shipped page holds a phone setting, and framing pages pass nothing

The shipped `/play/` page holds a setting that applies under `(max-width: 600px)`. Under it the
console starts with 16 by 24 point Cells and the Minimap closed. The Grid stays 256 by 256, so a
phone shows fewer, larger Cells. A page that frames the console, such as the site, passes no
settings and gets the phone setting from `/play/` itself.

Reason: the native default Cell is too small to tap, and a phone window has no room for the Minimap.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md), [ADR 0102](../adr/0102-a-finger-pans-the-source-and-asks-for-the-keyboard.md) (16 by 24), [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md).
Evidence: `console/index.html` element `orcvs_config`; `site/src/pages/index.astro`.

## Copy Link

### SITE-060 The top bar's share icon copies the site link to the running Source

The top bar's share icon, `↗` beside the mode toggle, puts on the clipboard the site link that
opens the running Source. It works on native and on the web.

Reason: a reader otherwise has no way to get a link to send.
Not: a share control on the site, because the site cannot see the running Source and native could not share.
Sources: [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/console/menu_bar.rs` `share_control`; `console/src/console/files.rs` `copy_link`; tests `console/src/console/kittest_tests.rs` `copy_link_puts_the_link_to_the_running_source_on_the_clipboard`, `the_top_bar_centres_its_menus_and_opens_them_below_itself` (assertions inspected).

### SITE-061 A copied link names orcvs.com, by link name or as encoded text

A Source that is a shipped example, unchanged, links by its link name:
`https://orcvs.com/?example=<link-name>`. Any other Source links as its Source File text,
`https://orcvs.com/#source=…`, with every byte percent-encoded except letters, digits and `-._~`.
An edited example therefore links as text. The link always names `https://orcvs.com/`, the page
with the Built-in Synth, whichever page or build the console runs in, so a preview's console copies
a production link.

Reason: the encoded text survives any address bar or chat client, and the link always plays.
Sources: [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `link`, `percent_encoded`, `SITE`; tests `an_unchanged_example_links_by_its_name`, `any_other_source_links_as_text_that_opens_it_again` (assertions inspected).

### SITE-062 A Source too large to link is refused with a notice

If a Source's encoded text is over the byte limit (SITE-022), it is refused with a notice rather
than copied as a link that would not open.

Reason: a link that would not open is worse than none.
Sources: [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `link`; `console/src/console/files.rs` `copy_link`; test `a_source_whose_link_is_over_the_limit_is_refused` (assertions inspected).

## Console publication

### SITE-070 A commit's console has a key that reads the console's path filter

A commit's console key is one hash over the git objects of every path the console is built from.
That set of paths is exactly the path filter of the Console web workflow, and the key script reads
the filter rather than keeping its own copy. Two commits share a key exactly when they build the
same console.

Reason: the key needs only the commit's own tree, not its history.
Not: waiting for a published console that names its commit.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md).
Evidence: `scripts/console-key.sh`; test `site/scripts/tests/console.sh` (the key changes exactly with console paths; a glob or an empty filter fails).

### SITE-071 The path filter covers every input of the console build

The path filter must cover every workspace member, every file a crate embeds from outside itself,
and every script a Trunk hook runs. The console script's test fails when one is missing.

Reason: a missing path would give two different consoles one key.
Sources: [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `.github/workflows/console-web.yml` `on.push.paths`; test `site/scripts/tests/console.sh` (member, embed and Trunk-hook checks).

### SITE-072 A push that changes the console builds it unless its key is published

Console web runs on a push to any branch except `gh-pages` and the merge queue's, and only when the
push changes a filtered path. Its first job computes the key and lists the `console-web` release,
reading every page. It fails on any API error other than a missing release. If the key is already
published, the run builds nothing. A branch that changes nothing in the console takes `main`'s
archive of its key. A Dependabot branch's console is built but not published. Fork pull requests do
not run the console build.

Reason: every preview of a console change hears that change, and pushes that do not change the
console run nothing.
Not: an opt-in label, a manual run, the `pull_request` trigger, or running on every push.
Sources: [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `.github/workflows/console-web.yml` jobs `key`, `build`, `publish`; `scripts/console-assets.sh`.

### SITE-073 `main` and other branches publish under separate names, and the rolling console tracks `main`

`main` publishes `console-web-<key>.tar.gz` on the `console-web` release. Every other branch
publishes `console-preview-<key>.tar.gz`. `main` counts only its own archives as published, and a
branch counts either name. The rolling `console-web.tar.gz` is `main`'s current console. Local
builds take it.

Reason: separate names keep an honest branch run from ever touching what production takes.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md), [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `.github/workflows/console-web.yml` jobs `key` (accepted names), `publish`, `latest`; test `site/scripts/tests/console.sh` "Production never takes a console a branch published".

### SITE-074 An archive is never replaced, and every pushed console gets published

`publish` lists the release again, leaves any archive another run has already published, and
uploads without overwriting. Builds of one archive run one at a time. No push cancels another
push's run, so every pushed commit's console is published and no preview waits for a console that
never comes.

Reason: one key is one console, and a cancelled run would leave its commit's preview waiting.
Sources: [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `.github/workflows/console-web.yml` `build.concurrency`, `publish`.

### SITE-078 The release keeps 100 of `main`'s archives and 50 of the branches'

The release keeps the newest 100 of `main`'s keyed archives and the newest 50 of the branches'. It
never prunes `main`'s current key. A branch so far behind that its key was pruned rebuilds it by
running Console web on the branch, and the failed preview's message names that step.

Reason: archives accumulate on every console push.
Sources: [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `.github/workflows/console-web.yml` job `latest` (prune loop); `site/scripts/console.sh` failure message.

## Site build and deploy

### SITE-075 Every Cloudflare build waits for its own commit's console

A Cloudflare build of the site (`WORKERS_CI_BRANCH` set) computes its commit's key and downloads that
archive. It retries every 20 seconds for up to 15 minutes. Production takes only `main`'s archive,
and a preview takes either name. If the console never arrives, the build fails and names the key,
so nothing is deployed. The site never deploys a console other than the one its commit builds.

Reason: two builds start at once on a merge, and without the wait the site could ship the previous
console without anything reporting it.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md), [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `site/scripts/console.sh`; test `site/scripts/tests/console.sh` (production and preview waits, 44 sleeps, failure names the key).

### SITE-076 Dependabot and local builds take the rolling console

A `dependabot/` branch's build, which cannot publish, takes the rolling console. So does a local
build.

Reason: Dependabot's pushes get a read-only token.
Sources: [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md), [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md).
Evidence: `site/scripts/console.sh`; test `site/scripts/tests/console.sh` "A local build, and a Dependabot branch's build, take the rolling console".

### SITE-079 No deploy hook, and GitHub holds no Cloudflare credential

The site needs no secret, and GitHub holds no Cloudflare credential of any kind. If a site build
fails because the console build failed or ran late, it is retried from Cloudflare's dashboard.

Reason: a hook works only while a secret nobody is reminded of stays set, and a missing secret
fails silently.
Not: a deploy hook.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md).
Evidence: `.github/workflows/` (no Cloudflare secret).

### SITE-080 Cloudflare builds the site on pushes that change the console

Cloudflare's build watch paths include the console's paths, or are left unset.

Reason: SITE-075's wait can deploy a console change only if a console push starts a site build.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md) (Consequences).
Evidence: Cloudflare dashboard (outside the repository).

### SITE-081 The site build never compiles the console

The site build takes a console that was built separately. It never compiles one.

Reason: the site build and the console build are separate, and Cloudflare builds only the site.
Not: building the console inside the site build.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md).
Evidence: `site/scripts/console.sh`.

## Proposed, built

This document owns no proposed ADR. The `[cell]` keys SITE-052 sets come from proposed
[ADR 0079](../adr/0079-source-cells-are-configurable-rectangles.md). [console.md](console.md) lists
them under its own "Proposed, built".

## Accepted but not built, or built differently

- SITE-043: `synth/web/synth-host.js` drops messages that arrived while the audio context was
  starting and are more than 500 ms past their planned instant. The rule says they sound at once.
  The drop is described only in [orcvs-site/16](../../.scratch/orcvs-site/issues/16-wake-the-synth-before-the-first-note.md)
  (resolved). No tracking issue found.
- SITE-074: the Console web workflow's concurrency group lets a newer pending run replace a pending
  run of the same archive, so not every pushed run finishes. The archive is still published. No
  tracking issue found.
- SITE-080: the build watch paths live in Cloudflare's dashboard and cannot be checked from the
  repository; [orcvs-site/26](../../.scratch/orcvs-site/issues/26-build-the-site-on-console-pushes.md)
  (ready-for-human).

## Open decisions

None of these is binding. Each is built as described but rests only on an undated `.scratch/` spec
decision or an implementation choice, so it is decided through the ADR process.

- SITE-022 detail: the limit is 16 KiB of text after `source=`, checked before decoding. A link that
  names an unknown example, carries unreadable text, or names both `?example=` and `#source=` opens
  the empty Source with a notice. Sources: `.scratch/orcvs-site/spec.md` (Loading a Source) and
  `.scratch/orcvs-site/issues/01-open-a-source-from-the-url-stopped.md` (the value and the
  both-parameters refusal were chosen in the ticket).
- SITE-023 (proposed): a Linked console, and the small console, register no service worker, while
  `/play/` keeps it, scoped to `/play/`. Source: `.scratch/orcvs-site/spec.md` (Service worker).
- SITE-082 (proposed): a deploy replaces the console a returning reader's service worker has cached.
  Sources: `.scratch/orcvs-site/spec.md` (Service worker) and
  `.scratch/orcvs-site/issues/05-deploy-to-cloudflare-with-previews.md`, whose preview-URL item is
  still ready-for-human.
