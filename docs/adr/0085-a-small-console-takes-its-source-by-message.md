# A small console takes its Source by message

Status: accepted. Settled in `.scratch/site-as-console/`.

The site shows Function examples beside the running console. Opening each example through a URL
(`?example=` or `#source=`, `console/src/linked_source.rs`) reloads the console and its WASM for
every example, and a link that opens a new tab takes the reader off the page.

## Decision

**A page that shows examples holds one small console, and sends it each example by message.** The
small console is `/play/?small`. It opens Linked, so it neither restores nor stores a Source, on
an empty Source with Playback stopped. It shows the Grid and the Panel without the menu bar.

**The message is `{ type: "orcvs.open", source }`, posted from the console's own origin.**
`source` is Source File text, read by the same reader as `#source=` and under the same byte limit.
A message from another origin, of another type, or without a string `source` is ignored. A Source
the reader refuses leaves the open Source in place and raises a notice. An accepted Source
replaces the open one with Playback stopped, so a message cannot play by itself.

## Rejected alternatives

**A console per example, each opened through its URL.** Every example pays a WASM start, and a
page with many examples starts many consoles.

**Opening examples in a new tab or window.** It takes the reader away from what they were
reading.

**Accepting messages from any origin.** Any page that frames the console could replace its Source.

## Consequences

- `#source=` and `?example=` stay: they are how a link opens a Source, and `/play/` keeps them.
- The small console has no File menu, so it cannot save. Its Source lasts as long as the page.
- The page that frames the small console must wait for it to start before posting, or the message
  is lost: the console listens only once its WASM runs.
