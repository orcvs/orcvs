# Site requirements

This document owns the web console's life on a page: opening a Source by link or by message, the
messages between a console and the page that frames it, the page's audio and the timing of its
Built-in Synth, the Sound Module, the web settings
element, Copy Link, and which console the site deploys. The Built-in
Synth's voices, effects, listing order and automatic selection belong to [playback.md](playback.md).
Settings keys, the top bar and the Panel belong to [console.md](console.md). The Source File format
belongs to [language.md](language.md), and Ticks to [evaluation.md](evaluation.md). Terms are defined in
[CONTEXT.md](../../CONTEXT.md), and decisions are argued in [the ADRs](../adr/).

A Linked Console and the Small Console are as `CONTEXT.md` defines them. Here a Linked Console is
one whose address uses `?example=`, `#source=` or `?small`, and the Small Console opens at
`/play/?small`.

> Snapshot: incorporates ADRs through 0110 and ADR changes dated through 2026-11-15.
> A newer ADR binds where it differs until this document is updated; the
> [reading order](README.md#reading-order) says which ADRs are newer.
> Source ADRs: 0054, 0086, 0087, 0088, 0089, 0091, 0093, 0094, 0097, 0099, 0101, 0102.

## Opening a Source by link

### SITE-020 A console's URL opens a shipped example or Source File text, read once at startup

`/play/?example=<link-name>` opens the shipped example with that link name. `/play/#source=<text>`
opens percent-encoded Source File text. In that text `+` stays `+`, and `%` must be followed by two
hex digits. The console reads its URL once, at startup. A URL that names neither form opens the
stored Source.

Reason: a link is how a reader is sent one Source, and `/play/` keeps both forms beside the Small
Console's messages.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md) (Consequences), [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `console/src/linked_source.rs` `opening`, `source_from`, `percent_decoded`; `console/src/console.rs` `Console::start_at`; test `console/src/linked_source.rs` `an_example_name_opens_that_example`.

### SITE-021 A Linked Console opens stopped and never reads or writes saved work

A Linked Console opens with Playback stopped, so a link cannot start Playback by itself. It
neither restores the stored Source nor stores its own, so a link never replaces saved work, and
plain `orcvs.com` opens that work again. Edits in a Linked Console are not saved.

Reason: a link someone sends must not play by itself or overwrite the reader's own Source.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md), [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/console.rs` `Console::new` (`stores_source`), `Console::save`; test `console/src/console/storage_tests.rs` `a_linked_console_neither_restores_nor_stores_the_source`.

### SITE-022 A link's text, a sent Source and a copied link share one byte limit

The `#source=` reader has one byte limit (`console/src/linked_source.rs` `SOURCE_TEXT_LIMIT`).
SITE-013 applies the same limit to a Source sent by message, and SITE-062 applies it to a copied
link.

Reason: one limit means a link the console copies always opens, and a message can carry no more
than a link.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md), [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `source_from`, `source_message`, `link`; test `text_over_the_limit_is_refused_before_decoding` in the same file.

### SITE-024 Link names are the console's, and are public

A shipped example's link name is the one the console gives it. The site keeps no list of examples.
A link name appears in links in the wild, so renaming a shipped example breaks those links.

Reason: the site would otherwise keep a second copy of the example list beside the console's.
Not: a page per example on the site.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `console/src/examples.rs` `Example::link_name`; test `console/src/examples.rs` `each_example_links_by_its_name`.

## The Small Console

### SITE-010 The Small Console opens empty, stopped and keeps nothing

`/play/?small` opens Linked on an empty Source with Playback stopped. It shows the Grid and the
Panel without the menu bar. It has no File menu and cannot save, so its Source lasts as long as
the page.

Reason: a page shows many examples beside one running console, and none of them may touch the
reader's work.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md), [ADR 0089](../adr/0089-the-panel-plays-and-rewinds.md) (every console shows the Panel).
Evidence: `console/src/linked_source.rs` `opening` (`Opening::Small`); `console/src/console.rs` `Console::new`; test `console/src/linked_source.rs` `small_opens_the_small_console`.

### SITE-011 The Small Console shows a notice bar only while it has notices

While the Small Console has notices, it shows a bar holding only those notices.

Reason: a refused Source has to say why the open one stayed.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/console/menu_bar.rs` `show_notice_bar`; test `console/src/console/kittest_tests.rs` `a_small_console_shows_a_refused_source_s_notice`.

### SITE-012 The console opens only `orcvs.open` messages from its framing page and origin

The open message is `{ type: "orcvs.open", source }`. It is posted by the page that frames the
console, from the console's own origin. The console ignores a message from another window or
origin, a message of another type, and a message without a string `source`.

Reason: otherwise any page that frames the console could replace its Source.
Not: accepting messages from any origin.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/web_startup.rs` `listen_to_page`; test `site/tests/reference.spec.ts` "pressing an example opens it in the page's small console".

### SITE-013 A sent Source opens stopped, or is refused with a notice

The console reads `source` as a Source File, the same way it reads `#source=` text and under the
same byte limit (SITE-022). If the reader refuses it, the open Source stays and a notice is
raised. If the reader accepts it, it replaces the open Source with Playback stopped, so a message
cannot start Playback.

Reason: a message is a link without a reload, and keeps a link's guarantees.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `console/src/linked_source.rs` `source_message`; `console/src/console/files.rs` `open_sent_source`; test `console/src/console/tests.rs` `a_small_console_opens_each_message_it_is_sent_stopped`.

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
Evidence: `console/src/web_startup.rs` `listen_to_page`; `site/src/console/small.ts` `start`, `fail`; test `site/tests/reference.spec.ts` "a console that does not start says so, and the next press starts another".

### SITE-016 A page that shows examples holds one Small Console

A page with examples holds one Small Console and sends it each example by message.

Reason: a console per example pays a WASM start for each one.
Not: a console per example, or opening examples in a new tab or window.
Sources: [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: `site/src/console/small.ts`; test `site/tests/lesson.spec.ts` "pressing two Grids opens each in the one small console, stopped".

## The site's pages

### SITE-030 `/` frames `/play/` with its own query and fragment

`/` frames `/play/`, passing on its whole query and fragment, inside the page that holds the
Built-in Synth and the Sound Module. So `orcvs.com/?example=<link-name>` opens what
`/play/?example=<link-name>` opens, and `/?small` opens the Small Console. The address is read
once, when the page loads. Editing the fragment of an open page changes nothing until a reload.

Reason: a link to the site has to name an example and still play through the page's synth.
Not: the console writing its link into the address bar, because a reload would drop the reader's edits.
Sources: [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `site/src/pages/index.astro`; test `site/tests/smoke.spec.ts` "`/${link}` frames `/play/${link}`".

### SITE-031 The home page is the console

`/` is the console filling the window, with the console's own splash. The Sound Module is the row
below it.

Reason: "The site should be the console."
Sources: ruled 2026-10-26, no ADR (`.scratch/site-as-console/spec.md`); [ADR 0094](../adr/0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md).
Evidence: `site/src/pages/index.astro`; test `site/tests/smoke.spec.ts` "/ is the console, filling the window above the Sound Module".

### SITE-032 Pages carry the repository's docs, draw each Source as a Grid, and open no new window

Site pages render the repository's own docs, `docs/first-result.md` and `docs/functions.md`, and add
no prose written for the site. Each Source block is drawn as a Grid of Cells, and pressing it sends
that Source to the page's Small Console (SITE-016). Nothing on the site opens a new tab or window.

Reason: "I do not want your copy", "Function source code references should use a grid", "Launching
examples in a new window is bad".
Sources: ruled 2026-10-26, no ADR (`.scratch/site-as-console/spec.md`).
Evidence: `site/src/console/docs.ts`, `site/src/console/grid.ts`; test `site/tests/lesson.spec.ts` "the lesson is docs/first-result.md, heading for heading".

## The Sound Module

What each effect does to the sound, and how the synth answers each controller, are in
[PLAY-600](playback.md#play-600-each-channel-has-its-own-eight-effects-on-standard-controllers), [PLAY-610](playback.md#play-610-channel-08-plays-one-303-style-voice) and [PLAY-620](playback.md#play-620-reset-all-controllers-clears-only-a-channels-modulation). The controller numbers and load values are in
[docs/functions.md § Built-in Synth](../functions.md#built-in-synth). This section states what the
Sound Module shows and sends.

### SITE-033 The Sound Module is a Grid of Cells on `/` only

The Sound Module is drawn the way the site draws a Source, as Cells in the console's font and the
Grayscale Theme colours. Its top row holds the sixteen channels, each a Cell holding its hex digit
with a spacer Cell after it. A blank row follows. Below it are eight effect columns, each two Cells
wide with two spacer Cells between columns. Each column holds three rows:
- a magnitude: two bordered Cells, filled at the value's fraction of `7F`, with no glyphs
- a value: two hex digits in the Number role's colours
- a name: two Cells in the Function role's colours

The columns run in controller order: `MD`, `EQ`, `LN`, `OF`, `R1`, `R2`, `CH`, `DL`. Hovering a Cell
names its effect and controller. The Sound Module appears on `/` only. A page with the Small Console
plays the effects a Source sets but shows no Sound Module.

Reason: it reads as part of the console rather than a control panel beside it.
Not: page HTML knobs and lights, or a bar of glyphs for the magnitude.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md) and its 2026-11-14 amendment.
Evidence: `site/src/console/sound-module.ts` `soundModule`; `site/src/console/sound-module.css`; test `site/tests/sound-module.spec.ts` "the Sound Module draws its lights and effects at their load values".

### SITE-034 A channel lights for 150 ms when a Note On sounds it, and shows no level

A channel's Cell is dim until a Note On sounds it. It then takes the Function role's colours and
goes dim again 150 ms later. A channel shows no level.

Reason: a light at velocity would read as one of the effects.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md).
Evidence: `site/src/console/sound-module.ts` `hear`, `SOUNDING_MS`; test `site/tests/sound-module.spec.ts` "a Note On from the console shows its channel as sounding, then idle".

### SITE-035 The effects show the chosen channel's values, and follow the stream until the reader chooses

The Sound Module keeps every channel's effect values from the one stream of `orcvs.midi` messages it
hears on its page, whoever sent them. Its effect columns show the chosen channel's values.
- Channel `00` is chosen at load.
- Until the reader chooses a channel, the chosen channel follows the channel of each effect Control
  Change heard, so a Source's effects show as it plays.
- Reset All Controllers changes no choice.
- The reader chooses a channel by clicking its digit, or by pressing Enter or Space on a digit
  reached with Tab. The arrow keys step from channel to channel.
- The chosen channel is outlined in the magnitude colour.
- Choosing a channel shows the values that channel was last sent.

Reason: each channel has its own effects, and a musician's other gear never lets one channel's
controller change another's.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md) (2026-11-14 amendment).
Evidence: `site/src/console/sound-module.ts` `choose`, `pick`, `hear`; test `site/tests/sound-module.spec.ts` "a Control Change from the console shows as its effect's value on its channel".

### SITE-036 The reader types values as on the Grid, and each change is sent as a Control Change on the chosen channel

Clicking a value's digit puts the Cursor there.
- A hex digit overwrites the digit under the Cursor, and the Cursor steps east.
- The arrow keys move the Cursor across the values.
- Escape gives the keys back to the page.
- A digit that would take the value past `7F` is refused.
- No drag or scroll changes a value.

Each change is sent as a Control Change on the chosen channel. It goes as an `orcvs.midi` message on
the Sound Module's own page, the same message a console posts, so the synth and the Sound Module
hear one stream.

Reason: typing matches the Grid and leaves dragging and scrolling to the page.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md) and its 2026-11-14 amendment.
Evidence: `site/src/console/sound-module.ts` keydown handler, `send`, `place`; test `site/tests/sound-module.spec.ts` "typing on a value writes it as the Grid does, and sends its Control Change".

### SITE-037 Values reset on every load, and the acid channel shows the acid voice's controls

Nothing the Sound Module shows persists across page loads. Every channel starts at its effects' load
values, and a Source that needs an effect sets it.

On the acid channel `08`, the second and third columns are the acid voice's cutoff, `CU`, and decay,
`DC`, which load at `30`. On every channel, Reset All Controllers shows `MD` as `00` and leaves the
other columns, `CU` and `DC` included, as they are, matching what the synth does.

Reason: a Source that sounded different on a second visit could not be shared.
Not: keeping the reader's changes across loads.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md) and its 2026-11-14 amendment; [ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md).
Evidence: `site/src/console/sound-module.ts` `EFFECTS`, `loads`, `labelOf`; test `site/tests/sound-module.spec.ts` "the acid voice's channel shows its cutoff and decay where EQ and LN stand".

### SITE-038 The Sound Module takes one Theme colour, `sound.magnitude`

Each magnitude's border and fill, and the chosen channel's outline, use `sound.magnitude`. Each
built-in Theme sets it to its `source.bang` colour. The site carries it as
`--orcvs-sound-magnitude`. The console's Theme format gains the key when the console draws the
Sound Module.

Reason: the magnitude colour can then differ from Bang without changing Bang.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md).
Evidence: `site/src/styles/orcvs.css` `--orcvs-sound-magnitude`; `site/src/console/sound-module.css`.

## The Built-in Synth on a page

### SITE-040 A page announces its synth with `orcvs.synth` after the console's `orcvs.ready`

A page that holds the Built-in Synth answers a console's `orcvs.ready` with
`{ type: "orcvs.synth", available }`. Listing and choosing the synth as a destination are stated in
[PLAY-520](playback.md#play-520-the-built-in-synth-is-a-destination-listed-after-every-midi-port).

Reason: the console has no audio code. It learns from the page that a synth is there.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` (ready handler, `tell`); `console/src/built_in_synth.rs` `receive`; test `console/src/built_in_synth.rs` `the_synth_is_listed_after_the_ports_only_while_a_page_announces_it`.

### SITE-041 A Linked Console picks the Built-in Synth once it is announced

When the page announces the synth, a Linked Console, the Small Console included, picks it as its
destination even if a MIDI port exists.

Reason: an example someone opens from a link or a page should be heard without any setup.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/built_in_synth.rs` `SynthAnnouncements::observe`; test `a_linked_console_picks_the_synth_over_a_port` in the same file.

### SITE-042 Each message goes to the page as `orcvs.midi` stamped with its Tick's planned instant

While the synth is chosen, the console posts each message Playback hands the destination as
`{ type: "orcvs.midi", bytes, time }`. It posts only to the framing page, with the console's own
origin as the target. `time` is the Tick's planned instant on the clock both frames share,
`performance.timeOrigin` plus `performance.now()`, not the moment the message is sent. Stop's safety
action reaches the page the same way as any other message.

Reason: a late timer or a slow frame must not move a note.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/built_in_synth.rs` `BrowserPage::post`; test `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks".

### SITE-043 The page plays a message a fixed delay after its planned instant

The console recovers `time` from how late Playback says each message's Tick is (PLAY-540). The
page plays each `orcvs.midi` a fixed delay after its `time`. The delay is set within a 50 ms
target. A message whose time has passed by more than the delay sounds at once. The page passes
each message on to the synth core, which [PLAY-550](playback.md#play-550-one-rust-synth-core-renders-the-built-in-synth) describes.

Built differently; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: no note moves until lateness exceeds the delay.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` `DELAY_MS`, `contextTime`; test `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks".

### SITE-044 The reader's gesture resumes the page's audio

The page resumes its audio context inside each pointer, touch or key gesture, whether the gesture
is on the page or in a console's frame. When it does, it also asks for media playback, so iOS's
ring/silent switch does not mute it. A framed console posts `{ type: "orcvs.wake" }` to its page on
each click and key. If no gesture has resumed the context, the first `orcvs.midi` of a run resumes
it.

Reason: Safari resumes audio only inside a gesture, and the console's first message arrives after
the gesture that caused it.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md) as amended by [ADR 0089](../adr/0089-the-panel-plays-and-rewinds.md).
Evidence: `synth/web/synth-host.js` `resume`, `resumeOnGestures`, `start`; `console/src/web_startup.rs` `wake_page_audio`; test `site/tests/synth.spec.ts` "a click in the console starts the page's audio before Space".

### SITE-045 A start on the synth waits up to 500 ms for the page's audio

The page tells every console `{ type: "orcvs.audio", running }` whenever its context starts or
stops. A Playback start on the Built-in Synth while the audio is not running is held until the
audio runs, for at most 500 ms.

Reason: starting audio takes a device over 100 ms. Without the hold, Tick 0 would sound late and the
Ticks after it on time.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `console/src/console/playback.rs` `PlaybackStart`, `AUDIO_WAIT`; `console/src/built_in_synth.rs` `SynthAnnouncements::start_is_early`; test `console/src/console/playback.rs` `a_held_start_is_granted_once_it_has_waited_long_enough`.

### SITE-046 A page that cannot make sound still announces the synth, unavailable

A page without audio announces `available: false` with a reason. The console still lists the
synth. Choosing it answers the reason as a connect failure, and Playback runs on silently.

Reason: an unavailable synth should explain itself, the way a MIDI device that has gone does.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/web/synth-host.js` `announcement`; `console/src/built_in_synth.rs` `WithBuiltInSynth::connect`; test `an_unavailable_synth_answers_its_reason_on_connect` in the same file.

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
[CONS-131](console.md#cons-131-the-settings-open-views-and-time-toasts).

Reason: the page that runs the console is what a deployment controls, and settings are needed
before the first frame.
Not: URL parameters per setting, a layout chosen inside the console by viewport, or settings sent
by message.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md).
Evidence: `console/src/web_startup.rs` `page_settings`; `console/src/config.rs` `Config::start`, `Config::from_page`; test `console/src/config.rs` `the_page_settings_are_read_as_the_file_is_and_named_in_a_notice`.

### SITE-051 Problems in the page's settings are reported in the settings notice, naming the page

A key the page's element cannot supply is reported through the settings notice. The notice names
the page's `orcvs_config` element rather than a file.

Reason: the reader has to know where the bad value came from.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md) (Consequences).
Evidence: `console/src/config.rs` `Config::from_page`; test `the_page_settings_are_read_as_the_file_is_and_named_in_a_notice` in the same file.

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
Evidence: `console/src/console/menu_bar.rs` `share_control`; `console/src/console/files.rs` `copy_link`; test `console/src/console/kittest_tests.rs` `copy_link_puts_the_link_to_the_running_source_on_the_clipboard`.

### SITE-061 A copied link names orcvs.com, by link name or as encoded text

A Source that is a shipped example, unchanged, links by its link name:
`https://orcvs.com/?example=<link-name>`. Any other Source links as its Source File text,
`https://orcvs.com/#source=…`, with every byte percent-encoded except letters, digits and `-._~`.
An edited example therefore links as text. The link always names `https://orcvs.com/`, the page
with the Built-in Synth, whichever page or build the console runs in, so a preview's console copies
a production link.

Reason: the encoded text survives any address bar or chat client, and the link always plays.
Sources: [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `link`, `percent_encoded`, `SITE`; test `an_unchanged_example_links_by_its_name` in the same file.

### SITE-062 A Source too large to link is refused with a notice

If a Source's encoded text is over the byte limit (SITE-022), it is refused with a notice rather
than copied as a link that would not open.

Reason: a link that would not open is worse than none.
Sources: [ADR 0097](../adr/0097-copy-link-gives-the-site-link-to-the-running-source.md).
Evidence: `console/src/linked_source.rs` `link`; `console/src/console/files.rs` `copy_link`; test `console/src/linked_source.rs` `a_source_whose_link_is_over_the_limit_is_refused`.

## Which console the site deploys

The chain that keys, builds, publishes and fetches the web console is engineering tooling:
[`docs/tooling.md`](../tooling.md) states it, and `.github/workflows/console-web.yml` and
`site/scripts/console.sh` implement it. The rules a reader of the site can observe are these two.

### SITE-075 Every site build deploys the console its own commit builds

A Cloudflare build of the site, production or preview, takes the console keyed to its own commit.
Production takes only `main`'s archive. If that console never arrives, the build fails and names the
key, so nothing is deployed. A local build, and a `dependabot/` branch's build, which cannot publish,
take `main`'s current console instead.

Reason: two builds start at once on a merge, and without the wait the site could ship the previous
console without anything reporting it.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md), [ADR 0101](../adr/0101-every-site-build-takes-the-console-its-commit-builds.md).
Evidence: `site/scripts/console.sh`; test `site/scripts/tests/console.sh`.

### SITE-081 The site build never compiles the console

The site build takes a console that was built separately. It never compiles one.

Reason: the site build and the console build are separate, and Cloudflare builds only the site.
Not: building the console inside the site build.
Sources: [ADR 0093](../adr/0093-the-site-deploys-only-the-console-its-commit-builds.md).
Evidence: `site/scripts/console.sh`.

## Accepted but not built, or built differently

- SITE-043: `synth/web/synth-host.js` drops messages that arrived while the audio context was
  starting and are more than 500 ms past their planned instant. The rule says they sound at once.
  The drop is described only in [orcvs-site/16](../../.scratch/orcvs-site/issues/16-wake-the-synth-before-the-first-note.md)
  (resolved). Tracked in [requirements-snapshot/12](../../.scratch/requirements-snapshot/issues/12-decide-late-messages-during-audio-start.md).

## Open decisions

### Built, awaiting a record

Each is built as described but rests only on an undated `.scratch/` spec decision or an
implementation choice ([decisions D3](../../.scratch/requirements-snapshot/decisions.md#d3-rulings-recorded-only-in-scratch--proposed)). Preserve each as built until an ADR or a dated ruling records it or
changes it. Tracked in
[requirements-snapshot/07](../../.scratch/requirements-snapshot/issues/07-ground-the-site-details-from-undated-spec-text.md).
All three are proposed for the record in
[ADR 0113](../adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md), not yet
accepted.

- The link limit (SITE-022) is 16 KiB of text after `source=`, checked before decoding. A link that
  names an unknown example, carries unreadable text, or names both `?example=` and `#source=` opens
  the empty Source with a notice. Sources: `.scratch/orcvs-site/spec.md` (Loading a Source) and
  `.scratch/orcvs-site/issues/01-open-a-source-from-the-url-stopped.md` (the value and the
  both-parameters refusal were chosen in the ticket).
- Service workers: a Linked Console, and the Small Console, register no service worker, while
  `/play/` keeps it, scoped to `/play/`. Source: `.scratch/orcvs-site/spec.md` (Service worker).
- A deploy replaces the console a returning reader's service worker has cached. Sources:
  `.scratch/orcvs-site/spec.md` (Service worker) and
  `.scratch/orcvs-site/issues/05-deploy-to-cloudflare-with-previews.md`, whose preview-URL item is
  still ready-for-human.
