# Baseline answers: 30 contributor questions, current reading path

Reading path used: `AGENTS.md` (= `CLAUDE.md`) -> `CONTEXT.md` (75 KB; the Read tool truncated it at line 357, so it took two reads) -> `docs/agents/domain.md` -> `docs/adr/README.md` (covers numbering only) -> ADR filenames (`ls`) and status lines (a grep over all 106 ADRs) -> grep in ADRs -> code and tests. `docs/requirements/` and `.scratch/requirements-snapshot/` were not read. One grep for "15 minutes" printed a single line from `docs/requirements/site.md`. That line was ignored, and S2 rests on ADR 0093 instead.

Fixed overhead for every question: AGENTS.md, CONTEXT.md, domain.md, ADR README and the ADR status sweep, about 5 opens. The per-question counts below leave that out.

---

## Language

### L1. `.+01 02` gives a diagnostic but `.+0102` works. Why?
**Answer.** An Expression claims the Cells its arity declares, and a space inside that claim is part of an operand, not a separator. In `.+01 02` the second operand is `" 0"`, which is not a Number, so the Addition is refused. The `2` after it then starts the next Expression and is diagnosed on its own. Spaces only cost nothing between Expressions.
**Owner.** ADR 0033 ("An empty Cell is a character like any other", which uses this exact example). Also ADR 0010 (two-Cell Numbers) and `docs/functions.md` "How to read an entry / Spelling". Code: `lang/src/parser.rs` (row walk and claim).
**Gap.** ADR 0033's text still describes Comments as `##`. Its status line says to read ADR 0035 and 0074 for that part.
**Confidence.** High.
**Discovery.** `docs/functions.md` "Spelling" says "no spaces", but not why. Grepping "space" across the likely ADRs found 0033 on the first hit. 3 files.

### L2. Can Timed Play's blank `length` mean `01`?
**Answer.** Yes. This is language design, not an API break. Change the `length` operand's declared type in Timed Play's Function-table row to `Defaulted<Length, 0x01>`, the same way Monophonic Play declares `legato: Defaulted<Number, 0x00>`. The Parser, Language Map and Tick need no change. A blank slot, or a nested Clear, then binds exactly as a written `01` would, and the Function body cannot tell them apart. The `lang` table test that binds every `BLANK` fails if `01` lies outside `Length`'s domain. Only Number-token operands can take a default.
**Owner.** ADR 0109 (`Defaulted<O, BLANK>`), building on ADR 0108. Code: `lang/src/atom.rs` Function table row `TimedPlay`; `lang/src/operand.rs::Defaulted`. CONTEXT "Defaulted Operand".
**Gap.** `Defaulted` requires `O: Operand<Token = Number>`. I did not confirm that `Length` meets that bound, only that `Length` is a `u8` newtype (`atom.rs:166`). Timed Chord Play's `length` would need the same decision to stay consistent.
**Confidence.** High on the mechanism, medium on whether `Length` compiles as `O` without changes.
**Discovery.** The CONTEXT "Defaulted Operand" entry pointed to ADR 0109, and grepping `Defaulted<` found the table row. 3 files. Good routing.

### L3. A Read nested in a Play's note reads two empty Cells. Is anything diagnosed, and what happens to the Cells under the Read?
**Answer.** Nothing is diagnosed, and nothing plays. The Read copies the empty pair, so its answer is a Clear. Like every nested value Function, it writes that Clear as two spaces through its own Output Portal, one row south of the Read's anchor, so whatever stood there is erased. A Clear leaves the note operand unwritten. A Play's note is `Optional<Note>`, so the Play sends no Play Command, raises no diagnostic, and keeps its Turn. Channel, velocity and length are still bound and diagnosed.
**Owner.** ADR 0108 (note optional by type), ADR 0073 (Clear), ADR 0061 (a nested Function also writes through its own Output Portal), ADR 0070 (Read family). Tests: `orcvs/src/source/tick/blank_note.rs::a_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing_and_reports_nothing` and `::a_cleared_note_leaves_the_clear_under_the_nested_read`.
**Gap.** None found.
**Confidence.** High.
**Discovery.** The CONTEXT entries for Return, Clear and Play Function together give the answer, but the "Cells under the Read" half is only stated in the Return and Copy entries. The test file confirmed it. To find the tests I tried four greps in `lang/src` before finding `orcvs/src/source/tick/blank_note.rs`. Nothing routes from an ADR to the Tick test modules. 6 files.

### L4. Why do the Note pairs east of my Track show diagnostics?
**Answer.** Track claims only its operands, so the pairs it selects from are ordinary Source that the Parser walks like any other row content. A bare `C4` where an Expression could start is not a Function spelling, so the Language Map diagnoses it, probably as `unknown function "C4"`. Track still reads those pairs fine at its Turn. Whether a standalone operand literal should be diagnosed at all is an open question. The fix available today is to frame the pattern in a Comment (`|| … ||`), whose text is never parsed, as the Tracker example does.
**Owner.** ADR 0067 ("The Cells east of Track are ordinary Source … whether a bare value … is diagnosed is a question for the Parser"). CONTEXT "Operand Literal" names this exact case as open. Code: `lang/src/parser.rs` (`Function::try_from` failure); `lang/src/error.rs`.
**Gap.** This is a known open design question, and no ADR decides it. I did not confirm the exact message text.
**Confidence.** High on why, medium on the exact message.
**Discovery.** The CONTEXT "Operand Literal" entry answers it directly. Without that sentence I would have had to search ADR 0067. 4 files.

### L5. Old Source used `C4MA` to sound a major scale; now it sounds three notes. Bug?
**Answer.** No, this is a deliberate rename. The 2026-11-13 amendment made `MA` the major triad and named the major scale `io` (Ionian). In the same way `mi` became the minor triad and `ae` the natural minor scale. No alias was kept, and Source that uses `MA` as a scale now sounds a triad without any diagnostic. Write `C4io`.
**Owner.** ADR 0105, "Amendment, 2026-11-13: names without a filler" (table: `MA` -> `io`). CONTEXT "Interval Set". Code: Interval Set table in `lang/src/atom.rs`.
**Gap.** Old Source changes meaning silently, and nothing warns about it. The ADR accepts that as pre-release behaviour.
**Confidence.** High.
**Discovery.** CONTEXT gives the current names but no history. The ADR 0105 status line points to the amendment. 2 files.

### L6. Can I keep a melody in a Comment, `|| C4D4E4 ||`, and have a Track read it?
**Answer.** Yes. A Comment's text Cells are data, and a Read, Track or Copy that selects a pair of them decodes it as it would any uncovered pair. Only the two `||` delimiters are occupied, so a selected pair that touches either delimiter is refused as partial input. Pair alignment matters:
- Track counts pairs from right after its last operand. If the Comment starts there, pair 0 is the `||` itself and is refused.
- The space after `||` shifts every later pair by one Cell, so ` C`, `4D` and so on would be read.

Write `||C4D4E4||` and offset the index, or use `&v`/`&>` Reads into the text, as `examples/tracker.orcvs` does.
**Owner.** ADR 0074 ("A Comment's text is data"), ADR 0035, ADR 0067 (Track reads as a Copy does). CONTEXT "Comment", "Track Function". Tests: `orcvs/src/source/tick/movers.rs::a_mover_meets_comment_text_as_ordinary_cells`, and `write.rs::a_write_into_comment_text_writes_its_cells`.
**Gap.** ADR 0067's reading rule lists "a Comment" as diagnosed input. That predates ADR 0074 and now applies only to the delimiters. ADR 0067's status line does not mention 0074.
**Confidence.** High on the rule, medium on the alignment advice.
**Discovery.** CONTEXT "Comment" answers it. The 0067 wording looked contradictory until I read ADR 0074. 4 files.

## Evaluation

### E1. `.+0102` below a `>>` mover's path: why does the mover sometimes bang and sometimes pass?
**Answer.** The question is ambiguous. If the Add sits directly below the path, its Output Portal is a further row down, and the mover never meets it. The likely case is that the Add's answer, `03` (one row south of its anchor), lies in the path.
- **The bang case.** A mover tests occupancy of the Cells it newly enters, in working Source at its own Turn. It does not wait for a vacancy, and occupancy creates no ordering edge. The Add writes every Tick and, being earlier in Source order, writes before the mover's Turn. So when the mover reaches the pair it is refused, leaves `**` (which fires on the next Tick), and is diagnosed if it met only part of the pair. A mover moves one Cell per Tick, so whether it meets the whole pair or half of it depends on where it started.
- **The pass case.** The mover passes when its destination is clear at its Turn. If it moves into Cells that are only written after its Turn, the later write wins Cell-wise and overwrites the mover's new Cells.
**Owner.** ADR 0060 rule 3b (placement tests occupancy at its Turn), ADR 0020/0032 (dependency then Source order), CONTEXT "Self-Banging Function". Tests: `orcvs/src/source/tick/movers.rs::a_mover_is_blocked_by_an_earlier_overwrite`, `::a_movers_new_cells_can_be_overwritten_after_its_turn`, `::a_move_that_lands_across_two_language_units_diagnoses_its_misalignment`.
**Gap.** No single document states how a mover and a value Function's Output Portal interact. That has to be pieced together from rule 3b and the tests.
**Confidence.** Low. The question is underspecified, and I could not find the "sometimes" mechanism stated anywhere.
**Discovery.** CONTEXT "Self-Banging Function" covers movers against Bang Functions and other movers, but not against value results. I read ADR 0060 and then listed the mover test names, about 40 of them. 4 files and a lot of scanning.

### E2. A producer below my `!>` Play writes its Note. Does it play this Tick or next?
**Answer.** This Tick. Turns are ordered by dependency, not by row, so the producer goes before the Play:
- A static writer, such as `=^` or a value root whose Portal reaches the note, goes first because its Reservation covers Cells the Play reads.
- A dynamic writer, such as `@^`, takes its Turn before every computation that does not feed it.

A Play answers nothing and feeds nothing, so no cycle can form. A writer would land a Tick late only if the reader fed it.
**Owner.** ADR 0032 (schedule by dependency; supersedes ADR 0031's row-major order), ADR 0036 (reserve result Cells), ADR 0060 rule 3a (value inputs wait for suppliers), ADR 0067 2026-10-10 amendment (dynamic writers first). CONTEXT "Dependency" and "Output Portal". Tests: `orcvs/src/source/tick/play_order.rs::fixed_upward_portals_schedule_note_and_bang_before_midi`; `write.rs::a_reader_that_feeds_the_write_sees_it_on_the_next_tick`.
**Gap.** None found.
**Confidence.** High.
**Discovery.** CONTEXT "Dependency" and "Output Portal" contain the rule. `play_order.rs` turned up from the test-file list. 3 files.

### E3. My Grid has a cycle in one corner; why do other voices still play?
**Answer.** An error never stops the performance. A same-Tick dependency cycle stops only the Expressions on it and those that depend on them, each diagnosed. Every other Expression is ordered and takes its Turns as if the stopped ones were absent. For a cycle found mid-Tick, through a Track or another dynamic Portal, Turns already completed keep their Effects.
**Owner.** ADR 0065, ADR 0068. CONTEXT "Tick" and "Dependency". Tests: `orcvs/src/source/tick/cycles.rs::a_cycle_leaves_an_independent_expression_publishing`, `::live_cycles_stop_only_the_expressions_they_reach`.
**Gap.** None.
**Confidence.** High.
**Discovery.** The CONTEXT "Tick" entry says it in one sentence, and the ADR filename 0065 matched. 3 files.

### E4. I typed while playing and my edit seemed to land a Tick late. Is the Tick overwriting me?
**Answer.** No. A Tick is planned from a snapshot without holding the lock, and commits only if the Source revision is unchanged. If your edit lands while the Tick is planning, the plan is dropped whole and re-planned at the same absolute Tick from the revision that includes your edit. After three refusals it plans under the lock. So no edit is ever overwritten. An edit that arrives after a Tick's snapshot was taken and committed first shows from the next Tick, which can feel like a Tick of delay.
**Owner.** ADR 0057. CONTEXT "Live Editing". Code: `orcvs/src/source/planning.rs` (`PlannedTick::commit`), `orcvs/src/source/mod.rs::OPTIMISTIC_TICK_ATTEMPTS`.
**Gap.** None.
**Confidence.** High.
**Discovery.** CONTEXT "Live Editing" cites ADR 0057 directly. 3 files.

### E5. After Stop then Space, my `~.` Clock resumed mid-cycle. Bug?
**Answer.** No, this is intended. A stop holds the absolute Tick, the next run begins at the Tick after the last one executed, and Clock reads the absolute Tick. So it resumes on the step where it stopped, as Orca's pause does. Rewind returns to Tick 0: Command/Ctrl Shift R on native, or the Panel's `R` on every target.
**Owner.** ADR 0076, ADR 0089 (Panel `R`), ADR 0012. CONTEXT "Clock Function", "Rewind".
**Gap.** None.
**Confidence.** High.
**Discovery.** The CONTEXT "Clock Function" entry states it explicitly. 1 file.

### E6. Why does the console still light a Read's Cells after I edited another voice while paused?
**Answer.** The last committed Tick's Portal record is kept. After an edit it is answered as `Retained`, not dropped. A retained entry still paints wherever its owner Function still stands at its anchor and no edit since the commit has written a Cell of its subtree. For a place its operands selected, it also paints only if no descendant read a Cell an edit has since written. Editing another voice leaves this Read's entry valid, so it stays lit. The ADR rejects dropping the record at the first edit, because then every keystroke would unlight every voice in a paused tracker.
**Owner.** ADR 0077. CONTEXT "Tick Portals".
**Gap.** None.
**Confidence.** High.
**Discovery.** CONTEXT "Tick Portals" is long but complete. 2 files.

## Playback

### P1. After Stop, my synth's sustain pedal and pitch wheel reset. Why, and what is sent?
**Answer.** Stop triggers the safety action, which returns all sixteen channels to silence and defaults, one channel at a time. Each channel gets:
1. CC 123 All Notes Off: `[0xB0|ch, 123, 0]`.
2. CC 121 Reset All Controllers: `[0xB0|ch, 121, 0]`. This releases a latched sustain pedal and zeroes the mod wheel.
3. A centred Pitch Bend: `[0xE0|ch, 0x00, 0x40]`. It comes last because device support for CC 121 varies.

No individual Note Offs are sent, and the pending Note-Off schedule is discarded. A refused channel does not stop the action: every channel is still attempted, and the first refusal is reported. Rewind, disconnecting and changing destination send the same action.
**Owner.** ADR 0016 (defines the action), ADR 0076 (a stop sends it), ADR 0090 (MIDI only). CONTEXT "Playback Engine". Code: `orcvs/src/midi.rs::safety_reset_messages`, `send_safety_reset`.
**Gap.** On the Built-in Synth, CC 121 follows RP-015 and clears only Modulation (ADR 0099).
**Confidence.** High.
**Discovery.** CONTEXT "Playback Engine" gives the sequence but no ADR. A grep for "Reset All Controllers" found ADR 0016, where the safety action lives inside an ADR titled "use distinct raw and timed play functions", so the title does not help. 4 files.

### P2. If I pause with Space and play again, does a Clock `~.` start over?
**Answer.** No. Space stops and resumes, and a stop holds the absolute Tick, so the Clock continues on the step it stopped on. Only Rewind (Cmd/Ctrl Shift R, or the Panel `R`) starts it over at Tick 0. Orcvs has no separate "paused" state: ADR 0076 rejects one.
**Owner.** ADR 0076, ADR 0089. CONTEXT "Clock Function", "Rewind", "Run Clock".
**Gap.** None. This is the same question as E5.
**Confidence.** High.
**Discovery.** 1 file.

### P3. My `!%` bassline has blank notes on some steps and a blank last pair. What sounds?
**Answer.** The blank last pair is `legato`, a Defaulted Operand that binds as `00`, so every sounding step retriggers: the prior note stops, then the new one starts. A step with a blank note binds to no note, sends no Play Command and raises no diagnostic. So a blank step does not replace or silence the channel's voice, and the previous note sounds on until its own Note Off. Velocity `00` or length `00`, by contrast, replace the voice with silence. A blank channel, velocity or length is still diagnosed on a blank step.
**Owner.** ADR 0108 (optional note), ADR 0109 (`legato: Defaulted<Number, 0x00>`), ADR 0016 (Mono voice per channel). CONTEXT "Monophonic Play Function", "Play Function". Code: `lang/src/atom.rs` row `MonophonicPlay`. Tests: `lang/src/functions/mod.rs::monophonic_play_retriggers_at_legato_00_or_blank_and_slides_at_any_other`; `orcvs/src/source/tick/blank_note.rs::a_nested_clear_under_a_bang_driven_output_binds_absent_in_a_note_and_as_00_in_legato`.
**Gap.** No document says outright that "a blank-note Mono step leaves the voice sounding". I inferred it from "sends nothing" plus "a note an earlier Turn started sounds to its own Note Off".
**Confidence.** Medium to high.
**Discovery.** CONTEXT plus the Function-table row. 3 files.

### P4. No MIDI device is selected. Do my `!o` sends still go out, and in what order relative to `!>` on the same Tick?
**Answer.** Yes. The MIDI connection gates MIDI alone, and Network Sends go on while no MIDI destination is connected. Within a Tick, MIDI is delivered before Network Sends even if the Source interleaves them; each list keeps its own order. With no device, the `!>` commands deliver nothing, so only the OSC sends leave. Stop, Rewind and the safety action never touch the network.
**Owner.** ADR 0090, ADR 0085. CONTEXT "Playback Engine", "Network Send". Code: `orcvs/src/playback/outputs.rs::Outputs::deliver`.
**Gap.** On the web build, every activated `!u` or `!o` is diagnosed instead, because the browser cannot send.
**Confidence.** High.
**Discovery.** CONTEXT "Playback Engine" states it and cites ADR 0090. 3 files.

### P5. I want to add a third output destination. Where does it go?
**Answer.** One Outputs module in the Playback Engine owns every destination and its policy: the MIDI adapter, note schedule, connection state, failure latches and the network output. A third destination is one more entry there. It needs:
- its own policy for what it waits on and what it latches;
- its own command list in the Tick Plan, if its commands are not MIDI;
- a seam that takes its command and destination, so tests can record it.

On the console side, its failures route through the one Playback status (ADR 0092) and show as a Toast with its own label (ADR 0100). It also appears in the Output list (ADR 0095).
**Owner.** ADR 0090 ("A third destination is one more entry in it"). Code: `orcvs/src/playback/outputs.rs`; `console/src/playback_status.rs`.
**Gap.** ADR 0090 assumes two lists, Play Commands and Network Sends. A third destination type would need a decision on cross-list order.
**Confidence.** High.
**Discovery.** The question is not in CONTEXT. The ADR filename "each output destination keeps its policy in one module" found it by scanning `ls`. 3 files.

### P6. After a long stall the console didn't show my device error. Was it lost?
**Answer.** Almost certainly not. The engine keeps at most 32 diagnostics between drains and evicts by class, least serious first: Overruns before start/retune failures, before output failures, before clock failures. A stall's flood of Overruns therefore never displaces a device (output) failure, and any omission is counted in an `Omitted` summary. Independently, the Playback observation carries `midi_failed` until a delivery succeeds. That sets the MIDI status, which shows as a Toast. Two things can make it look lost:
- The Toast fades after `[toasts] seconds` or is dismissed, but the Panel's **O** readout stays in the error colour while the failure stands.
- An identical repeated failure is latched and reported only once until it changes.
**Owner.** ADR 0055 (bounded retention, eviction by class), ADR 0090 (latch), ADR 0092 (`midi_failed` routing), ADR 0100 (Toasts, O readout). CONTEXT "Omitted Diagnostics", "Toast". Code: `console/src/playback_status.rs` (test `a_midi_failure_is_shown_until_the_observation_clears_it`).
**Gap.** ADR 0055 says the console's `MidiDeviceSelection::observe_diagnostics` shows the last failure. ADR 0092 later moved that routing to the Playback status, so 0055's Consequences section is stale.
**Confidence.** Medium to high. "Didn't show" has several possible causes.
**Discovery.** CONTEXT "Omitted Diagnostics" led to ADR 0055. Finding the display half took ADRs 0092 and 0100, found by filename scan. 5 files.

## Console

### C1. I want to ship a third built-in Theme. What must it pass, and which tests do I edit?
**Answer.** A built-in must pass three gates: the contrast gate, the colour-vision gate, and the check of each built-in against its table in `console/src/theme.md`. All three iterate the registry's built-ins, so adding the Theme brings it under them automatically. You edit no tests. You add:
- the definition;
- its reserved identity;
- its entry in the registry;
- its record (table) in `console/src/theme.md`.

It must be dark or light. Tests must not name it, because fixtures use `ThemeRegistry::built_in().fallback(appearance)`.
**Owner.** ADR 0082, ADR 0053 (Theme model and registry), ADR 0051. CONTEXT "Theme". Code: `console/src/theme_registry.rs`, `console/src/theme.rs`, `console/src/contrast.rs`, `console/src/contrast/colour_vision.rs`, `console/src/theme.md`.
**Gap.** I did not open the gate tests to confirm the exact contrast thresholds.
**Confidence.** High on the procedure, low on the numeric thresholds.
**Discovery.** CONTEXT "Theme" has no ADR citation. ADR filenames 0053 and 0082 matched. The `egui` skill presumably covers it too, but I did not open it. 3 files.

### C2. Drawing is slow. Can I cache each Cell's Paint facts on the Language Map?
**Answer.** No. That was ADR 0050: it was implemented, reviewed and rejected after benchmarks showed it moved too much work into Source-revision rebuilds. The accepted design (ADR 0052) is that the parser's Claim carries one fact, `written`, computed while a Render Frame builds its Claims. The Language Map keeps only parser Claims, with no Source bytes and no whole-Grid Paint view. A performance claim needs a benchmark (CLAUDE.md), and the comparison runs in CI.
**Owner.** ADR 0052 (supersedes 0050), ADR 0044, ADR 0040. Code: `orcvs/src/source/language_map.rs::LanguageMap::claims_by_cell`; `RenderCell::source_paint`.
**Gap.** None.
**Confidence.** High.
**Discovery.** ADR 0050's title, "the language map answers source paint per cell", matches the question exactly. Its status line says it was rejected. 2 files. Grepping only ADR titles would find the rejected ADR, not the accepted one.

### C3. How does a performer move around the Source on a laptop, and on a phone? Can they pinch to zoom?
**Answer.**
- **Laptop.** The Cursor moves by arrows, and Tab or Shift-Tab step a Sector along the row. Pan uses the wheel, a middle-drag, an Alt-drag, or a press or drag on the Minimap, and stops at the Grid's edges. A primary drag selects a Region.
- **Phone.** A finger's drag Pans, a tap selects a Cell, a long press of 0.8 s selects a Region, and a finger press opens the soft keyboard. The keyboard has no arrows, Tab or Escape.
- **Zoom.** Zoom is egui's whole-UI zoom in fixed steps. Pinch zoom is deliberately not supported: ADR 0058 retired it with the infinite-canvas design.
**Owner.** ADR 0045 (Pan gestures), ADR 0046 (drag selects), ADR 0058 (zoom; no pinch), ADR 0102 (touch), ADR 0084 (no margin). CONTEXT "Pan", "Zoom" (Avoid: pinch), "Sector". Code: `console/src/console/input.rs`, `console/src/grid_viewport.rs`.
**Gap.** On iOS Safari the keyboard can still cover the lower Cells (ADR 0102 consequence). I did not confirm the exact zoom key bindings.
**Confidence.** High.
**Discovery.** CONTEXT gives laptop Pan and Zoom. The phone half needed ADR 0102, found by filename, and a grep for "pinch". 4 files.

### C4. A MIDI clock failure shows as a MIDI problem. Where should it appear, and when does it clear?
**Answer.** It should appear under its own engine status, not MIDI's. One console Playback status routes each frame's diagnostics. Clock, start and retune failures are engine failures, shown as an `ENGINE` Toast. They are events, not states: the latest stays until the next run begins, and a newer one replaces it. A Rewind while playing begins a new run, which clears it. While any message stands, the **O** readout is drawn in the error colour.
**Owner.** ADR 0092, ADR 0100. Code: `console/src/playback_status.rs` (tests `an_engine_failure_is_shown_until_the_next_run_begins`, `a_rewind_while_playing_begins_a_new_run`).
**Gap.** If a clock failure still shows as MIDI, that contradicts ADR 0092, which was written to fix exactly that. The current code appears to route it correctly. The phrase "MIDI clock" is ambiguous: the engine's clock is not MIDI clock.
**Confidence.** High.
**Discovery.** The CONTEXT "Toast" entry names engine failures but cites no ADR. Filename scan found 0092. 3 files.

### C5. Typing into a value box in a menu wipes my Grid. Which controls should keep the keys?
**Answer.** Every focused control, plus any open popup or menu. This is deliberately not a named list. The Source gets keys only while `egui_wants_keyboard_input()` is false and no popup is open (`Popup::is_any_open`), using the previous frame's latched answer. This exact bug, a Theme menu Slider's value box passing command `A` and Backspace to the Source and emptying the Grid, is the case ADR 0048 records as fixed. If it happens, it is a regression.
**Owner.** ADR 0048. Tests named there: `a_focused_theme_menu_value_box_keeps_region_and_clipboard_commands_from_the_source`, `typing_with_a_menu_open_leaves_the_source_unwritten`. Code: `console/src/console/input.rs`.
**Gap.** None in the docs.
**Confidence.** High.
**Discovery.** The ADR filename "the source takes the keys no control holds" matched. CONTEXT has nothing on it. 2 files.

### C6. Can a phone visitor to the site play an example?
**Answer.** Yes:
- Every console, including the small one, shows the Panel's `P` (play/stop) and `R` (rewind) Cells, sized to tap. A phone has no Space key, and Space typed on Android Gboard writes nothing.
- The page's Built-in Synth sounds the example with no MIDI device, and a small or linked console picks it automatically.
- The audio context is resumed inside the tap gesture, as Safari requires.
- A finger Pans, and a tap opens the keyboard for editing.
**Owner.** ADR 0089, ADR 0087, ADR 0086, ADR 0102, ADR 0091 (phone config).
**Gap.** iOS: the keyboard can cover the lower Cells (ADR 0102).
**Confidence.** High.
**Discovery.** ADR 0089's opening states the phone problem. 3 files.

## Site

### S1. How do I send a friend a link that opens the Tracker example with sound?
**Answer.** Send `https://orcvs.com/?example=tracker`. `/` frames `/play/` with its query and holds the Built-in Synth, so the example sounds without MIDI. A bare `/play/?example=tracker` has no synth. The link opens stopped and never touches the friend's saved work, so they press `P` or Space to play. The audio starts on that gesture. From inside the console, the share icon `↗` copies exactly this link.
**Owner.** ADR 0094, ADR 0097. Code: `console/src/examples.rs::Example::link_name` (test `each_example_links_by_its_name`: `beat, melody, tracker, sequencer, knobs`).
**Gap.** ADR 0094 gives the example link as `?example=orca-tracker`, which is stale. The real link name is `tracker`, as in ADR 0097, the code, and the website skill.
**Confidence.** High.
**Discovery.** The ADR filename matched. The conflict between 0094 and 0097 sent me to the code. 4 files.

### S2. My branch changes `console/`. Why is the preview build stuck or failing with "was not published within 15 minutes"?
**Answer.** Every Cloudflare build, previews included, waits for the console archive keyed to its commit. `site/scripts/console.sh` retries for up to 15 minutes. A branch that changes the console must have its own `console-web.yml` run publish `console-preview-<key>.tar.gz`. If that run failed, never ran, or is still building, the preview waits and then fails. Runs are never cancelled, but a failed run publishes nothing. Also, if the branch is so far behind `main` that its key was pruned (only 100 `main` and 50 branch archives are kept), re-run Console web on the branch. A Dependabot branch takes the rolling console instead.
**Owner.** ADR 0101, amending ADR 0093 (the 15-minute wait). Code: `site/scripts/console.sh`, `.github/workflows/console-web.yml`.
**Gap.** The docs are stale. `AGENTS.md` says "The site builds around the console published from `main`", and the `website` skill says console.sh "downloads the build console-web.yml publishes from `main`". Both contradict ADR 0101's per-branch previews.
**Confidence.** Medium to high. I did not read `console.sh`.
**Discovery.** AGENTS.md and the website skill pointed the wrong way. Grepping for "15 minutes" found ADR 0093, whose status line points to 0101. It also produced one hit in the out-of-bounds `docs/requirements/`, which I ignored. 4 files.

### S3. On a phone the Cells are too small and the Minimap covers the Grid. Where do I change that?
**Answer.** In the page, not in Rust. `console/index.html` holds `<script type="application/toml" id="orcvs_config" data-media="(max-width: 600px)">`, which currently sets `[cell] width = 16, height = 24` and `[minimap] open = false`. The web console reads it with the same reader as native's `config.toml`, once at startup. The site gets this from `/play/` and passes nothing.
**Owner.** ADR 0091 (page settings), ADR 0102 (16×24 phone Cell), ADR 0096 (Minimap placement and sizing). Code: `console/index.html:127`; `console/src/config.rs`.
**Gap.** ADR 0091's body still says 24×36, and only its status line notes ADR 0102's change.
**Confidence.** High.
**Discovery.** ADR 0091's first paragraph describes this exact symptom. I confirmed in `index.html`. 3 files.

### S4. My MIDI device is connected, so why does an example on `/learn/` play through the Built-in Synth?
**Answer.** The example pages run the page's one small console (`/play/?small`). A Linked or small console always picks the Built-in Synth, whatever ports exist. Elsewhere, the synth is picked automatically only when no port exists. You can still choose your device under **O**, where the synth is listed after every MIDI port.
**Owner.** ADR 0087 ("A Linked or small console always picks it"), ADR 0086 (small console). CONTEXT "Built-in Synth".
**Gap.** I did not verify that `/learn/` specifically uses the small console. The website skill says doc pages do. ADR 0087's channel map for the kit (`08`–`0F`) is also stale after ADR 0099, which made `08` the acid channel; CONTEXT gives `09`–`0F`.
**Confidence.** Medium to high.
**Discovery.** The CONTEXT "Built-in Synth" entry ends with the rule. 3 files.

### S5. I want the page to send the console a new message, say a tempo. What must it respect?
**Answer.** It must follow the existing `postMessage` contract:
- Use a namespaced `{ type: "orcvs.<name>", … }`.
- Post only from the framing page, from the console's own origin. The console ignores other windows, other origins, unknown types and malformed fields.
- Send nothing before the console posts `orcvs.ready`.
- Only the latest message waiting for a frame takes effect.
- A message must not start Playback by itself. An `orcvs.open` Source arrives stopped.

Settings that are needed before the first frame cannot be messages (ADR 0091), but a runtime tempo can. A new message type is an embed contract, so record it in an ADR and in the website skill's contracts.
**Owner.** ADR 0086 (`orcvs.open`, origin rule, `orcvs.ready`), ADR 0087 (`orcvs.synth`, `orcvs.midi`, `orcvs.audio`, `orcvs.wake`), ADR 0091. `.agents/skills/website/SKILL.md` "Contracts the code cannot show".
**Gap.** No single document lists the message protocol. It is spread across ADRs 0086, 0087 and 0089. Whether a page may change tempo, which is performer state, is undecided.
**Confidence.** Medium. These are rules inferred from analogous messages.
**Discovery.** 4 files: ADR 0086, ADR 0087, ADR 0091 and the website skill.

### S6. On an iPhone the first note of a run comes late or is silent. What should happen?
**Answer.**
- The page resumes the Built-in Synth's audio context inside each pointer, touch or key gesture, because Safari resumes only during a gesture. Resuming on the first `orcvs.midi` is the fallback.
- The page requests media playback, so the ring/silent switch does not mute it.
- The console holds a start on the Built-in Synth until `orcvs.audio { running }` arrives, for at most 500 ms, so Tick 0 is not late.
- Each message plays at its Tick's planned time plus a fixed delay, targeted within 50 ms.

A late or silent first note means one of these failed.
**Owner.** ADR 0089 (gesture resume, media playback), ADR 0087 (Tick 0 waits ≤500 ms, fixed-offset timing).
**Gap.** None found in the docs. I did not check the site code (`site/src/console/*`).
**Confidence.** High.
**Discovery.** ADR 0087's status line points to 0089. 2 files.

---

## Summary of discovery problems

1. **AGENTS.md routes by tooling, not by topic.** It names `CONTEXT.md` and `docs/adr/` but gives no map from area (language, evaluation, playback, console, site) to ADRs. `docs/adr/README.md` covers only numbering. With 106 ADRs, nearly every question needed `ls` plus a grep over filenames and status lines.
2. **CONTEXT.md is the best single source but too large.** At 75 KB it is truncated by the reader. Its entries cite ADRs inconsistently: Live Editing and Defaulted Operand cite theirs, while Theme, Toast and Panel cite none. Console and site behaviour (key routing, previews, Output module, message protocol) is barely in it.
3. **Supersession chains need manual walking.** For example 0062 -> 0066 -> 0069 -> 0075 -> 0104 -> 0107 -> 0108, and 0050 -> 0052. Titles of superseded ADRs state the opposite of current rules ("a blank note is a rest", "a blank note is invalid", "the language map answers paint per cell"), so a filename search lands on the wrong decision first.
4. **Rules live in ADRs whose titles don't name them.** The safety action is in ADR 0016 ("distinct raw and timed play functions"), and the 15-minute wait is in ADR 0093.
5. **Some facts are stale in places that are not status lines:**
   - ADR 0094: `orca-tracker` (should be `tracker`).
   - ADR 0087: kit on `08`–`0F` (should be `09`–`0F`).
   - ADR 0091: 24×36 (should be 16×24).
   - ADR 0055: diagnostics routed to MidiDeviceSelection.
   - ADR 0067: "a Comment" is diagnosed input.
   - ADR 0033: `##`.
   - AGENTS.md and the website skill: the site console comes from `main`.
6. **Tests are the precise spec, but nothing routes to them.** The behavioural tests in `orcvs/src/source/tick/*.rs` (for example `blank_note.rs` and `movers.rs`) answered L3, E1 and E2 precisely, but no doc points to them. Only `docs/functions.md` names its test.
7. **Mover-versus-value-result interaction (E1) has no stated rule.** It must be inferred from ADR 0060 rule 3b and test names.
