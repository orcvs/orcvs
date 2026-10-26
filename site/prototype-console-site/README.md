# PROTOTYPE: the site as the console

Throwaway. Answers: does the site work as the console page itself, with the sound module beneath it,
and do Function examples read as Grids with a small console fed Source by message?

Run: copy a web console build into `play/` (`mise run console_web_build`, then
`cp -R target/console-web site/prototype-console-site/play`), then `python3 -m http.server` here.

- `index.html`: the shipped console page, with its own menubar and splash, above a hypothetical
  sound module strip (ticket 07).
- `reference.html`: entries from `docs/functions.md` as Grids of Cells in the Grayscale Themes,
  with a small console per example. Sending Source by message is a stand-in: it reloads the
  frame with `#source=`.
