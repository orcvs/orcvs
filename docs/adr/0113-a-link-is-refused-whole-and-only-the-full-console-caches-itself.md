# A link is refused whole, and only the full console caches itself

Status: accepted by Toby on 2026-11-16. Drafted from the built console and site, to record what
the undated `.scratch/orcvs-site/spec.md` ("Loading a Source", "Service worker") and the choices
made in `.scratch/orcvs-site/issues/01-open-a-source-from-the-url-stopped.md` and
`.scratch/orcvs-site/issues/05-deploy-to-cloudflare-with-previews.md` decided and no ADR states.
Toby's acceptance changed two things the code does not yet do: the service worker follows the same
reading of a link as `opening`, and every `/play/` file reaches a returning reader on their next
load.

[ADR 0094](0094-a-link-to-the-site-opens-what-it-opens-on-the-console.md) and
[ADR 0097](0097-copy-link-gives-the-site-link-to-the-running-source.md) say what a link opens and
that a copied link and a sent Source share the link's limit (SITE-022). None says what the limit is,
what a link the console cannot open does, or which consoles keep a service worker and how a deploy
reaches a reader who has one. `docs/requirements/site.md` lists these as built, awaiting a record
([requirements-snapshot/07](../../.scratch/requirements-snapshot/issues/07-ground-the-site-details-from-undated-spec-text.md)).

## Decision

**A link's Source text is at most 16 KiB, counted before decoding.**
`SOURCE_TEXT_LIMIT` in `console/src/linked_source.rs` is 16 × 1024 bytes of the percent-encoded
`source=` value in the fragment. `source_from` refuses longer text without decoding it. Ticket 01
chose the value: it holds 21 full Grid rows with every byte percent-encoded. The same constant
bounds a Source sent by message (`source_message`, on its raw text) and a copied link (`link`, on
its encoded text), as SITE-022 states.

**A link the console cannot open opens the empty Source with a notice saying why.** `opening`
answers `Opening::Linked(Err(..))`, and `Console::new` raises the message as a notice and opens the
empty Source, Linked and stopped. Each refusal is whole; nothing of a refused link opens. A link is
refused when:

- `?example=` names no shipped example;
- `#source=` text is over the limit, has a `%` not followed by two hex digits, or is not a Source
  File;
- it names both `?example=` and `#source=`. Ticket 01 chose to refuse rather than pick one, so an
  ambiguous link never opens something its author did not mean.

`?small` is read first: a URL naming it opens the Small Console whatever else it names, and a
Source sent to the Small Console that the reader refuses leaves the open Source in place (SITE-013).

**Only `/play/` with no link registers a service worker, and its scope is `/play/`.** A URL is a
link exactly when `opening` in `console/src/linked_source.rs` reads it as one, and the inline script
in `console/index.html` registers `sw.js` exactly when it is not and does not name `small`, so
neither a Linked Console nor the Small Console caches the site's console. It also skips registration on a `#dev` fragment. `register('sw.js')` names no scope, so the
worker takes the default: the directory it is served from, `/play/`. `/` frames `/play/` with its own
query and fragment (SITE-030), so `/` with no link holds a console that registers it.

**A deploy reaches a returning reader on their next load, with fixed file names.**
`console/Trunk.toml` turns file hashing off. `console/assets/sw.js` answers every `/play/` request
from the network first, caching each good response and falling back to the cache only offline. `site/public/_headers` sets `Cache-Control: no-cache` on every
`/play/` file, so each is revalidated. Ticket 05 records that this answers its choice between file
hashing and a versioned worker with neither, since either would repeat that guarantee.

## Considered options

- **Pick one parameter when a link names both.** Ticket 01 left the choice open and chose refusal,
  for the reason above.
- **Narrow the deploy guarantee to the page, `console.js` and `console_bg.wasm`.** Toby rejected it
  on 2026-11-16: the guarantee is only useful whole, and network-first costs little on a few small
  files.
- **Record both readings of a link.** Rejected on 2026-11-16: one definition of a Linked Console,
  not two.
- **File hashing, or a versioned worker whose new cache name evicts the old.** Ticket 05 rejected
  both as repeating what the network-first worker and `_headers` already guarantee.

## Consequences

- `docs/requirements/site.md` states these in SITE-020, SITE-022, SITE-082 and SITE-083.
- Built differently when accepted, each tracked as a ticket:
  - `console/index.html` and `opening` decide "Linked" separately and differ at the edges.
    `?example` with no `=` registers no worker, yet `opening` opens the stored Source and autosaves.
    A fragment with `source=` after another pair, such as `#a&source=..`, opens Linked, yet
    registers the worker. No test covers either
    ([orcvs-site/31](../../.scratch/orcvs-site/issues/31-register-the-service-worker-by-the-consoles-own-reading-of-a-link.md)).
  - `sw.js` answers only navigations, `console.js` and `console_bg.wasm` from the network first,
    and serves every other file it has cached, `logo-splash.png` among them, from its cache first,
    so a deploy that changes such a file reaches a returning reader only when `sw.js` itself
    changes ([orcvs-site/32](../../.scratch/orcvs-site/issues/32-answer-every-play-file-from-the-network-first.md)).
- A reader offline still gets the last `/play/` console their worker cached.
- Tests: `console/src/linked_source.rs` `text_over_the_limit_is_refused_before_decoding`,
  `an_unknown_example_name_is_refused`, `malformed_percent_encoding_is_refused`,
  `text_that_is_not_a_source_file_is_refused`, `an_example_and_source_text_together_are_refused`;
  `site/tests/smoke.spec.ts` "/play/ starts the console and registers its service worker" and the
  link forms that register none; `site/tests/deploy.spec.ts` "a second deploy replaces the console
  a returning reader cached".

## Where this differs from the sources

- The spec's "Service worker" and ticket 05's opening text still say a deploy is handled by file
  hashing or a versioned worker. The code uses neither, as ticket 05's 2026-10-26 comment records.
- The spec's testing decisions and ticket 01's checklist say the ticket picks which parameter wins
  when a link names both. The code refuses both.
- Nothing asserts the worker's scope; it follows from where `sw.js` is served.
- The `#dev` exception is in no spec.
