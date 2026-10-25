# 01 — Orient the README and walk a reader to a first result

**What to build:** The README says what Orcvs is and how far along it is, starts it, and links one
walkthrough. The walkthrough takes a reader from a running console to a Source change they can see,
without MIDI.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The README holds no retired syntax and no command that no longer runs.
- [x] The README states purpose, pre-release status, how to start the console, and where the
      language and contributor material live.
- [x] One linked walkthrough shows an initial Source, the action to take, and the change the next
      Tick makes, with no MIDI dependency.
- [x] Every step of the walkthrough is exercised through the console's own input path, and the
      validation record says what ran and what was not checked.

## Comments

The README at `47103ec` is notes to self rather than orientation. Its two examples, `++0102` and
`++++010202`, spell no Function in the current table: Add is `.+`. Its benchmark command names
`parser_benchmark`, which no crate declares; the benches are `lang`, `source`, `paint`, `overview`
and `stored_source`, and `mise run bench` runs them. The rest is Orca source and design sketches
that describe no current behaviour.

The reader is a technically comfortable evaluator or contributor who has the repository and a Rust
toolchain. MIDI routing is left to a later walkthrough: a Play Command alone does not show that a
note was heard.

Validation record at `47103ec`, on Linux in a headless container. A temporary `egui_kittest` test,
not committed, drove `Console::new` with no storage through every step: File > New on the empty
start, typing `.+0102`, Space, Left Arrow and `5`, Down three and Left six, typing `~.0804`, and
Space to stop. It observed `03`, then `06`, the Clock answering `00` to `03` within 4.5 seconds,
changing every 1.0 seconds at 120 BPM (observed 0.76, 1.00, 0.98, 1.02, 1.00 and 1.00 seconds
between values), and the Clock holding its value for 1.5 seconds after the stop. After the stop it
played again and edited `.+0105` to `.+010a`: across 12 Ticks the `06` below stayed, and the
Render Frame painted the `0a` Cells as an Invalid Number operand, which Paint draws in the Theme's
diagnostic foreground. The run caught one wrong claim in the first draft, and in `function_reference.rs`'s
module documentation: a console that restores nothing starts empty, not on the Function Reference.

Not checked: a window on macOS or Windows, rewinding with Command Shift R, how the diagnostic
colour looks on screen, and a person from the intended audience following the walkthrough. A fresh-context
model review raised fourteen gaps; the ones that would block a reader were fixed, namely the ALSA
prerequisite's order, the File > New chord, Cursor columns, the Clock's row, and where BPM and the
Tick are shown.
