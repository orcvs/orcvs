# 04 — Make the knobs audible on the Built-in Synth

Status: resolved

**What to build:**

`examples/knobs.orcvs` did nothing on orcvs.com: it sent controllers `41` to `48`, which the
Built-in Synth ignores and the Sound Module does not show, and it played no notes. Point the knobs
at the Built-in Synth's effects, Control Change `01` to `08` (ADR 0088), make them move on their
own, and give the patch a bassline so the effects have something to act on.

## Acceptance criteria

- [x] Six knobs send `MD`, `OF`, `DL`, `CH`, `R1` and `R2` on channel `00`, so the Sound Module
      shows each effect move; `LN` and `EQ` keep their load values.
- [x] Each knob's value climbs with a Clock, a step at a time, and an Interpolation glides it back
      to `00` when the Clock wraps, so no value jumps.
- [x] A sixteen-step bassline on channel `02`, the saw, plays one step a Tick from a Comment, so the
      file opens with no lexical diagnostic.
- [x] The shipped-file test asserts every Tick's bassline note and Control Changes past `MD`'s wrap.
- [x] `examples/knobs.md` describes the bassline and the knobs, and what controllers `01` to `08`
      mean on a General MIDI device.

## Comments

### 2026-10-31 — Resolved

Toby reported that `?example=knobs` did nothing and chose to drive the synth's effects, then asked
for "some sort of cool 303 baseline thing to make the effects audible". A first version kept Learn
Orca's six presets with an Add, a Read north and a Read east per knob. Toby found it more complex
than it needed to be, with nothing moving unless a preset was typed, so each knob is now a Clock
under an Interpolation. Only `MD` sounds until the other effects' tickets (`orcvs-site/17` to `23`)
land. Test: `playback::tests::shipped_knobs::the_knobs_sweep_the_effects_over_the_bassline`.
