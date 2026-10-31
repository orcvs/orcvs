# 04 — Make the knobs audible on the Built-in Synth

Status: resolved

**What to build:**

`examples/knobs.orcvs` did nothing on orcvs.com: it sent controllers `41` to `48`, which the
Built-in Synth ignores and the Sound Module does not show, and it played no notes. Point the eight
knobs at the Built-in Synth's effects, Control Change `01` to `08` (ADR 0088), with presets chosen
for those effects, and give the patch a bassline so the effects have something to act on.

## Acceptance criteria

- [x] Knob `k` sends controller `k` on channel `00`, so the Sound Module shows every knob's effect
      move with the preset.
- [x] No preset drives `LN` below `40` or tilts `EQ` far from flat, and preset `01` is the synth's
      load values.
- [x] A sixteen-step bassline on channel `02`, the saw, plays one step a Tick from a Comment, so the
      file opens with no lexical diagnostic.
- [x] The shipped-file test asserts every Tick's bassline note and Control Changes through three
      presets.
- [x] `examples/knobs.md` describes the bassline, the effects and where the patch now differs from
      Learn Orca.

## Comments

### 2026-10-31 — Resolved

Toby reported that `?example=knobs` did nothing and chose to drive the synth's effects, then asked
for "some sort of cool 303 baseline thing to make the effects audible". The bassline is the
Sequencer's voice machine reading `A1 A1 A2 A1 C2 A1 G2 A1 A1 E2 A1 A2 C3 A1 G2 E2`. The
Interpolation's rate rises from `04` to `08`, so a preset is reached within sixteen Ticks. Only `MD`
sounds until the other effects' tickets (`orcvs-site/17` to `23`) land. Test:
`playback::tests::shipped_knobs::the_knobs_glide_the_effects_to_each_preset_over_the_bassline`.
