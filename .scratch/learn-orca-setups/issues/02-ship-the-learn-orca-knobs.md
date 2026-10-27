# 02 — Ship the Learn Orca knobs

Status: resolved

**What to build:**

Port the KNOBS patch from Learn Orca's setups menu
(`docs/reference/learn-orca/examples/setups/knobs.orca`): eight Control Change knobs on channel
`00` whose values glide to one of six presets, all chosen by one preset, with a guide and an entry
in `Help > Examples`.

## Acceptance criteria

- [x] `examples/knobs.orcvs` sends controllers `41` to `48`, the controllers Orca's `!` sends knobs
      `1` to `8` to, with Orca's presets scaled as Orca scales them to MIDI values.
- [x] Changing the one preset moves every knob toward its value in that preset by at most its
      Interpolation's rate a Tick.
- [x] `examples/knobs.md` maps each Orca operator to its Orcvs Function, gives the presets and
      exact columns, and lists where it differs from Orca.
- [x] A test runs the exact shipped file through three presets, asserting every Tick's delivery and
      that no preset Cell is diagnosed.
- [x] `Help > Examples > Orca Knobs` opens it, and `/play/?example=orca-knobs` links to it.

## Comments

### 2026-10-27 — Resolved

Each knob is a Delay `~*0101` and an Add `.+04&^nn`, whose nested Read north reads the preset at
(22, 0), over a Control Change `!c004k~>04&>`, whose nested Interpolation glides toward the pair
its nested Read east selects from the knob's Comment of presets. Orca's `Z` with an empty rate holds
its value, so the port gives the Interpolation a rate of `04`. Test:
`playback::tests::shipped_knobs::the_knobs_glide_to_each_preset_selected_and_hold_it`. The audible
check is `03`.
