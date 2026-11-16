# 13 — Ground the native document model and per-key settings fallback

**Status:** resolved

**What to decide:** the console builds a document model and a per-key settings fallback that rest
only on the undated `.scratch/menu-structure/spec.md`, with no recorded maintainer ruling
(decisions D3): the window title names the open Source File and marks unsaved changes; New, Open,
the Function reference, an example, Quit and closing the window ask before discarding unsaved
changes; autosave is session recovery and Save writes the file; a faulty settings key raises a
notice and only that key takes its default. `docs/requirements/console.md` lists it under "Open
decisions" as built, awaiting a record.

**Done when:** an ADR or a dated ruling records the model, and console.md states it as a rule.

## Comments

2026-11-15: proposed as
[ADR 0111](../../../docs/adr/0111-the-native-console-edits-one-source-file-and-autosave-only-recovers.md).
Waiting to be accepted or changed; its open questions list where the code and the spec differ.

2026-11-16: accepted [ADR 0111](../../../docs/adr/0111-the-native-console-edits-one-source-file-and-autosave-only-recovers.md), deciding that ⌘Q asks and that Help → Function Reference opens the site's page with the runnable Grid as an example. CONS-133 to CONS-137 state it; the two gaps are [menu-structure/10](../../menu-structure/issues/10-ask-before-cmd-q-discards-unsaved-changes.md) and [menu-structure/11](../../menu-structure/issues/11-open-the-function-reference-page-from-help.md).
