# Syntax Prototypes

Use a self-contained interactive HTML file for every prototype that explores Orcvs syntax or evaluation semantics.

## Required presentation

- Start from actual Orcvs grammar: contiguous two-Cell prefix Functions and typed Atom encodings.
- Label every hypothetical Function name, signature, or syntax directly in the prototype.
- Present behaviour as a sequence of Orcvs Source Grids, one frame per Tick or edit.
- Show the relevant diagnostic beside each Grid, including successful context when no error occurs.
- Make Source changes visually identifiable and state when generated code first becomes executable.
- Include side-by-side models when the prototype compares semantics.
- Provide guided walkthrough controls that reset deterministically and advance one meaningful transition at a time.

The prototype is complete when a reader can decide the design question by following the Grids without translating from an abstract state dump.

## Review scope

Prototypes are excluded from CodeRabbit by `reviews.path_filters` in `.coderabbit.yaml`. The filter suppresses findings, not reading: an excluded prototype still appears in the CLI's `reviewedFiles`, so its content is still sent to the CodeRabbit API. Exclude a file here to stop review noise, not to keep its content off the wire.

A prototype explores one design question and is discarded once that question is decided. While the question is open it may hold competing spellings, contradictory rows, or a deliberately unresolved comparison — that tension is the artifact's content, not a defect in it. A review finding against a prototype therefore proposes a design decision while presenting it as a correction, and applying one silently closes the question the prototype exists to keep open.

Review a prototype only when someone asks for that specific prototype by name. CodeRabbit has no per-run override, so an explicit request means lifting the filter for that run and restoring it afterwards; treat the resulting findings as design input to put to the author, never as fixes to apply.
