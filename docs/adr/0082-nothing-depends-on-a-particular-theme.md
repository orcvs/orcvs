# Nothing depends on a particular Theme

Status: accepted. Builds on [ADR 0053](0053-one-theme-styles-the-whole-console.md), whose Theme model, registry, selection and fallback rules stand. Settled in `.scratch/theme-independence/`.

Retiring a built-in Theme should mean deleting its definition and its record. When the console's code, tests, benches and records depend on particular Themes, a retirement touches hundreds of lines across dozens of files instead, and every later palette change pays the same cost. Okabe–Ito was the first built-in, so generic paint, style, contrast, document and console tests built from it and asserted on its hex values; tests about shipped Themes named each built-in in turn; document and registry tests named `okabe-ito` as their sample parent; and the console's kittest fixture chose Okabe–Ito because its Portal and Locked fills happened to differ.

## Decision

**No code, test or bench depends on a particular Theme.**

- **Shipped code reaches a Theme through the Theme registry**: a selection, or an appearance's default and fallback. Only the built-in definitions and the registry's list of them name a built-in.
- **A test that needs a Theme as a fixture uses the default Theme for its appearance**, reached through the registry's fallback, `ThemeRegistry::built_in().fallback(appearance)`. The defaults ship anyway, so a test defines no Theme of its own. Its expected colours come from the Theme it uses, never from hex copied out of a Theme. A test that needs a channel the default leaves transparent, or two channels the default sets alike, overrides only those fields of the default with struct-update syntax, and says why.
- **A test about the shipped Themes covers every built-in the registry holds**, not a named one. The contrast gate, the colour-vision gate and the check of each built-in against its table in `console/src/theme.md` iterate the registry's built-ins, so adding a built-in brings it under them and removing one edits no gate.
- **A test that needs a parent identity or a saved selection uses the appearance's default identity**, `ThemeIdentity::default_for(appearance)`. A test about a wrong-appearance selection uses the other appearance's default.

## Rejected alternatives

**A test-only Theme.** A fixture Theme defined beside the tests would free them from the shipped palettes, but it is one more Theme to keep complete as fields are added, and it tests painting against colours no viewer sees. The defaults are complete by construction and ship anyway.

**Recapturing byte baselines against the default.** A test that freezes one Theme's composed bytes breaks on every retune of that Theme without saying which behaviour changed. Tests that read the Theme's fields state the behaviour instead.

**A lint or grep gate.** A mechanical check could forbid built-in names outside the definitions, but the names a test may still use (the registry's fallback, the default identity) and the ones it may not are told apart by role, not spelling. The rule is recorded here instead.

## Consequences

- Retiring a built-in deletes its definition, its reserved identity, its registry entry and its record, and nothing else.
- Retuning a default's colours breaks no generic test; changing behaviour does.
- A saved selection or a custom Theme's `inherits` naming a retired identity follows the existing missing-identity paths: fallback for a selection, refusal with a notice for a document.
