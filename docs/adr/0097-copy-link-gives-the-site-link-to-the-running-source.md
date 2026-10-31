# Copy Link gives the site link to the running Source

Status: accepted. Settled in `.scratch/orcvs-site/issues/28-copy-a-link-to-the-running-source.md`.

A site link opens an example or Source File text in the console (ADR 0094), but nothing in the
console shows that link, so a reader has no way to get one to send.

## Decision

**`File → Copy Link` puts the site link that opens the running Source on the clipboard**, on native
and on the web. A Source that is a shipped example unchanged links by its link name,
`https://orcvs.com/?example=tracker`; any other Source links as its percent-encoded Source File
text, `https://orcvs.com/#source=…`. Every byte but a letter, a digit or one of `-._~` is encoded,
so the text survives any address bar or chat client. A Source whose encoded text is over the limit
a linked console reads is refused with a notice rather than copied as a link that would not open.

The link always names `https://orcvs.com/`, the page that frames the console with the Built-in
Synth, whichever page or build the console runs in.

## Considered options

**A share control on the site, outside the console.** The site would have to ask the console what
it is running, by message, and native would have no way to share.

**The console writes the link into the address bar.** ADR 0094 rejected it: a reload of that
address drops the reader's edits.

## Consequences

- An edited example links as text, so the link opens the edits, not the shipped example.
- A preview deployment's console copies a link to the production site.
