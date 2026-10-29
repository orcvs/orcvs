# 01 — One section per output type

Status: resolved
Type: task

**What to build:** the **O** list shows a labelled **MIDI** section and a **UDP/OSC** section, and
its closed field names every live output ([spec](../spec.md), ADR 0094).

- [x] Each section appears only where this build can use it.
- [x] The closed field reads e.g. `Studio Synth · 127.0.0.1`, or `None`.
- [x] `DestinationPresentation` and its test-only copy are gone.
