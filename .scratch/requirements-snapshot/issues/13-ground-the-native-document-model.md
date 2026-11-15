# 13 — Ground the native document model and per-key settings fallback

**Status:** needs-triage

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
Waiting on Toby to accept or change it; its open questions list where the code and the spec differ.
