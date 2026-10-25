# Theme independence

Status: ready-for-agent

## Problem Statement

Retiring a built-in Theme should mean deleting its definition and its record. Instead it touches about 600 lines in 25 files, because the console's code, tests, benches and records depend on particular Themes. Okabe–Ito was the first built-in, so generic paint, style, contrast, document and console tests build from it and assert on its hex values. Tests about shipped Themes name each built-in in turn. Document and registry tests name `okabe-ito` as their sample parent, and `ticked_console` picks Okabe–Ito because its Portal and Locked fills happen to differ.

The console should ship two built-in Themes, Grayscale Dark and Grayscale Light, which are already each appearance's default and fallback. Okabe–Ito, Orcvs Light, My Grayscale Dark and My Grayscale Light go. That retirement is blocked by the coupling, and every later palette change pays the same cost again.

## Solution

No code, test or bench depends on a particular Theme. The rule is recorded as an ADR, and the console is brought into line with it before the four extra built-ins are retired.

- Shipped code reaches a Theme through the Theme registry: a selection or an appearance's default and fallback. Only the built-in definitions and the registry's list of them name a built-in.
- A test that needs a Theme as a fixture uses the default Theme for its appearance, reached through the registry's fallback. The defaults ship anyway, so a test builds no Theme of its own. Its expected colours come from the Theme it uses, never from hex copied out of a Theme. A test that needs a channel the default leaves transparent, or two channels the default sets alike, overrides only those fields of the default.
- A test about the shipped Themes covers every built-in the registry holds, not a named one.
- A test that needs a parent identity or a saved selection uses the appearance's default identity.

With that in place, the console ships Grayscale Dark and Grayscale Light only.

## User Stories

1. As a maintainer, I want to retire a built-in Theme by deleting its definition and its record, so that a palette decision does not cost a codebase-wide edit.
2. As a maintainer, I want to retune a default Theme's colours without editing generic tests, so that design iteration stays cheap.
3. As a maintainer, I want to add a built-in Theme and have the contrast and colour-vision gates cover it automatically, so that no shipped Theme escapes them.
4. As a maintainer, I want a paint test to state the behaviour it checks through the Theme fields it reads, so that it reads as a statement about painting, not about one palette.
5. As a maintainer, I want a test that needs a visible Cursor fill to override just that field of the default, so that the test shows which input it depends on.
6. As a maintainer, I want a test that needs distinct Portal and Locked fills to set them itself, so that it does not depend on a Theme that happens to have them.
7. As a maintainer, I want document and registry tests to name the appearance's default as their parent, so that they survive any rename or retirement of a built-in.
8. As a maintainer, I want the benches to paint with the dark default, so that a benchmark measures painting rather than one retired palette.
9. As a maintainer, I want the rule written as an ADR, so that the next contributor or agent does not reintroduce the coupling.
10. As a maintainer, I want the four extra built-ins gone from the console, the Theme record, the release captures and the examples, so that what ships matches what is documented.
11. As a viewer, I want the console to offer Grayscale Dark and Grayscale Light as its built-ins, so that the Theme menu shows only the Themes the project supports.
12. As a viewer whose saved selection names a retired built-in, I want the console to fall back to my appearance's default and keep my selection as written, so that it starts normally.
13. As a viewer whose custom Theme inherits from a retired built-in, I want the console to refuse it with a notice naming the setting, so that I know to point it at a default.
14. As a viewer, I want custom Themes to keep starting from a built-in and stating only their differences, so that authoring a Theme is unchanged.
15. As a reviewer, I want each migration ticket to keep CI green on its own, so that the work can land in any order.
16. As a reviewer, I want the retirement in a single final ticket, so that the one change to what ships is reviewed in one place.

## Implementation Decisions

- **The rule is a new ADR** beside ADR 0053, titled along the lines of "Nothing depends on a particular Theme". It states the four bullets in the Solution. ADR 0053's text that names Okabe–Ito or Orcvs Light as the shipped Themes is amended in the retirement ticket. An ADR's historical record is not rewritten.
- **The fixture seam is the registry's fallback.** `ThemeRegistry::built_in().fallback(appearance)`, cloned where a test overrides fields with struct-update syntax, is the existing seam that already answers "the default Theme for this appearance". No new shipped function is added for tests, and no test-only Theme is defined. Paint's own `test_theme` (the dark default with a visible `cursor.background`) is built from it.
- **The shipped-Theme gates iterate the registry's built-ins.** This covers the contrast gate, the colour-vision gate, and the check that each built-in matches its table in the Theme record. Tests that name a built-in, such as `okabe_ito_has_nothing_to_except` or `orcvs_light_keeps_every_okabe_ito_glyph_hue`, go once that coverage holds.
- **A baseline captured from one Theme's bytes goes.** `okabe_ito_reproduces_the_pre_refactor_cell_visuals_exactly` freezes one palette's composed bytes. Its behaviour is covered by tests that read the Theme's fields, so it is deleted rather than recaptured against a default.
- **Parents and selections name the default identity.** Document, registry, selection, config and storage tests use `ThemeIdentity::default_for(appearance)` for a parent or a saved selection. A test about a wrong-appearance selection uses the other appearance's default.
- **The release-capture matrix covers the two defaults.** The capture workflow and its `mise` task capture each viewport once in Grayscale Dark and once in Grayscale Light.
- **Retirement deletes outright.** The four built-in literals, their reserved identities, their Theme record sections and evidence, the Okabe–Ito copy example and their record tests go. The registry holds two built-ins. A saved selection or a custom Theme's `inherits` naming a retired identity follows the existing missing-identity paths: fallback for a selection, refusal with a notice for a document. The Orcvs language and its Theme format are pre-release, so no migration is offered.
- **Expand, migrate, contract.** The migration tickets stay green on their own because the extra built-ins still exist until the final ticket.

## Testing Decisions

- A good test states one console behaviour through the Theme fields it reads. It asserts that a Cell's border is `theme.source_function_border`, not `#444444`. Changing a default's colours breaks no generic test; changing behaviour does.
- Where a default sets a channel transparent, or sets two channels alike that a test must tell apart, the test overrides those fields on the default and asserts on its own values. Grayscale Dark's identical Output and Input Portal fills are the known case.
- The shipped-Theme gates run over every built-in the registry holds, so the retirement ticket removes Themes without editing a gate.
- Modules touched: style, paint, Overview, contrast and colour vision, Theme document, Theme registry, Theme selection, config, storage, and the console's kittest and unit tests. The paint and Overview benches change too.
- Prior art: paint's `test_theme` already overrides one field of a built-in for Cursor precedence, and `style`'s tests already use `..okabe_ito()` struct update. The migration keeps that idiom with the default in place of Okabe–Ito.

## Out of Scope

- Changing the grayscale defaults' values. The open choices in `role-borders/03` and `cursor-foreground/03` stay with those tickets, and the latter shrinks to the two remaining built-ins.
- Shipping a colour palette as a custom Theme example in place of Okabe–Ito.
- Any change to the Theme document format, inheritance, or the selection and fallback rules.
- A lint or grep gate that enforces the rule mechanically.

## Further Notes

- The local branch `claude/grayscale-themes-7111p9` (PR #256) carries the grayscale defaults; this work stacks on it.
- `colour_vision`'s gate exists to keep hued roles apart under protanopia and deuteranopia. With two grayscale built-ins, only the Bang and Diagnostic carry hue. The gate keeps running over whatever the registry holds; whether its thresholds still mean something for grayscale is a question for the contrast ticket to record, not to resolve by deleting the gate.
