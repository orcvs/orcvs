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
