# 12 — Declare the page language in index.html

**What to fix:** The root `<html>` element has no `lang` attribute. A screen reader then guesses the
language and can use the wrong pronunciation rules.

**Status:** ready-for-agent
**Implementation:** complete

- [x] The `<html>` element declares a language.
- [x] The declared language matches the page content.

## Comments

`shell/index.html:2` is `<html>` with no attributes.

Add `lang="en"`, or a more specific tag such as `lang="en-GB"` if you prefer one.

The same line has a second, smaller fault. The `<meta http-equiv="Content-Type">` element on line 3
sits before `<head>`, not inside it. Browsers recover from this, so it changes nothing that a user
sees. Correct it while you are in the file.
