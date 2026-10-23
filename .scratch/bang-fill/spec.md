# A Bang keeps its own paint inside a Portal

Status: resolved

## Problem Statement

A Bang inside an Output Portal draws its own glyph colour over the Portal's fill, so a Bang a Producer writes takes the Output Portal style. A Bang should take the Bang style wherever it stands.

## Solution

The 2026-10-22 amendment to [ADR 0053](../../docs/adr/0053-one-theme-styles-the-whole-console.md) decides it: a Bang keeps its own glyph, fill and border inside an Input or Output Portal, as a bound Function does, and a lock still paints it. `style::fill_overlay` bypasses a Bang beside a Function, and `glyph_overlay` loses its separate Bang filter.

## Testing Decisions

The tests that pin the Portal tint on a Bang are flipped at the same sites. The contrast and colour-vision gates reuse `style`'s composition and follow without change.

## Tickets

- 01 — Paint a Bang with its own fill inside a Portal
