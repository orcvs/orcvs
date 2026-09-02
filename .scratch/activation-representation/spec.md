# Prototype the Activation representation

**Status:** ready-for-agent

## Goal

Choose the smallest coherent implementation model for the Source-resident directional activation
spellings without changing their fixed movement and collision behavior or accidentally expanding
the FRC value and Sequence language.

## Constraints

- Compare a distinct spatial Language Unit, a Self-Banging Source Function, and a source-encodable
  value model.
- Keep recognition, Tick scheduling, Source writes, collision-to-Bang, Glyph classification, and
  generated-Source behavior in the comparison.
- Treat Directional identity and one move per Tick as observable behavior; Rust enum placement is an
  implementation choice.
- Self-Banging Functions are root-only Source Functions. They gain no operand, runtime-value, or
  Sequence behavior unless a later focused design explicitly demonstrates and specifies it.

## Delivery

1. `issues/01-prototype-the-activation-representation.md`
