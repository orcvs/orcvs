# Keeping a Tick's answer span

Orcvs does not keep the span each root wrote during a Tick past the Tick Plan. Once a Tick commits, only Cell contents remain. Anything the console needs about a root's answer is derived from the current Source revision, never from a record of the last Tick.

## Why this is out of scope

During a Tick, each admitted write knows the span its root answered into. The Tick Plan flattens those writes into per-Cell writes, and the span is gone. Keeping it would make it the first Tick outcome the model stores, and that conflicts with how the rest of the console is derived:

- **It goes stale.** The Output Portal highlight reads only the current Source revision, so it never shows something the Source no longer holds (`.scratch/syntax-highlighting/issues/05`). A kept span describes the last Tick, so an edit to the root or its Output Portal row would need a rule for when to discard it.
- **It can lie about ownership.** Writes resolve Cell by Cell, and a later producer wins every Cell it overlaps. An earlier root's kept span can name Cells that now hold another root's answer.
- **It removes nothing.** The Output Portal derivation is still needed before the first Tick, after an edit, and for an empty Sequence, where there is no span to read. The span would be a second source of truth beside it, not a replacement.
- **It widens a public surface.** The Tick Plan crosses into playback and tests, so carrying a span on it adds to what they depend on.

What it would have bought is narrow: the written-content rule (`.scratch/syntax-highlighting/issues/12`) extends a Sequence-capable highlight over stale or hand-written content that directly follows an answer, and a kept span would not. That rule is paint-only and reads the current revision, which is why it was chosen.

## When to reconsider

Reconsider when a second consumer genuinely needs "what did this root write last Tick", such as an inspection view or a Tick history. The highlight alone does not justify it. A reconsideration needs answers to the questions above: where the span lives, when an edit discards it, and how it reads when a later producer overwrote part of it.

## Prior requests

- `syntax-highlighting/13`: "Decide whether a Tick's admitted answer span is kept for the next frame"
