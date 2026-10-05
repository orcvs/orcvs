# 01 — Prototype the empty Cell glyph

Status: needs-triage
Blocked by: None — can start immediately

**What to build:**

A syntax prototype that draws the same Source Grids under each of the three options in
[`../spec.md`](../spec.md), so the user can choose the empty Cell glyph by looking at them. The
semantics are fixed by ADR 0033, ADR 0063 and ADR 0066 and are the same in every option. Only the
glyph differs.

The prototype is [`../prototype.html`](../prototype.html). It is a self-contained interactive page,
as `docs/agents/syntax-prototypes.md` requires. It has three scenarios, each a sequence of Grids with
one frame per Tick or edit. Each frame shows the three options side by side. Every Grid has its
diagnostic beside it: the evaluation when one succeeds, **pending, no diagnostic** when an operand
is unwritten, and the error when a slot is malformed. Every Grid also shows the Source File line it
would save. Changed Cells are outlined. Reset, previous and next walk each scenario
deterministically. A **Plain text** view drops the tint and shows what a file, a diff or a terminal
shows. It is the view where options 1 and 2 coincide.

- **A. One-row nested tracker.** `!~0064@t0208C4D4E4  G4C5  E404`, with a Delay above it supplying
  the Bang. The performer edits the index onto a blank Item (a rest), fills the Item in, then
  deletes half of it (malformed).
- **B. Track feeds Timed Play through a Portal.** A Clock writes Track's index, a root Track writes
  the selected Item south into Timed Play's note slot, and a Delay Bangs Timed Play. Ticks 0–4 run
  over the List `C4 __ E4 G4`. The Source File is loaded with both slots unwritten.
- **C. Arithmetic with a pending operand.** `.+01` typed one Cell at a time, from pending to
  malformed `0 ` to `03`, then retyped as `.+01 02` (ADR 0033's example). A Comment row shows what
  each option does to spaces in Comment text.

Option 3 needs a respelling of the numeric `.` family and of the Clock `~.`. The prototype uses
**`#` (`#+`, `#x`, `#v` …) and `~:` as placeholders**, labelled as hypothetical in the page, only so
the Grids can be drawn. Choosing option 3 opens the respelling as its own question.

The prototype lives in this directory because this effort owns it. It is outside the
`lang/*-prototype.html` CodeRabbit path filter, so a review run will read it. Treat any finding
against it as design input, as `syntax-prototypes.md` says.

## Key frames in plain text

Each Grid is bracketed so its trailing empty Cells are visible. Option 2 is drawn as the console
would paint it. Its file is identical to option 1's.

#### A, edit 1: index 03 selects the blank Item

```text
1 space   [~*0101                          ]
          [**                              ]
          [!~0064@t0308C4D4E4  G4C5  E404  ]
          [                                ]

2 painted [~*0101..........................]
          [**..............................]
          [!~0064@t0308C4D4E4..G4C5..E404..]
          [................................]

3 Source  [~*0101..........................]
          [**..............................]
          [!~0064@t0308C4D4E4..G4C5..E404..]
          [................................]
```

#### B, as loaded from the file (before Tick 0)

```text
1 space   [~*0101  ~.0104      ]
          [      @t  04C4  E4G4]
          [!~0064  02          ]

2 painted [~*0101..~.0104......]
          [......@t..04C4..E4G4]
          [!~0064..02..........]

3 Source  [~*0101..~:0104......]
          [......@t..04C4..E4G4]
          [!~0064..02..........]
```

#### B, Tick 1: Clock answers 01, Track copies the blank Item

```text
1 space   [~*0101  ~.0104      ]
          [**    @t0104C4  E4G4]
          [!~0064  02          ]

2 painted [~*0101..~.0104......]
          [**....@t0104C4..E4G4]
          [!~0064..02..........]

3 Source  [~*0101..~:0104......]
          [**....@t0104C4..E4G4]
          [!~0064..02..........]
```

#### C, edit 0: `.+01` with its second slot unwritten

```text
1 space   [.+01                    ]
          [                        ]
          [|| an empty slot waits  ]

2 painted [.+01....................]
          [........................]
          [|| an empty slot waits  ]

3 Source  [#+01....................]
          [........................]
          [||.an.empty.slot.waits..]
```

#### C, edit 3: `.+01 02`

```text
1 space   [.+01 02                 ]
          [03                      ]
          [|| an empty slot waits  ]

2 painted [.+01.02.................]
          [03......................]
          [|| an empty slot waits  ]

3 Source  [#+01.02.................]
          [03......................]
          [||.an.empty.slot.waits..]
```

What to look at:

- **A, B.** Under options 2 and 3, a blank List Item reads as `..`, a visible rest. Under option 1
  it is a gap that reads as spacing.
- **B, loaded.** Under option 1 the unwritten index and note slots are invisible in the file. Under
  option 3 they are visible in the file. Under option 2 they are visible only in the console.
- **C, edit 3.** Under option 2, the painted dot and the numeric sigil are the same glyph:
  `.+01.02`. The painted dot is legible only while it is clearly dimmer than Source.
- **C, Comment.** Option 3 puts dots between the words of every Comment. Option 2 can leave Comment
  text alone, because the painter knows the Comment's claim.

## Acceptance criteria

- [ ] The user has walked the prototype and chosen option 1, 2 or 3, and the choice is recorded
      under `## Answer` in this ticket.
- [ ] Follow-up issues for the chosen option are filed in this effort:
  - Option 1: none beyond recording the decision, for example as a `CONTEXT.md` note on **Cell**.
  - Option 2: an ADR for the paint rule, settling the sub-choices in `spec.md`, and a
    `console/src/paint.rs` issue.
  - Option 3: an ADR choosing the numeric family's new sigil and Clock's new spelling, and then
    issues to respell the Function table, parser, Function reference, ADR and `CONTEXT.md` code
    spans, and tests; to change the Source File format in `orcvs/src/source/file.rs`, including
    trailing-empty trimming and whether a space is read as empty; and to migrate or refuse
    existing Source Files.
- [ ] The prototype is deleted once the decision is recorded.
