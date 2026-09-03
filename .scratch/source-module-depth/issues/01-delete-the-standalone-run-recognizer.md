# 01 — Delete the standalone-run recognizer

**What to build:** A Source made only of Bangs and Activations — `**`, `^^`, `>>` and the rest —
derives the same Expression it does today, without a second reading of the characters. The
partition already names every one of those spellings; the Expression is built from the kinds it
established rather than from a recognizer that re-reads the same bytes and re-applies the same
`Activation::try_from`.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

- [x] A run of Bangs and Activations derives the Expression it derives today.
- [x] No function outside the row partition converts two Source characters into a Bang or an
      Activation.
- [x] The length and parity checks the retired recognizer performed are expressed as a property of
      the units covering the Span, not restated.
- [x] The existing `**`, `^^` and `>>` cases pass unchanged; no new behavioural test is needed.

## Comments

The friction is that the answer already exists, typed, twelve lines from where it is recomputed:
`parse_span` binds the Language Units covering the Span and then pattern-matches over them for the
single-Operand-Literal case, while the standalone run is recognised from raw bytes above it. The
file demonstrates the technique it declines to use.

ADR 0024 makes the partition the single owner of what a two-Cell spelling is. This is that ADR
applied to the one place still holding a second opinion.
