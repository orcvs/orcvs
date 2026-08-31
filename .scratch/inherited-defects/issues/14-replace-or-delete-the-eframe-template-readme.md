# 14 — Replace or delete the eframe template README

**What to fix:** `README_EFRAME.md` gives instructions for renaming a fresh copy of the `eframe`
template. It names files and identifiers that this repository does not have.

**Status:** resolved

- [x] No document tells a reader to edit a file that does not exist.
- [x] Any kept content describes this repository.

## Comments

`shell/README_EFRAME.md` is the upstream template README, kept without edits.

Line 14 tells the reader to click "Use this template". Line 18 tells them to rename `package.name`
from `eframe_template`. Line 21 names `eframe_template::TemplateApp`, a type this project does not
have. Lines 25 and 26 name `./eframe_template.js` and `./eframe_template_bg.wasm` in the
`filesToCache` array. Issue 06 covers that array; the names there are `shell.js` and
`shell_bg.wasm`, and both are already wrong for a different reason.

Lines 3, 4, 78, and 82 link to the upstream repository and its demonstration page.

A reader who follows these instructions edits files that are not here.

Delete the file. The upstream README is available upstream, and its setup steps were completed once,
long ago.

Keep the file only if it holds guidance that is still true and is written nowhere else, such as the
notes on web deployment. Then rewrite those parts for this repository and delete everything else.
