# A link to the site opens what it opens on the console

Status: accepted. Chosen by Toby on 2026-10-29. Settled in `.scratch/orcvs-site/issues/27-link-to-an-example-from-the-site.md`.

A reader wants to send someone a link to one example, such as the Orca Tracker. The console
already opens a link: `/play/?example=<link-name>` opens a shipped example, and
`/play/#source=<text>` opens Source File text (ADR 0086). But `/play/` on its own holds no Built-in
Synth, because the page that frames it holds the synth (ADR 0087), and `/` framed `/play/` without
its address, so a link to `/` could not name an example.

## Decision

**`/` frames `/play/` with its own query and fragment.** `orcvs.com/?example=orca-tracker` opens
the console that `orcvs.com/play/?example=orca-tracker` opens, inside the page that holds the
Built-in Synth and the Sound Module. The link names are the console's (`console/src/examples.rs`),
so the site keeps no list of examples.

A linked console opens stopped and neither reads nor writes the reader's saved Source, so a link
never replaces saved work: plain `orcvs.com` opens it again.

## Considered options

**A page per example, such as `orcvs.com/examples/orca-tracker/`.** Tidier links, but the site
would keep its own copy of the example list beside the console's.

**The console writes the link into the address bar when Help → Examples opens an example.** The
console would have to tell its page by message, and a reload of that address would open the example
fresh and drop the reader's edits.

## Consequences

- An example's link name is now part of links in the wild: renaming a shipped example breaks them.
- Edits in a linked console are not saved, as in any linked console.
- `/` passes its whole query and fragment on, so `/?small` opens the small console on `/`.
- The address is read once, when the page loads: editing the fragment of an open `/` changes
  nothing until a reload, as on `/play/`.
