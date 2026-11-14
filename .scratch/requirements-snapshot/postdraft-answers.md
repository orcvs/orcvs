# Post-draft answers: 30 contributor questions

Route taken for every question: `AGENTS.md` (Sources of truth: "Read the owning requirement before
the ADRs") -> `docs/requirements/README.md` (topic table) -> the topic document -> the cited ADR,
then code or tests where the requirement named them. `.scratch/requirements-snapshot/` was not read.
"Files to owner" counts files opened before the owning rule was in hand, starting at AGENTS.md.

General routing note: the CLAUDE.md text injected into this session's system context is an older
AGENTS.md. Its Sources of truth say only "Architecture and vocabulary: `CONTEXT.md` and
`docs/adr/`", and its Domain docs section has no pointer to `docs/requirements/`. The AGENTS.md on
disk has both pointers. An agent working from the injected copy alone would never be sent to the
requirements.

---

## Language

### L1. "`.+01 02` gives a diagnostic; `.+0102` works. Why?"
**Answer:** An Expression claims exactly the Cells its arity declares. Inside that claim an empty
Cell counts as a character, not as a separator. So the second operand of `.+01 02` is `" 0"`. That
is not a Number, so the Addition is refused at that operand. The trailing `2` then starts a new
(invalid) Expression. Spaces mean nothing between Expressions and something inside one.
**Owner:** language.md LANG-070 (with LANG-060). Sources: ADR 0033 ("An empty Cell is a character
like any other", which gives this exact example), ADR 0028, ADR 0061. Test
`a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** AGENTS -> README -> language.md. 3 files. Clean. LANG-070 quotes the example itself.

### L2. "Can I make Timed Play's blank `length` mean `01`?"
**Answer:** Yes, mechanically. Change the operand's declared type in the Function table row, from
`length: Length` to `length: Defaulted<Length, 0x01>`. This type-checks because `Length` is an
operand whose Token is Number. The Parser, the Language Map and the Tick need no change. A `lang`
test binds every declared blank. A blank then binds exactly as a written `01` would, and the body
cannot tell the two apart.
Today a blank length is a required slot. The Play is diagnosed and is not a root (LANG-280,
LANG-290).
This changes language semantics, so it needs an ADR, or an amendment to ADR 0109 or 0016. The same
change must restate LANG-320 and PLAY-220 and advance their markers.
**Owner:** language.md LANG-310, LANG-320; playback.md PLAY-220 (what length means). Sources: ADR
0109 (Consequences: "Making an operand default to a value is a change to its declared type"), ADR
0108. Code: `lang/src/atom.rs` row `TimedPlay`; `lang/src/operand.rs` `Defaulted`, `Length`.
**Gap/open:** you would have to decide on consistency. Timed Chord Play `!#` and Monophonic Play
`!%` also have a `length`, and Mono's length `00` means "replace with silence" (PLAY-230).
**Confidence:** high on the mechanism. The need for an ADR is inferred from requirements/README.md
("new decisions keep arriving as ADRs").
**Discovery:** 4 files (plus atom.rs and operand.rs to confirm the type is legal). Clean.

### L3. "A Read nested in a Play's note reads two empty Cells. Is anything diagnosed, and what happens to the Cells under the Read?"
**Answer:** Nothing is diagnosed. A Read of two empty Cells answers a Clear. A nested Clear leaves
the slot unwritten, and a Play's note is `Optional<Note>`, so the Play sends nothing and reports
nothing. A nested Function still writes through its own Output Portal. The Read therefore writes two
spaces into the Cells under it (one row south), clearing whatever it last wrote there.
All of this happens only if the Play is Bang-activated: a nested Function takes its Turn only
through its root.
**Owner:** LANG-220 (a Read answers a Clear for two empty Cells), LANG-330 (a Clear writes two
spaces, root or nested), LANG-270 (nested Output Portal write), LANG-300 (optional note), EVAL-042.
Sources: ADR 0067, 0073, 0061, 0108. Tests `orcvs/src/source/tick/blank_note.rs`
`a_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing_and_reports_nothing`,
`a_cleared_note_leaves_the_clear_under_the_nested_read`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files to LANG-300. The answer has to be assembled from four rules: no single rule
covers "nested Clear into an optional slot also writes its Output Portal". The clearest statement is
the module doc of `blank_note.rs`.

### L4. "Why do the Note pairs east of my Track show diagnostics?"
**Answer:** A pair that spells a Number or a Note where an Expression could start has no slot to
type it, so the Parser refuses it as an unknown spelling. The cost is one Cell per refusal, so a pair
gets two diagnostics. This explicitly includes the pairs a Track selects from. The diagnostics are
cosmetic: Track still reads those pairs. A 2026-10-16 ruling would stop diagnosing standalone Number
and Note pairs, but it has no ADR yet, so the current behaviour binds. To avoid the diagnostics
today, frame the pattern in a Comment (see L6).
**Owner:** language.md LANG-130 (also LANG-440, LANG-060). Source: ADR 0021 ("a standalone operand
literal without typed Expression context is invalid").
**Gap/open:** Open decision. `.scratch/source-diagnostics/issues/01` (an ADR to write,
ready-for-agent) and `04` (implementation).
**Confidence:** high.
**Discovery:** 3 files.
**Ambiguity:** LANG-130 says the rule "includes ... the pairs a Track or Read selects from" and
cites only ADR 0021. ADR 0067 §"The Cells east of Track are ordinary Source" says explicitly that
whether a bare `C4` is diagnosed "is a question for the Parser ... this decision does not answer
it". The requirement settles, by inference from ADR 0021's general clause, a question the
Track-specific ADR leaves open. I would not call it wrong, but its source line should cite ADR
0067's deferral.

### L5. "Old Source used `C4MA` to sound a major scale; now it sounds three notes. Bug?"
**Answer:** Not a bug. A 2026-11-13 amendment renamed the Interval Sets. `MA` now names the major
triad (0 4 7), and the major scale is `io` (Ionian). The natural minor scale moved from `mi` to `ae`,
and `mi` is now the minor triad. The old names change meaning rather than disappear, so nothing
diagnoses old Source. Write `C4io` to sound all seven tones.
**Owner:** language.md LANG-140 (one Interval Set type, names listed in `docs/functions.md#interval-sets`).
Source: ADR 0105, 2026-11-13 amendment ("`MA` and `mi` change meaning rather than disappear ...
nothing diagnoses it").
**Gap/open:** none.
**Confidence:** high.
**Discovery:** AGENTS -> README -> language.md -> functions.md -> ADR 0105. 5 files.
**Routing failure (minor):** LANG-140 states the current list but does not record the rename. Only
ADR 0105 explains why old Source changed meaning. ADR 0105's original Decision paragraph still reads
"`C4MA` in a Chord Play sounds all seven tones". That is superseded by its own amendment, which a
reader skimming the Decision could miss.

### L6. "Can I keep a melody in a Comment, `|| C4D4E4 ||`, and have a Track read it?"
**Answer:** Yes. Comment text is never parsed, so the pairs are ordinary Cells and raise no
standalone-literal diagnostic. Track reads a pair of Comment text as data.
- **Placement:** the Track has to stand outside the Comment, to its left. Everything after `||`
  is unparsed text.
- **Alignment:** Track counts its pairs from the end of its last operand, so the pairs it selects
  must line up with the text. Its indices must never select a pair that touches a delimiter: a
  partial pair is diagnosed and nothing is written. For example, `&t0103||C4D4||` reads `C4`.
  With the spaces in your example, offset `index` so it skips the ` |`/`| ` pairs.

This is the "tracker idiom" the rule was written for.
**Owner:** language.md LANG-090 (text is ordinary Cells; a pair touching a delimiter is partial
input), LANG-080, LANG-200 (Track counts from its last operand), LANG-220. Sources: ADR 0035, ADR
0074, ADR 0067. Test `orcvs/src/source/tick/track.rs` `track_reads_a_pair_of_a_comments_text`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

## Evaluation

### E1. "`.+0102` below a `>>` mover's path: why does the mover sometimes bang and sometimes pass?"
**Answer:** At its Turn, a mover tests only the Cells its one-Cell step would newly enter, in
working Source after every earlier admitted effect. If those Cells are empty and in the Grid, it
moves. Otherwise it writes `**` over itself, and that `**` is a Bang Function on the next Tick.
- **Off the path:** a root on another row never blocks the mover. Passing over it activates
  nothing, because only the `**` a blocked mover leaves bangs, and only roots anchored exactly at
  `(x, y±1)` or `(x±2, y)`.
- **On the path:** if your `.+` writes into the path (its Output Portal is the row south of it),
  the `03` it writes every Tick decides it. The mover passes while those Cells are empty at its
  Turn and bangs once it reaches them. Occupancy adds no Dependency, so row-major Position decides
  who goes first.
- **Contact:** if the mover's destination covers the `.+`'s own Cells, the schedule orders the
  move first and it bangs.

Separately, a value root like `.+` takes a Turn every Tick in the implementation, Bang or not.
Whether it should is an open decision.
**Owner:** evaluation.md EVAL-054, EVAL-025, EVAL-051, EVAL-021, EVAL-056; open decision in
EVAL-050. Sources: ADR 0006, 0060, 0014, 0072, 0020. Tests `movers.rs`
`a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops`,
`the_schedule_orders_a_move_ahead_of_the_computation_it_would_land_on`,
`complete_aligned_root_contact_activates_nothing_until_the_bang_it_leaves_fires`.
**Gap/open:** EVAL open decision D5 (is a value root inert until Bang? ADR 0006 vs ADR 0032).
**Confidence:** low. The question does not say how "below" relates the `.+` and its Output Portal
to the path, and no requirement or test addresses this exact layout.
**Discovery:** 3 files to EVAL-054, then movers.rs to look for an analogous test. No single owner:
"why sometimes" needs three rules combined.

### E2. "A producer below my `!>` Play writes its Note. Does it play this Tick or next?"
**Answer:** This Tick, provided the Play is Bang-activated this Tick. Turns are ordered by
Dependency, not by row. A value input waits for every writer whose Reservation covers it, so a
producer below its consumer reaches it in the same Tick.
The one exception is a dynamic writer (Write, Track) landing on Cells a completed Turn already read.
That is next-Tick feedback, which happens only when the Play itself feeds that writer.
**Owner:** evaluation.md EVAL-020 ("A Producer below its consumer reaches it in the same Tick"),
EVAL-022; exception EVAL-028. Sources: ADR 0032, 0034, 0060, 0067. Test `live_editing.rs`
`live_cross_boundary_chain_uses_lower_producers`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### E3. "My Grid has a cycle in one corner; why do other voices still play?"
**Answer:** No error rejects a whole Tick. A same-Tick cycle stops only the Expressions on it and
those that depend on them by data, activation, lock or overwrite. Every other Expression takes its
Turns exactly as if the stopped ones were absent. The cycle is diagnosed once, and each starved
Expression is diagnosed at its root. If the cycle is found mid-Tick, Turns already completed keep
their effects.
**Owner:** evaluation.md EVAL-070, EVAL-071, EVAL-072, EVAL-073. Sources: ADR 0065, 0034, 0060,
0014, 0068.
**Gap/open:** emission/activation cycles through operand Reservations are still open (placement-semantics/04).
**Confidence:** high.
**Discovery:** 3 files.

### E4. "I typed while playing and my edit seemed to land a Tick late — is the Tick overwriting me?"
**Answer:** No. A Tick is planned without a lock and committed only if the Source revision is
unchanged. If you edit during planning, that plan is dropped whole and the Tick is replanned at the
same absolute Tick from a revision that holds your edit, so no musical Tick is consumed. After
three refusals the fourth attempt runs under the write lock and cannot be refused.
The perceived lateness has three likely causes:
- An edit made just after a commit is first heard on the next Tick.
- Newly typed Function code first takes a Turn from the next Source Snapshot.
- On the site, the Built-in Synth sounds a fixed delay (target 50 ms) after each Tick's planned
  instant.
**Owner:** evaluation.md EVAL-090, EVAL-091, EVAL-092, EVAL-012; playback.md PLAY-540. Source: ADR
0057 (also ADR 0087 for the delay).
**Gap/open:** none.
**Confidence:** high on "never overwritten". Medium on the cause of the perceived lateness.
**Discovery:** 3 files.

### E5. "After Stop then Space, my `~.` Clock resumed mid-cycle. Bug?"
**Answer:** Not a bug. A stop holds the absolute Tick, and the next run begins at the Tick after
the last one executed, as in Orca. Every Clock, Delay and Euclidean resumes on the step it stopped
on. To start over, Rewind (the Panel's `R` Cell, or Command/Ctrl Shift R on native), which makes
Tick `0` next. Opening a Source, File New and the Function Reference also start at Tick `0`.
**Owner:** playback.md PLAY-150, PLAY-160 (evaluation.md EVAL-060 defers to playback). Sources: ADR
0076 (amends ADR 0012's first-Tick clause), ADR 0089 (the Panel Rewind on the web). Test
`orcvs/src/playback.rs` `a_clock_resumes_at_the_step_it_stopped_on`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** AGENTS -> README -> evaluation.md (README lists "time and feedback Functions" under
Evaluation) -> EVAL-060 -> playback.md. 4 files. The README routed to the wrong topic first, but
EVAL-060 sent me on to the owner.

### E6. "Why does the console still light a Read's Cells after I edited another voice while paused?"
**Answer:** By design. Stopping keeps the last Tick Portals record, so a paused tracker keeps
showing what its last Tick read. After an edit the record is `Retained`. A retained entry stays lit
when three things hold:
- no edit has touched its owner's subtree;
- no edit has touched a nested read that selected its Cells;
- the displayed Language Map still anchors the same Function there.

Editing one voice therefore leaves the other voices lit until the next Tick replaces the record.
**Owner:** evaluation.md EVAL-106; console.md CONS-029. Source: ADR 0077 ("Editing one voice of a
paused tracker therefore leaves the other voices' reads and writes lit"). Test
`console/src/console/kittest_tests.rs`
`editing_one_voice_of_a_paused_tracker_keeps_the_other_voices_read_lit`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files. Two topic documents share the rule: EVAL-106 holds the record, CONS-029
the paint. They agree. CONS-029 leaves out ADR 0077's condition that the Language Map still anchors
the entry's Function, so it is incomplete, not wrong.

## Playback

### P1. "After Stop, my hardware synth's sustain pedal and pitch wheel reset. Why, and what is sent?"
**Answer:** Stop sends the safety action, the one output the engine originates. For each of the
sixteen channels in turn it sends:
1. CC 123 All Notes Off;
2. CC 121 Reset All Controllers, which releases a latched pedal;
3. a centred Pitch Bend `[0xE0|ch, 0x00, 0x40]`, last, so a device that honours CC 121 cannot move
   the wheel after it.

No Note Off is sent for known notes, and the Note Off schedule is discarded. A refused message does
not end the action: the first refusal is reported and the rest are still attempted. UDP and OSC get
nothing. The reason: a CC or Pitch Bend can leave device state behind on a device Orcvs can no
longer reach.
**Owner:** playback.md PLAY-260, PLAY-270, PLAY-400. Sources: ADR 0016 (safety action), ADR 0076,
ADR 0090. The "remaining channels" clause has no ADR
(`.scratch/midi-output-family/issues/06`).
**Gap/open:** after a refused delivery the code keeps claims and retries, unlike ADR 0016's teardown
clause. Listed as built differently, with the open decision recorded.
**Confidence:** high.
**Discovery:** 3 files. PLAY-260 matches ADR 0016's text exactly.

### P2. "If I pause with Space and play again, does a Clock `~.` start over?"
**Answer:** No; this is the same rule as E5. Space stops, the absolute Tick is held, and the next
run continues from the following Tick, so the Clock resumes mid-cycle. Use Rewind for Tick `0`.
There is no separate paused state.
**Owner:** playback.md PLAY-150 (also PLAY-170 for the Run Clock, CONS-099 for the Panel). Source:
ADR 0076.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### P3. "My `!%` bassline has blank notes on some steps and a blank last pair. What sounds?"
**Answer:**
- **Blank-note steps:** the note is `Optional<Note>`, so a blank-note step sends no Play Command
  and no diagnostic. With no command there is no Mono replacement: the previous note keeps
  sounding until its own scheduled Note Off.
- **Other operands on those steps:** channel, velocity and length still bind and are diagnosed if
  they are wrong.
- **The blank last pair:** that is `legato`, a `Defaulted<Number, 0x00>`, so it binds `00` and
  each replacement retriggers (prior Note Off, then Note On). Any non-`00` value slides instead.

**Owner:** language.md LANG-300, LANG-320; playback.md PLAY-230, PLAY-240. Sources: ADR 0108, ADR
0109, ADR 0016 (2026-11-14 amendment). Tests `lang/src/functions/mod.rs`
`monophonic_play_retriggers_at_legato_00_or_blank_and_slides_at_any_other`;
`orcvs/src/playback.rs` `a_monophonic_play_whose_note_goes_blank_leaves_its_voice_sounding`. The
requirements cite neither test.
**Gap/open:** PLAY-240's edge rules await maintainer confirmation but bind.
**Confidence:** high, after reading the code.
**Discovery:** 4 files plus a grep of the tests.
**Ambiguity:** PLAY-230 says "Every Monophonic Play replaces the channel's Mono-owned claim, whatever
note it held". Read alone, a blank-note `!%` would silence the voice. LANG-300 ("the Play sends
nothing ... by any route") and ADR 0108 mean it does not. CONTEXT.md (Raw Play: "A note an earlier
Turn started sounds to its own Note Off") and the playback test confirm the voice keeps sounding.
PLAY-230 should say "every Monophonic Play Command".

### P4. "No MIDI device is selected. Do my `!o` sends still go out, and in what order relative to `!>` on the same Tick?"
**Answer:** Yes, natively. Network Sends ignore MIDI's connection state and go on while no MIDI
destination is connected. Within a Tick, MIDI is delivered before Network Sends, whatever their
Source order. With no MIDI destination the `!>` commands simply go nowhere: nothing is owned while
disconnected. In the web build every activated `!o` is diagnosed as unsendable, and Playback goes
on. On the site a Linked console would usually have picked the Built-in Synth anyway.
**Owner:** playback.md PLAY-400, PLAY-390, PLAY-360, PLAY-270. Sources: ADR 0090, ADR 0085, ADR 0016
(2026-10-26 amendment). Tests `outputs.rs` `network_sends_go_on_while_midi_is_disconnected`,
`a_ticks_midi_is_delivered_before_its_network_sends`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### P5. "I want to add a third output destination. Where does it go?"
**Answer:** Put it in the one Outputs module, `orcvs/src/playback/outputs.rs` `Outputs`. That
module alone owns every destination, its policy and its failure latch: "A further destination is
one more entry in it."
- **Tick Plan and delivery:** if Source drives the destination, the Tick Plan needs its own ordered
  list beside Play Commands and Network Sends, and you must define its delivery order (MIDI goes
  first today).
- **Seam:** give it an adapter seam taking the command, not wire bytes (as PLAY-420 does for the
  network).
- **Failures:** a latched, once-reported failure published on the observation (PLAY-410).
- **Console:** route its failures through the one Playback status, which gives it a Toast and an
  output tag (CONS-101, CONS-106), and add a labelled Output-list section (CONS-104, ADR 0095).

A new destination type is a design decision and gets an ADR.
**Owner:** playback.md PLAY-380 (also PLAY-390, PLAY-410, PLAY-420); console.md CONS-104, CONS-101.
Sources: ADR 0090, ADR 0095, ADR 0092.
**Gap/open:** whether the network destination channel should be an ADR 0056 slot is open. A new
destination would face the same question. Also, PLAY-120's eviction classes have no slot for a new
failure class (the network class was added without an ADR).
**Confidence:** high.
**Discovery:** 3 files.

### P6. "After a long stall the console didn't show my device error. Was it lost?"
**Answer:** It should not be.
- **The log:** the engine keeps at most 32 diagnostics between drains and evicts by class, least
  serious first. Overruns go before output failures, so a stall's Overruns cannot displace a MIDI
  output failure. Every eviction is counted in an Omitted summary, and a stall is reported as one
  Overrun anyway.
- **The status:** the observation publishes whether the latest MIDI delivery failed, and the
  console's Playback status keeps a MIDI output failure standing until a delivery succeeds or a
  destination is installed.

If you missed it, the Toast probably faded after `[toasts] seconds` or was dismissed. The Panel's
**O** stays in the error colour while the failure stands. A failure is also latched (reported once
until it changes), and a delivery that succeeds clears it. If the device recovered during the stall,
there was nothing left to show.
**Owner:** playback.md PLAY-110, PLAY-120, PLAY-130, PLAY-190, PLAY-410; console.md CONS-101,
CONS-106, CONS-107. Sources: ADR 0055, ADR 0037, ADR 0090, ADR 0092, ADR 0100.
**Gap/open:** the code adds a network-failure class above Overrun that ADR 0055 does not name
(listed as built differently).
**Confidence:** medium. The answer depends on what "didn't show" means.
**Discovery:** 4 files (playback.md, then console.md for the Toast and O). The rule is split across
two topics, and the routing between them is clean.

## Console

### C1. "I want to ship a third built-in Theme. What must it pass, and which tests do I edit?"
**Answer:** It must pass three gates:
- **Contrast (CONS-073):** every painted text state at 4.5:1 or better, except Comment, which is
  only reported.
- **Colour vision (CONS-074):** ΔE00 of 5.0 or more for protanopia and deuteranopia between glyph
  roles, unless both are neutral greys or the roles differ in fill or border.
- **Chrome record:** `every_built_in_chrome_matches_its_record`, against `console/src/theme.md`.

It must also be dark or light, and its identity becomes reserved. You should edit no gate tests:
they iterate every built-in the registry holds (CONS-078, ADR 0082). You edit the definition in
`console/src/theme.rs`, the registry list (`ThemeRegistry::built_in`, typed `[Theme; 2]`) and the
record in `theme.md`.
CONS-075 says exactly two built-ins ship, and each is its appearance's default. The coloured
built-ins were retired by the maintainer on 2026-10-25 (an ADR 0053 amendment). A third built-in is
therefore a decision change: it needs an ADR 0053 amendment and a CONS-075 update. It also sets
`sound.magnitude` for the site (CONS-079).
**Owner:** console.md CONS-075, CONS-073, CONS-074, CONS-078, CONS-064, CONS-066. Sources: ADR 0053
(amendments 2026-10-01/10/23/25), ADR 0082.
**Gap/open:** none listed.
**Confidence:** high.
**Discovery:** 3 files plus `theme_registry.rs` and `style.rs` to confirm which tests iterate.

### C2. "Drawing is slow. Can I cache each Cell's Paint facts on the Language Map?"
**Answer:** No. The Language Map holds only parser Claims: no Source bytes and no whole-Grid Paint
view. Caching finished Paint facts there is the rejected option, because it moved work into every
revision rebuild. The Claim computes `written` once at construction, and the Paint covers only the
visible Positions. A cost claim on this path is asserted by counting Cells and Shapes, not by
timing. Any performance claim needs a reproducible benchmark (AGENTS.md); there are console benches
for `paint_shapes` and the Minimap. The Minimap's per-Cell picture is cached, but in a console
buffer and never on the Language Map.
**Owner:** console.md CONS-026 ("Not: caching finished Paint facts on the Language Map"), CONS-022,
CONS-023, CONS-091. Sources: ADR 0052, ADR 0040, ADR 0080 (ADR 0050 reviewed, superseded).
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### C3. "How does a performer move around the Source on a laptop, and on a phone? Can they pinch to zoom?"
**Answer:**
- **Laptop:** Pan with the wheel or a two-finger scroll, a middle-drag, Alt plus a primary drag,
  or a press or drag on the Minimap. None of these moves the Cursor or the Region. Arrows move the
  Cursor, Tab and Shift Tab step it by Sector, and the view follows the Cursor just far enough.
- **Phone:** a finger drag Pans, a tap selects a Cell (and asks for the on-screen keyboard), and a
  long press then drag selects a Region. The phone page also starts with larger Cells and no
  Minimap.
- **Pinch:** never zooms. On native, zoom is egui's whole-UI zoom by command `+`, `-` and `0`. On
  the web the browser's page zoom is the only zoom.

**Owner:** console.md CONS-032, CONS-033, CONS-052, CONS-057, CONS-090, CONS-110, CONS-111,
CONS-040, CONS-042, CONS-043; site.md SITE-052. Sources: ADR 0045, 0046, 0048, 0058, 0080, 0102,
0091.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files. Spread over many rules, but all in one document.

### C4. "A MIDI clock failure shows as a MIDI problem. Where should it appear, and when does it clear?"
**Answer:** A clock failure is an engine failure, not a MIDI one. It should appear as its own
`ENGINE` Toast, with **O** in the error colour. As an event, the latest engine failure stands until
the next run begins, and a newer one replaces it. A Rewind while playing begins a new run. Showing it
as a MIDI problem is exactly the call-order routing bug that ADR 0092 removed: one Playback status
must route every failure. MIDI selection keeps only its own messages.
**Owner:** console.md CONS-101, CONS-102, CONS-106, CONS-107. Sources: ADR 0092, ADR 0100. Tests
`console/src/playback_status.rs` `each_output_failure_goes_to_its_own_destination`,
`an_engine_failure_is_shown_until_the_next_run_begins`.
**Gap/open:** none listed. If it reproduces, it is a regression against CONS-101.
**Confidence:** high.
**Discovery:** 3 files.

### C5. "Typing into a value box in a menu wipes my Grid. Which controls should keep the keys?"
**Answer:** Every one, and none is named. The Source gets keys only while no control holds egui's
keyboard focus and no popup (a menu, a ComboBox list) is open. The answer is latched after all
widgets have run, and read on the next frame. Keys a control took also disarm an armed fill.
If typing in a menu field reaches the Grid, either the field does not hold egui focus or the latch
is bypassed. That is a regression against CONS-055. Do not fix it by adding the control to a list,
because a named list is the rejected design.
**Owner:** console.md CONS-055, CONS-056, CONS-057. Source: ADR 0048. Tests `kittest_tests.rs`
`typing_with_a_menu_open_leaves_the_source_unwritten`,
`a_focused_menu_item_keeps_region_and_clipboard_commands_from_the_source`; `tests.rs`
`keys_the_bpm_field_took_disarm_a_fill_armed_before_it`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### C6. "Can a phone visitor to the site play an example?"
**Answer:** Yes.
- **Where:** on `/learn/` or `/reference/`, pressing a Grid sends that example to the page's one
  small console, stopped. A link `orcvs.com/?example=…` opens it on the home page.
- **Playing:** there is no Space key, so the Panel's `P` Cell plays and `R` rewinds. Every console
  shows the Panel, the small console included.
- **Sound:** a Linked console picks the page's Built-in Synth. The tap resumes the page's audio
  context and asks for media playback, so the iOS silent switch does not mute it.
- **Layout:** the phone setting gives 16x24-point Cells and no Minimap, and a finger Pans.

**Owner:** console.md CONS-100, CONS-110, CONS-111; site.md SITE-016, SITE-041, SITE-044, SITE-045,
SITE-052. Sources: ADR 0089, 0102, 0086, 0087, 0091.
**Gap/open:** native has no Built-in Synth (irrelevant to phones).
**Confidence:** high.
**Discovery:** 4 files (console.md, then site.md). This is a console-and-site question, and the
README routes it to both.

## Site

### S1. "How do I send a friend a link that opens the Tracker example with sound?"
**Answer:** Send `https://orcvs.com/?example=tracker`. The link name comes from
`console/src/examples.rs`. You can also press the top bar's share icon `↗` on the unchanged
example, which copies exactly that link. An edited example copies a `#source=` link instead.
`/` frames `/play/` with the same query inside the page that holds the Built-in Synth, and a Linked
console picks the synth. Two things to tell your friend:
- The console opens stopped, so they press `P` (or Space).
- A bare `/play/?example=tracker` has no synth, because no page frames it.

**Owner:** site.md SITE-020, SITE-030, SITE-060, SITE-061, SITE-021, SITE-041, SITE-048, SITE-024.
Sources: ADR 0094, ADR 0097, ADR 0087, ADR 0086.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files plus `examples.rs` for the link name.

### S2. "My branch changes `console/`. Why is the preview build stuck or failing with 'was not published within 15 minutes'?"
**Answer:** A Cloudflare build waits for its own commit's console key. It retries every 20 seconds
for up to 15 minutes, then fails, naming the key. The Console web workflow did not publish that key
in time. Likely causes:
- the run failed, or is still queued or running;
- the push came from a fork (fork pull requests do not run the console build). AGENTS.md says to
  push to `origin`, not the `fork` remote;
- a Dependabot branch (built, never published), although its site build takes the rolling console;
- the key was pruned (more than 50 branch archives), so run Console web on the branch;
- a newer pending run replaced a pending one (a known concurrency gap).

Fix the Console web run, then retry the site build from Cloudflare's dashboard. There is no deploy
hook.
**Owner:** site.md SITE-075, SITE-072, SITE-073, SITE-074, SITE-078, SITE-079, SITE-080. Sources:
ADR 0093, ADR 0101. Code `site/scripts/console.sh`, `.github/workflows/console-web.yml`.
**Gap/open:** SITE-074's concurrency group differs from the rule. SITE-080's watch paths can only be
checked in Cloudflare's dashboard (`orcvs-site/26`, ready-for-human).
**Confidence:** high.
**Discovery:** 3 files plus `console.sh` to confirm the message. Engineering CI is out of scope for
the requirements per README, but this lives in site.md as product deploy behaviour, which works.

### S3. "On a phone the Cells are too small and the Minimap covers the Grid. Where do I change that?"
**Answer:** Change it in the shipped `/play/` page: `console/index.html`, element
`<script type="application/toml" id="orcvs_config" data-media="(max-width: 600px)">`. It sets
`[cell] width = 16, height = 24` and `[minimap] open = false`. The web console reads it once at
startup with native's settings reader. Framing pages such as the site pass nothing and inherit it.
The Minimap also hides by itself where 256 physical pixels plus its border do not fit in half the
Source's width. Check the media query matches, and that you are on a fresh load: settings are read
only at startup.
**Owner:** site.md SITE-050, SITE-052; console.md CONS-094, CONS-131, CONS-120. Sources: ADR 0091,
ADR 0102 (16x24), ADR 0096, ADR 0079 (proposed).
**Gap/open:** the `[cell]` keys belong to proposed ADR 0079. They are built but not binding, and are
listed under console.md "Proposed, built".
**Confidence:** high.
**Discovery:** 3 files plus `index.html`.
**Ambiguity:** accepted ADRs 0091 and 0102 set a phone Cell size through `[cell]` keys that exist
only under proposed ADR 0079. CONS-131's settings list omits `[cell]`. An accepted rule (SITE-052)
depends on a non-binding one.

### S4. "My MIDI device is connected, so why does an example on `/learn/` play through the Built-in Synth?"
**Answer:** By design. The page's small console is Linked, and a Linked console picks the Built-in
Synth once the page announces it, even if a MIDI port exists. An example someone opens from a page
should be heard without setup. You can still choose your port from the Panel's **O** Output list; a
destination the performer chose stays chosen. An unlinked console picks the synth only when no port
exists.
**Owner:** site.md SITE-041; playback.md PLAY-520. Source: ADR 0087 (and ADR 0059's 2026-11-01
amendment). Test `console/src/built_in_synth.rs` `a_linked_console_picks_the_synth_over_a_port`.
**Gap/open:** none.
**Confidence:** high.
**Discovery:** 3 files.

### S5. "I want the page to send the console a new message, say a tempo. What must it respect?"
**Answer:** It must respect the page-message contract:
- **Who may send:** only the page that frames the console, from the console's own origin. Messages
  from any other window or origin, of another type, or malformed are ignored.
- **When:** nothing before the console posts `orcvs.ready`.
- **Bursts:** only the latest waiting message applies per frame.
- **Guarantees:** a message keeps a link's guarantees. It cannot start Playback, and a refused
  message keeps the current state and raises a notice.

ADR 0091 explicitly rejects "settings sent by message". A tempo that is really a setting belongs in
the page's `orcvs_config` element, read before the first frame. A runtime tempo message is a new
page protocol and needs an ADR amending ADR 0086, with requirement SITE-012 and its tests updated.
SITE-012's only test covers the happy path, so add tests for the origin and source checks.
**Owner:** site.md SITE-012, SITE-013, SITE-014, SITE-015, SITE-050 ("Not: ... settings sent by
message"). Sources: ADR 0086, ADR 0091.
**Gap/open:** no rule covers a non-Source message at all. That is the gap the new ADR fills.
**Confidence:** medium. Whether tempo counts as a "setting" under ADR 0091 is a judgement call.
**Discovery:** 3 files.

### S6. "On an iPhone the first note of a run comes late or is silent. What should happen?"
**Answer:**
- **Gesture:** the page resumes its audio context inside every pointer, touch or key gesture,
  including in the console's frame (the console posts `orcvs.wake`). It also asks for media
  playback, so the ring/silent switch does not mute it.
- **Held start:** a Playback start on the synth while audio is not running is held until audio
  runs, for at most 500 ms, so Tick 0 is not late.
- **Timing:** each message sounds a fixed delay (target 50 ms) after its Tick's planned instant,
  and a message already later than the delay sounds at once.
- **Fallback:** if no gesture resumed the context, the first `orcvs.midi` resumes it.

**Owner:** site.md SITE-044, SITE-045, SITE-043; playback.md PLAY-540. Sources: ADR 0087 as amended
by ADR 0089.
**Gap/open:** SITE-043 is built differently. `synth/web/synth-host.js` drops messages that arrived
while the context was starting and are more than 500 ms past their planned instant, where the rule
says they sound at once. There is no tracking issue: the drop is described only in resolved
`orcvs-site/16`. That would explain a silent first note.
**Confidence:** high.
**Discovery:** 3 files.

---

## Suspected wrong or ambiguous rules (checked against the cited ADR)

1. **LANG-130** extends ADR 0021's standalone-literal rule to "the pairs a Track or Read selects
   from". ADR 0067 explicitly leaves that question to the Parser. This is an inference beyond the
   Track ADR; the requirement should cite the deferral. Not contradicted by an ADR.
2. **PLAY-230** "Every Monophonic Play replaces the channel's Mono-owned claim" misreads for a
   blank-note `!%`. Under ADR 0108 a blank-note Play sends no command, so nothing is replaced (test
   `a_monophonic_play_whose_note_goes_blank_leaves_its_voice_sounding`). The wording predates ADR
   0108 and should say "Play Command".
3. **SITE-052 / CONS-120:** an accepted rule sets values through keys defined only by proposed ADR
   0079.
4. **CONS-029** leaves out ADR 0077's Language-Map anchor condition for a retained entry.
   Incomplete, not contradictory.
5. **LANG-140** is silent on the 2026-11-13 rename, which changed the meaning of old Source with no
   diagnostic. ADR 0105's original Decision paragraph still says `C4MA` sounds seven tones.

No rule was found that contradicts the ADR it cites. PLAY-260 matches ADR 0016 word for word, and
PLAY-150 matches ADR 0076.
