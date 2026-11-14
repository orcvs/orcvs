# Language requirements

> Snapshot: checked against `c8cc197` (2026-11-14); incorporates ADRs through 0109.
> ADRs accepted after 0109 bind where they differ, until this document is updated.
> Source ADRs: 0008, 0010, 0011, 0013, 0015, 0018, 0019, 0021, 0024, 0033, 0035, 0043, 0049, 0054,
> 0061, 0063, 0067, 0069, 0070, 0071, 0072, 0073, 0074, 0103, 0105, 0108, 0109. Reviewed, not
> applicable or wholly superseded: 0005, 0007, 0017, 0023, 0025, 0026, 0029, 0030, 0039, 0062,
> 0075, 0104, 0107.

This document states what Source text means: the Grid and its Positions, how a row is parsed into
Language Units and Expressions, the literal types, operand binding (blank, optional, defaulted), the
Function families and the rules shared across them, nesting and Returns, and Chord building. It
does not state when Turns run, how Portals are ordered or how a Tick commits
([evaluation.md](evaluation.md)), what is sounded or sent ([playback.md](playback.md)), how the
console shows Source ([console.md](console.md)), or the site ([site.md](site.md)). Per-Function
spellings, operands, domains and examples are in [the Function reference](../functions.md). Terms
are defined in [CONTEXT.md](../../CONTEXT.md); decisions are argued in [the ADRs](../adr/).

## Source, Grid and Position

### LANG-010 A Cell holds one printable ASCII character, and Source Cells are the only stored program state

Every Cell holds one printable single-byte ASCII character, a space being an empty Cell; Function
names use standard ASCII. The Source is the only persistent program state; everything semantic is
derived from it.

Reason: one byte per Cell keeps editing a character Grid; Unicode is deferred.
Sources: [ADR 0008](../adr/0008-preserve-symbolic-arithmetic-functions.md), [ADR 0018](../adr/0018-derive-language-map-from-character-source.md).
Evidence: `orcvs/src/source/file.rs` `read`; test `orcvs/src/source/file.rs` `reading_refuses_every_byte_that_is_not_printable_ascii_or_space`; test `lang/src/parser.rs` `source_that_is_not_ascii_is_refused_rather_than_panicking`.

### LANG-020 Every Grid is 256 columns by 256 rows, and a position outside it does not exist

The Grid is part of the language and has one shape. Every pair `00 00`–`FF FF` names a Position;
there is an outside, and it is not a Position. What happens when a Function reaches the outside is
stated in [evaluation.md](evaluation.md).

Reason: a row is the whole horizontal extent there is, and a single shape means no edit to a Source
File can change the Grid it opens on.
Not: a Grid sized from its content, or a shape kept per Source.
Sources: [ADR 0043](../adr/0043-the-grid-is-bounded-by-definition.md), [ADR 0049](../adr/0049-a-position-is-two-numbers.md), [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md).
Evidence: `orcvs/src/grid.rs` `COL_COUNT`, `ROW_COUNT`, `Grid::new`; tests `orcvs/src/grid.rs` `test_every_grid_is_as_large_as_two_numbers_address`, `test_a_persisted_grid_of_any_shape_but_the_one_is_refused` (assertions inspected).

### LANG-030 A Position is two Numbers, column then row, minted only by its Grid

A Position is spelled as two Numbers, column first, each `00`–`FF`, counted in Cells from `00 00`
at the top-left. A Position is obtained only from the Grid that contains it, and the Grid alone
converts between a Position and the Cell index the Source addresses; a Cell index and a Language
Unit index are distinct types.

Reason: a Number already spends two Cells, so every Position a Source holds has a spelling the Source
can write.
Not: base-36 single-glyph axes, or a limit on addressing only.
Sources: [ADR 0049](../adr/0049-a-position-is-two-numbers.md), [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md), [ADR 0024](../adr/0024-the-language-map-records-spellings-not-atom-types.md).
Evidence: `orcvs/src/grid.rs` `Position`, `CellIndex`, `Grid::position`; tests `orcvs/src/grid.rs` `test_grid_mints_positions_inside_it`, `test_grid_refuses_positions_outside_it`, property `a_position_exists_exactly_where_the_grid_has_a_cell`; test `orcvs/tests/language_map.rs` `token_at_refuses_a_position_minted_by_another_grid`.

### LANG-040 A Source File states Cells, never a shape

Line *n* is row *n* and character *m* is column *m*; a short line, a missing line or stripped
trailing whitespace reads back as empty Cells. Text past column 256 or row 256 is not a Source File.

Reason: a header or inferred shape would be the first thing a hand edit breaks.
Sources: [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md).
Evidence: `orcvs/src/source/file.rs` `read`, `write`; tests `orcvs/src/source/file.rs` `reading_treats_short_and_missing_lines_as_empty_cells`, `reading_refuses_a_257th_line`, `reading_refuses_a_257th_character`.

## Parsing a row

### LANG-050 The Language Map is derived from each Source revision by the Parser

One Language Map is derived per revision, on the Grid's own Positions, with no second grid or
coordinate system. It takes its units and their kinds from the Parser's positioned Expressions and
records no persistent type for a literal: a `**` in a Number slot is invalid input, not a Bang
unit. Units ascend strictly by anchor in row-major order.

Reason: the Parser alone interprets spellings, so the map cannot disagree with it.
Not: an independent spelling classifier.
Sources: [ADR 0018](../adr/0018-derive-language-map-from-character-source.md), [ADR 0024](../adr/0024-the-language-map-records-spellings-not-atom-types.md).
Evidence: `orcvs/src/source/language_map.rs`; `lang/src/parser.rs` `Parser::analyze`; tests `orcvs/src/source/language_map.rs` `language_map_builds_cohesive_expression_state`, `invalid_operand_bang_spellings_are_not_parsed_bang_values`.

### LANG-060 A row is partitioned left to right into non-overlapping Language Units by parse

The row walk hands the Parser the remaining Cells; the Parser reports the Cells its Expression
occupies and the walk resumes after them. A Cell belongs to at most one unit. A spelling the
Function table does not hold costs one Cell and the walk resumes at the next, so `***` is a Bang
Function and one invalid `*`. Incomplete or invalid Source is valid content that forms no Language
Unit. Empty Cells between Expressions are skipped silently.

Reason: partition by parse removes every reconciliation between whitespace runs and arity claims.
Not: the whitespace run as the unit of partition.
Sources: [ADR 0018](../adr/0018-derive-language-map-from-character-source.md), [ADR 0033](../adr/0033-partition-a-row-by-parse.md).
Evidence: `lang/src/parser.rs` `take_language_unit`; test `lang/src/parser.rs` `an_unrecognized_function_consumes_one_cell_and_records_one_invalid_slot`; tests `orcvs/src/source/language_map.rs` `language_map_partitions_complete_units_left_to_right_without_overlap`, `an_odd_standalone_run_costs_one_cell_and_leaves_the_rest_readable`.

### LANG-070 An Expression claims exactly what its arity declares, within one row

A Function claims its spelling plus each operand its signature declares, in signature order; a
nested Function fills the slot it stands in and extends the claim by its own operands. Inside a
claim an empty Cell is a character, so `.+01 02` is refused at its second operand. No claim crosses
a row edge: an operand slot running past it, or a spelling with one Cell left, is refused. A long
Expression stays one Expression; there is no record limit. The bound on the Operand Stack is stated
in [evaluation.md](evaluation.md).

Reason: arity is the boundary, so spacing is taste between Expressions and meaning inside one.
Sources: [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md), [ADR 0033](../adr/0033-partition-a-row-by-parse.md), [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md).
Evidence: `lang/src/parser.rs` `Parser::analyze`; tests `orcvs/src/source/language_map.rs` `a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary`, `language_map_never_forms_a_unit_across_a_row_edge`, `a_partly_written_or_edge_cut_operand_is_diagnosed`; test `orcvs/tests/language_map.rs` `truncated_operand_owns_the_available_row_tail`.

### LANG-080 A Comment opens at `||` where an Expression could start and closes at the next `||` on its row

The closer is the leftmost `||` after the introducer; with none the Comment claims the rest of the
row. `|||` is unclosed and `||||` is an empty closed Comment. After a closed Comment the walk
resumes and the Source there parses. Inside a Function's arity claim `||` is an operand Cell that
fails to bind. One `|` alone is invalid Source.

Reason: one delimiter that closes as well as opens lets a pattern be framed beside the Functions
that read it, as Orca's `#` does.
Not: `!!` (a sigil's single form) or `//` (`/` is the octave-below-zero marker).
Sources: [ADR 0035](../adr/0035-spell-the-comment-as-a-parser-unit.md), [ADR 0074](../adr/0074-a-comment-closes-at-the-next-introducer.md).
Evidence: `lang/src/parser.rs` Comment arm; tests `lang/src/parser.rs` `a_comment_ends_at_the_next_comment_introducer`, `a_comment_introducer_does_not_close_itself`, `a_run_of_pipes_closes_a_comment_at_its_first_two`, `a_comment_introducer_inside_an_operand_claim_is_a_refused_operand`, `a_lone_vertical_rule_is_not_a_comment` (assertions inspected).

### LANG-090 A Comment is never evaluated, and its text is ordinary Cells

A Comment records a Token and no Atom: it answers no value, performs no effect and is never
scheduled. Only its two delimiters are its occupied Cells. Its text is never parsed and is ordinary
Cells to every Function: a Read, Track or Copy selecting a pair of it reads it as a pair no unit
covers; a pair touching a delimiter is partial input; a Write writes the text; a Halt aimed at the
text locks nothing; a mover is refused there only by what the Cells hold. A write that overwrites a
closer obscures those Cells until the next Snapshot re-parses the row.

Reason: the text is data, so the tracker idiom of reading a commented pattern works.
Sources: [ADR 0035](../adr/0035-spell-the-comment-as-a-parser-unit.md), [ADR 0074](../adr/0074-a-comment-closes-at-the-next-introducer.md).
Evidence: `lang/src/parser.rs`; `orcvs/src/source/portal.rs`; tests `orcvs/src/source/tick/read.rs` `a_pair_of_a_comments_text_reads_as_data` (assertions inspected), `a_comment_closer_at_the_selected_pair_diagnoses`; `orcvs/src/source/tick/track.rs` `track_reads_a_pair_of_a_comments_text`; `orcvs/src/source/tick/write.rs` `a_write_into_comment_text_writes_its_cells`; `orcvs/src/source/tick/halt.rs` `a_halt_aimed_at_comment_text_locks_nothing_and_does_not_diagnose`; `orcvs/src/source/portal.rs` `occupancy_of_a_comment_is_its_delimiters_and_not_its_text`.

## Literals and Atoms

### LANG-100 A Number is two uppercase hexadecimal Cells, `00` to `FF`

Every Number is one unsigned byte spelled by exactly two uppercase hexadecimal Cells. Narrower
domains, such as MIDI's, are checked by the Functions that need them.

Reason: two Cells buy byte alignment, tracker notation and a fit with MIDI data.
Not: Orca's base 36.
Sources: [ADR 0010](../adr/0010-two-cell-hexadecimal-numbers.md).
Evidence: `lang/src/lib.rs`; `lang/src/atom.rs` `number_atom_from_spelling`; tests `lang/src/lib.rs` `str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings`, `every_number_has_one_canonical_two_cell_source_encoding`.

### LANG-110 A Note is a pitch letter and an octave Cell, from `C/` to `G9`

Uppercase letter for a natural, lowercase for its sharp, then `/` for the octave below `0` or `0`–`9`;
every Note is MIDI note `00`–`7F`.

Reason: a contextual two-Cell pitch spelling sits beside the hexadecimal Number spelling, told apart by
the operand position.
Sources: [ADR 0021](../adr/0021-convert-numbers-and-notes-explicitly.md).
Evidence: `lang/src/atom.rs` `Note`, `note_atom_from_spelling`; tests `lang/src/lib.rs` `midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range`, `note_source_encoding_covers_the_documented_boundaries`.

### LANG-120 An Operand Literal takes its type from the slot that receives it

The same two Cells are a Number in a Number slot, a Note in a Note slot, an Interval Set or Voicing
where those are declared: `C4`, `69`, `13`, `07`, `09`, `d7`, `D2`, `23` and `24` each read as the
receiving position decides. Source keeps characters, never a type; nothing coerces globally between
types; a Note operand never changes another Function's arithmetic domain.

Reason: a literal that recorded the type a Function once read it as would be type provenance, which
Source does not carry.
Sources: [ADR 0021](../adr/0021-convert-numbers-and-notes-explicitly.md), [ADR 0024](../adr/0024-the-language-map-records-spellings-not-atom-types.md), [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/atom.rs`; tests `lang/src/atom.rs` `a_name_that_also_spells_a_number_or_note_is_read_by_the_position_receiving_it`; `orcvs/src/source/tick/driven_chord.rs` `the_same_cells_mean_what_each_operand_receiving_them_reads`; `lang/src/parser.rs` `conversion_literal_operands_are_monomorphic`.

### LANG-130 A standalone Operand Literal is invalid Source

A pair that spells a Number or a Note where an Expression could start has no slot to type it and is
diagnosed; the Parser refuses it as an unknown spelling. This includes Cells a Function wrote on an
earlier Tick and the pairs a Track or Read selects from. A reversal is pending; see Open decisions.

Reason: a literal's type exists only in the operand position that receives it.
Sources: [ADR 0021](../adr/0021-convert-numbers-and-notes-explicitly.md).
Evidence: `lang/src/parser.rs` `take_language_unit`; test `lang/src/parser.rs` `an_operand_literal_outside_a_slot_is_not_an_expression` (assertions inspected); `orcvs/src/source/model.rs` assertion "a lone Number reads as an unknown Function on the next Tick" (assertions inspected).

### LANG-140 An Interval Set is one literal type of 39 two-Cell names, case-sensitive

An Interval Set names semitones above a root: 25 chords and 14 scales, listed in
[the Function reference](../functions.md#interval-sets). Every name fills both Cells and a half-blank
pair is partial input. Names are case-sensitive and nothing folds case. A scale name is valid
wherever a chord name is.

Reason: one type lets Pick and Stack serve chords and scales alike.
Not: separate chord and scale types; half-blank names.
Sources: [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/atom.rs` `define_interval_sets!`, `IntervalSet`; tests `lang/src/atom.rs` `every_interval_set_name_decodes_to_its_semitones`, `an_unknown_or_half_blank_name_is_refused_by_both_literal_types`.

### LANG-150 A Voicing is a separate literal type of eight names

`CL` close, `OP` open, `D2`, `D3`, `23`, `24` drops, `SH` shell and `RL` rootless. A single drop keeps
its `D`; a double drop names both positions, counted from the top. Inversion is a Number operand, not
part of a Voicing.

Reason: a drop is named for the positions it lowers, as jazz texts list them.
Sources: [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/atom.rs` `define_named_literal!`; tests `lang/src/atom.rs` `every_voicing_name_decodes_and_spells_itself_back`; `lang/src/functions/chord.rs` `every_voicing_gives_the_tones_it_names`.

### LANG-160 Names are unique across Functions, Interval Sets and Voicings, and no two differ only by look-alike glyphs

No spelling names two things across the three tables. No name holds the filler `_`, and no two
names differ only by `0` against `o`; the build refuses a table that breaks either. Names that
differ by one meaningful Cell (`M7`/`m7`, `mM`/`mm`, `23`/`24`) are accepted, and a slip between
them is a valid different name, not diagnosed.

Reason: two Cells hold few names, so the names musicians read win over slip detection.
Sources: [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/atom.rs` `const _` assertions, `shares_a_spelling` (assertions inspected); test `lang/src/atom.rs` `an_interval_set_and_a_voicing_refuse_each_others_names`.

### LANG-170 An Atom is a Number, a Note, an Interval Set, a Voicing, a Bang or a Function

Every Atom has a Source encoding. A Clear is not an Atom.

Reason: an Atom is what an operand resolves to and what a value Function answers when it has a
Language Unit to write.
Sources: [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md).
Evidence: `lang/src/atom.rs` `enum Atom`; test `lang/src/parser.rs` `every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it`.

## Function families

### LANG-180 A Function spelling puts its family glyph first and its operation glyph second

The families are `.` numeric, `~` Tick and feedback, `*` activation, `&` read, `@` write, `=` copy,
`#` pitch and `!` terminal output; the operandless units are doubled glyphs (`**`, `^^ vv << >>`,
`||`). An operation glyph may recur in another family, because the prefix names the behaviour first
(`~+` against `.+`, `$` in `!$` and `&$`). The `!` family holds every Function that sends an
outbound effect and answers no value: MIDI, UDP, OSC and Application Command. Per-Function
spellings are in [the Function reference](../functions.md).

Reason: a reader identifies the evaluation family before the specific operation.
Sources: [ADR 0008](../adr/0008-preserve-symbolic-arithmetic-functions.md), [ADR 0035](../adr/0035-spell-the-comment-as-a-parser-unit.md), [ADR 0063](../adr/0063-a-list-is-cells-in-a-claim-not-a-value.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md); `#` family ruled 2026-11-10, no ADR (`.scratch/chords-and-pitch/spec.md`).
Evidence: `lang/src/atom.rs` `define_functions!`; tests `lang/src/atom.rs` `pitch_functions_display_with_the_hash_family_spellings`, `arithmetic_functions_display_with_the_dot_family_spellings`; `lang/src/parser.rs` `no_function_is_spelled_in_the_colon_family`, `every_real_function_parses_and_renders_from_its_canonical_spelling`.

### LANG-190 `&` reads Source, `@` writes it and `=` copies it, and an arrow names the Cell acted on

A Read reads the pair on its arrow's side, a Write writes it, and a Copy writes it and reads the
opposite side, leaving what it read. The same arrow means the same direction in every family:
`*^`, `^^`, `=^`, `&^` and `@^` all point north. Each action has a directional form, an absolute
`$` form and, for Read and Write, an indexed form (Track `&t`, Push `@t`); Copy has no indexed form.

Reason: the prefix tells a reader which ordering rule applies to the Function's Turn.
Not: reads and writes sharing `@`; letter glyphs for directions.
Sources: [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: `lang/src/atom.rs` rows `Read*`, `Write*`, `Copy*`, `Function::direction`; tests `lang/src/parser.rs` `the_ampersand_arrows_parse_as_reads_of_one_number`; `lang/src/atom.rs` `every_function_names_its_input_portal`.

### LANG-200 A directional distance counts Portals from its `distance` operand

`00` is the `distance` operand itself; each step is one pair east or west, or one row north or south
in the operand's columns: `&>00C4` answers `00`, `&>01C4` answers `C4`. A nested `distance` counts
from the slot it occupies, its own spelling and operands included. A Write counts as the Read with
the same arrow; a directional Copy steps one Portal. Track counts its pairs from the end of its last
operand, nested Functions included.

Reason: a distance is a measure from the operand, so it does not move when a nested Function's width
changes.
Not: counting from the anchor, or in Cells.
Sources: [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: `lang/src/atom.rs` `Function::distance`; tests `orcvs/src/source/tick/read.rs` `a_distance_counts_portals_from_the_distance_operand`, `every_read_at_distance_zero_answers_its_own_operand`, `a_nested_distance_counts_from_the_slot_it_occupies`.

### LANG-210 An absolute form addresses a Position as two Number operands, named as one Role

`&$ at`, `@$ at value` and `=$ from to` take each Position as two Number operands, column then row.
Each Position is one Role. A pair starting at column `FF` is cut short by the row edge and
diagnoses.

Reason: `$` marks an absolute reference, as in a spreadsheet; a pair of Numbers is one thing to a
reader.
Sources: [ADR 0049](../adr/0049-a-position-is-two-numbers.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: `lang/src/atom.rs` `Function::position`, `RoleGroup`; tests `orcvs/src/source/tick/read.rs` `an_absolute_read_of_column_fe_reads_and_of_column_ff_is_cut_short`; `lang/src/atom.rs` `a_position_grouped_as_one_role_is_accepted`.

### LANG-220 Every Copy, Read and Track reads its pair by one set of rules

Two empty Cells are copied: the Function answers a Clear. `**` answers a Bang. A Function spelling
answers that Function. A Number, Note, Interval Set or Voicing answers that Atom, decoded later by
whatever receives it. Anything else, such as a partial pair, a Comment delimiter or Cells straddling
two Language Units, is diagnosed as partial or invalid input and nothing is written. A Function that
passes a Bang through writes `**` over its complete destination. No reader has rules of its own.
When each reader's Turn comes is stated in [evaluation.md](evaluation.md).

Reason: Reads and Writes pass a pair through with the least ceremony possible.
Sources: [ADR 0067](../adr/0067-track-reads-a-portal-as-jump-does.md), [ADR 0069](../adr/0069-an-unwritten-operand-is-invalid-input.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md), [ADR 0071](../adr/0071-a-bang-travels-as-its-glyph.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md), [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/functions/copy.rs`, `read.rs`, `track.rs`; tests `orcvs/src/source/tick/read.rs` `a_read_answers_what_a_copy_answers_for_the_cells_it_reads`; `orcvs/src/source/tick/carried_names.rs` `a_read_answers_each_name_unchanged`, `a_read_or_track_diagnoses_a_half_written_name_and_writes_nothing`; `orcvs/src/source/tick/track.rs` `all_empty_pairs_clear_the_output_portal_without_a_diagnostic`.

### LANG-230 A Write's or Push's `value` takes no type and carries its pair unchanged

The destination's reader decodes it. A Function spelling typed into `value` is a nested Function, so
`@$0000.+0101` writes `02`; writing a spelling itself takes a nested Read of it. A `**` in `value`
is written as a Bang each Turn. An empty `value` makes the Write invalid. Orca's Generator is a Read
nested in a Write's `value`.

Reason: a typed `value` would refuse Notes, Function spellings or Bangs and stop Writes composing.
Not: a Number, a Note, or a `value` that tries decodings in turn.
Sources: [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md), [ADR 0071](../adr/0071-a-bang-travels-as-its-glyph.md).
Evidence: `lang/src/atom.rs` `Untyped`; tests `lang/src/parser.rs` `a_writes_value_slot_carries_any_language_unit_a_literal_spells`; `orcvs/src/source/tick/carried_names.rs` `a_write_carries_each_name_to_its_pair_unchanged`, `a_push_carries_each_name_to_its_pair_unchanged`.

### LANG-240 `**` is the Bang Function where an Expression starts, the Bang value in an untyped operand, and invalid in a typed one

A standalone `**` is the Bang Function: no operands, root only, intrinsically active, able to emit a
Bang. What its Turn does is stated in [evaluation.md](evaluation.md).

Reason: a standalone Bang follows the rules every Function follows, so Source alone decides whether
it fires.
Sources: [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0024](../adr/0024-the-language-map-records-spellings-not-atom-types.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: `lang/src/parser.rs` root-only `**` arm; `lang/src/atom.rs` `Bang` row; tests `lang/src/parser.rs` `bang_and_self_banging_functions_parse_as_complete_language_units`, `a_bang_in_a_typed_operand_is_still_invalid_syntax`; `orcvs/src/source/language_map.rs` `a_bang_outside_every_claim_is_a_bang_however_the_source_around_it_reads`.

### LANG-440 The cells a Track or Read selects from, and a Push's lane, are ordinary Source

Track, Read and Push claim only their operands. The Cells they read or write are parsed, painted and
edited as any other Source, and a Function standing in a Push lane takes its Turn.

Reason: the pair read is whatever working Source holds there; no second reader owns it.
Not: a List in the Function's claim; Orca's locked Push lane.
Sources: [ADR 0067](../adr/0067-track-reads-a-portal-as-jump-does.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: `lang/src/atom.rs` rows `Track`, `Push`; tests `orcvs/src/source/tick/push.rs`, `orcvs/src/source/tick/track.rs`.

### LANG-460 Application Command `!$` reaches only commands the host application provides

It never invokes an operating-system shell or arbitrary executable. Its command encoding is
deferred until Orcvs defines a text or message value.

Reason: Orca's `$` capability without process execution.
Sources: [ADR 0008](../adr/0008-preserve-symbolic-arithmetic-functions.md), [ADR 0019](../adr/0019-map-orca-capabilities-onto-orcvs-language-families.md).
Evidence: no `!$` row in `lang/src/atom.rs`; `console/src/function_reference.rs` module doc.

## Operand binding

### LANG-280 An operand slot that requires a value and holds none makes its Function invalid

A slot whose Cells are all empty is invalid input exactly as a partly written one is, whether the
Source was written that way, a Portal write emptied it, or a nested Function answered a Clear. The
Function does not evaluate, writes nothing, returns nothing and is diagnosed; a nested invalid
Function fails its parent. Pending is not a state. Optional and defaulted slots (LANG-300,
LANG-320) are the exceptions, by type.

Reason: a third outcome beside evaluating and being invalid would have to be carried by the Language
Map, the schedule and every Turn.
Not: an empty operand read as zero, as in Orca.
Sources: [ADR 0069](../adr/0069-an-unwritten-operand-is-invalid-input.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md), [ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md).
Evidence: `lang/src/operand.rs` `Operand::ABSENT`; tests `lang/src/operand.rs` `a_plain_operand_still_refuses_a_blank_slot`; `orcvs/src/source/tick/clear.rs` `a_nested_function_invalid_on_an_unwritten_slot_fails_its_parent`; `lang/src/functions/mod.rs` `a_plain_operand_refuses_an_absent_value_as_it_refuses_a_blank_slot`.

### LANG-290 An Expression with an unbound required operand is diagnosed and is not a root

The Language Map diagnoses it from Source alone and keeps it out of the roots; an Expression whose
only blank slots are optional or defaulted is a root.

Reason: only a complete Expression is scheduled.
Sources: [ADR 0069](../adr/0069-an-unwritten-operand-is-invalid-input.md), [ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md).
Evidence: `orcvs/src/source/language_map.rs` roots; tests `orcvs/src/source/language_map.rs` `an_expression_waiting_on_unwritten_operands_is_diagnosed_and_not_a_root`, `a_play_whose_only_blank_operand_is_its_note_is_a_root` (assertions inspected), `a_play_with_anything_else_unbound_is_diagnosed_and_not_a_root`.

### LANG-300 An optional operand binds a slot holding no value to no value; a Play's note is optional

`Optional<O>` reads a present value as `O` does and refuses what `O` refuses, with `O`'s
diagnostic; a blank or cleared slot binds to no value. The note of Raw Play `!>`, Timed Play `!~`
and Monophonic Play `!%` is `Optional<Note>`: with no note, the Play sends nothing and reports
nothing, by any route. A half-typed or mistyped note is diagnosed. Every other operand still binds
and is diagnosed as when a note is present. Reading and taking the Turn of a blank-note Play
allocate nothing. A Chord Play's root is a plain Note.

Reason: "a blank note plays nothing" is what a Play's note is, not an exception each module spots.
Not: a Rest; each Function deciding what its empty slots mean.
Sources: [ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md).
Evidence: `lang/src/operand.rs` `Optional`; `lang/src/atom.rs` rows `RawPlay`, `TimedPlay`, `MonophonicPlay`; tests `lang/src/parser.rs` `a_blank_raw_play_note_is_recorded_absent_and_reports_nothing` (assertions inspected); `orcvs/src/source/tick/blank_note.rs` `a_raw_play_whose_note_a_portal_write_emptied_plays_nothing_and_reports_nothing`, `a_raw_play_with_a_blank_note_still_diagnoses_its_channel_and_velocity`; `lang/tests/allocation.rs` `reading_and_taking_the_turn_of_a_play_with_a_blank_note_allocates_nothing`.

### LANG-310 Whether a slot admits no value comes from its declared operand type alone

The Function table derives each slot's admission from its operand type; there is no separate list.
The Parser reads a blank slot as absent only where the signature admits one and diagnoses a blank
required slot before any Tick. The Parser, the Language Map and the Tick carry an absent operand
without interpreting it; only binding gives it meaning. Making an operand optional or defaulted is a
change to its declared type.

Reason: one declaration keeps the Parser, the map, the Tick and the binder from drifting apart.
Sources: [ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md), [ADR 0109](../adr/0109-a-blank-operand-can-stand-for-a-fixed-value.md).
Evidence: `lang/src/atom.rs` `blank_admitted`; `lang/src/operand.rs` `ABSENT`; test `lang/src/atom.rs` `every_blank_a_function_admits_binds`.

### LANG-320 A defaulted operand binds a slot holding no value as its fixed Number

`Defaulted<O, BLANK>` binds a blank or cleared slot exactly as `BLANK` written there, through `O`'s
own bind, so the Function cannot tell the two apart; a written value binds as `O` binds it. The
Function table row states `BLANK` as the operand type's second parameter. Only a Number has a fixed
value. A `lang` test binds every declared blank and fails on one `O` refuses. Neither `Optional` nor
`Defaulted` wraps an operand that already admits a blank; the build refuses it.

Reason: the Function body never carries its own copy of what blank means.
Not: a default applied in the body; `ABSENT` returning a `Result`.
Sources: [ADR 0109](../adr/0109-a-blank-operand-can-stand-for-a-fixed-value.md).
Evidence: `lang/src/operand.rs` `Defaulted`; `lang/src/atom.rs` `legato: Defaulted<Number, 0x00>`; tests `lang/src/operand.rs` `a_blank_defaulted_slot_binds_its_fixed_value`, `a_fixed_value_outside_the_domain_fails_the_table_check_as_if_written`; compile-fail doctests in `lang/src/operand.rs`; `orcvs/src/source/tick/blank_note.rs` `a_nested_clear_under_a_bang_driven_output_binds_absent_in_a_note_and_as_00_in_legato`.

## Answers, nesting and Returns

### LANG-330 A value Function answers a Language Unit or a Clear, and a Clear writes two spaces

A Clear writes two spaces at the Output Portal, root or nested, and is not an Atom: no operand holds
it. Equality, Every and Euclidean answer a Clear on every Turn they do not Bang, so each writes its
Output Portal on every Turn. A Copy, Read or Track answers a Clear for two empty Cells. A nested
Clear leaves its slot unwritten: where the slot requires a value the parent is invalid and its
diagnostic names the child (`nested computation at column X, row Y cleared its operand`); an
optional slot binds no value, a defaulted slot its fixed Number.

Reason: as Orca's bang ports write `.`, so a pulse erases its last `**`.
Not: an Absence Marker value.
Sources: [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md), [ADR 0071](../adr/0071-a-bang-travels-as-its-glyph.md), [ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md), [ADR 0109](../adr/0109-a-blank-operand-can-stand-for-a-fixed-value.md).
Evidence: `orcvs/src/source/tick/execution.rs`; tests `lang/src/interpreter.rs` `only_the_pulses_and_the_functions_that_copy_cells_answer_a_clear`; `orcvs/src/source/tick/clear.rs` `a_nested_equality_that_does_not_bang_leaves_its_parent_operand_unwritten` (assertions inspected), `every_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang`.

### LANG-340 A Function with nothing to write this Tick answers nothing, and its last write stands

A Function that takes no Turn, or is invalid, writes nothing; a Cell it wrote earlier keeps its
characters until something overwrites it.

Reason: an invalid Function that cleared its Output Portal would wipe the Cells under every mistake
and lose Increment's and Slew's state.
Sources: [ADR 0069](../adr/0069-an-unwritten-operand-is-invalid-input.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md).
Evidence: `orcvs/src/source/tick/execution.rs`; test `orcvs/src/source/tick/clear.rs` `an_unwritten_operand_in_the_source_blocks_its_function_as_the_map_reports`.

### LANG-350 Orcvs has no series value

No Function accepts or answers a series of values, and nothing extends over one. A pattern is written
in Source Cells; a chord is a Chord Play or several Play roots one Bang activates.

Reason: a Return is one two-Cell encoding, and a pattern the performer edits should be visible.
Sources: [ADR 0063](../adr/0063-a-list-is-cells-in-a-claim-not-a-value.md), [ADR 0103](../adr/0103-one-chord-play-sounds-every-tone-of-its-chord.md).
Evidence: tests `lang/src/parser.rs` `no_function_is_spelled_in_the_colon_family`; `orcvs/src/source/language_map.rs` `a_retired_colon_spelling_reserves_nothing`.

### LANG-250 Only a Function that answers a value can be nested, and the Parser refuses any other from Source alone

Each Function declares whether it answers a value or performs an effect ([evaluation.md](evaluation.md)
states the declaration). A Function that answers no value has no Return, so the Parser refuses it in
an operand slot, whatever its operands hold. The Bang Function is the one effect Function no slot
reads as a Function: there `**` is the Bang value (LANG-240).

Reason: the refusal reads the declaration, not a spelling or family prefix, so the Language Map
marks it from any revision.
Sources: [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md).
Evidence: `lang/src/parser.rs` `SyntaxError::NestedEffectFunction`; `lang/src/atom.rs` `answers_value`; tests `lang/src/parser.rs` `a_nested_function_that_answers_no_value_is_refused_from_source_alone`, `a_nested_play_with_a_blank_note_is_refused_as_nested` (assertions inspected).

### LANG-260 A nested Function's Return is its two-Cell encoding, decoded by the receiving operand's type

The receiving slot decodes a Return exactly as it decodes Cells a Portal wrote there, so the child's
Atom type does not cross: a Note `C5` returned into an Addition operand is the Number `C5`, and
`.+.+010102` behaves as if `02` were written into the outer first operand.

Reason: nesting and spatial composition deliver Source encoding one way.
Not: a typed nested result.
Sources: [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0021](../adr/0021-convert-numbers-and-notes-explicitly.md).
Evidence: `orcvs/src/source/encoding.rs`; tests `orcvs/src/source/tick/nesting.rs` `a_return_and_a_portal_decode_the_same_characters_the_same_way` (assertions inspected); `lang/src/functions/numeric_conversion.rs` `nested_conversions_compose_and_read_their_return_by_the_receiving_operand`.

### LANG-270 A nested Function also writes through its own Output Portal

The Return is an additional delivery, not a redirection, so the row below a nested Expression traces
every step and feedback Functions keep their state when nested. A refused Output Portal write does
not withdraw the Return, and a parent's failure does not undo the child's write. A nested Function
that answers a Bang writes `**` through its own Output Portal, activating as a root's would, and its
Return is `**`. How nested writes are ordered and reserved is stated in [evaluation.md](evaluation.md).

Reason: Source is the only place a feedback Function can keep its previous value.
Not: a Return that suppresses the nested write.
Sources: [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0067](../adr/0067-track-reads-a-portal-as-jump-does.md).
Evidence: `orcvs/src/source/tick/execution.rs`; tests `orcvs/src/source/tick/nesting.rs` `nested_feedback_functions_keep_their_state_in_their_own_output_portals`, `a_nested_bang_writes_its_bang_and_activates_its_aligned_roots`, `a_child_write_survives_its_parents_failure`; `orcvs/src/source/tick/live_editing.rs` `live_child_write_survives_parent_failure_and_rejected_portal_keeps_its_return`.

## Numeric, pitch and random Functions

### LANG-360 General arithmetic takes Numbers and wraps over the byte

Add, Subtract and Multiply wrap modulo 256 rather than saturating. Arithmetic accepts no Note and
infers no result type; Note-shaped Cells in a Number slot read as the Number they spell, and a
spelling that is no Number diagnoses. Division and Modulo by zero diagnose and produce no value.
Equality answers a Bang or a Clear. Per-operation detail is in
[the Function reference](../functions.md#arithmetic).

Reason: a byte is the domain; a zero divisor has no cyclic meaning.
Sources: [ADR 0011](../adr/0011-general-arithmetic-wraps-over-bytes.md).
Evidence: `lang/src/functions/math.rs`; tests `lang/src/interpreter.rs` `general_arithmetic_wraps_for_every_pair_of_bytes`, `general_arithmetic_rejects_note_operands`, `division_is_the_asymmetry_that_diagnoses_every_zero_divisor`, `modulo_by_zero_diagnoses_distinctly_from_division_by_zero`.

### LANG-370 Number and Note convert only through `.v` and `.^`, which read one fixed type each

`.v` reads a Note and answers its MIDI number; `.^` reads a Number and answers its Note, diagnosing
`80`–`FF`. Each reads its operand only as its declared type, however the Cells arrived, so
`.v.^3C` composes while `.v.vC4` and `.^.^3C` diagnose. A conversion never passes through a value of
its result type.

Reason: movement between Number and Note is always explicit.
Sources: [ADR 0021](../adr/0021-convert-numbers-and-notes-explicitly.md), [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md).
Evidence: `lang/src/functions/numeric_conversion.rs`; tests `conversion_to_note_refuses_every_number_above_the_range`, `each_conversion_refuses_a_value_of_the_type_it_answers`; `orcvs/src/source/tick/nesting.rs` `a_return_and_a_portal_decode_the_same_characters_the_same_way` (assertions inspected).

### LANG-450 Pitch Functions derive Notes and Interval Sets, and a Note answer never wraps or clamps

The `#` family holds the stateless value Functions that derive a Note or an Interval Set: Up `#+`,
Down `#-`, Pick `#p` and Stack `#s`; `.` stays numeric. They run every Tick, root or nested, and
write their answer through their Output Portal as every value Function does. A Note answer outside
`00`–`7F` diagnoses, naming the Function and the semitone it would reach, and writes nothing. Pick
answers `root + set[index % n] + 12 * floor(index / n)`. Stack answers the Interval Set its tones
form and diagnoses when no name holds exactly those semitones. Operands and examples are in
[the Function reference](../functions.md#pitch).

Reason: a Function deriving pitches belongs to a family whose prefix says so, and a wrapped pitch
would sound the wrong note.
Sources: ruled 2026-11-10, no ADR (`.scratch/chords-and-pitch/spec.md`); Stack named by [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/functions/pitch.rs`; tests `up_diagnoses_an_answer_above_the_note_range_rather_than_wrapping`, `pick_wraps_an_index_past_the_end_of_the_set_up_an_octave`, `stack_diagnoses_when_no_name_fits_rather_than_answering_a_near_one`, `pick_and_stack_nest_and_compose`.

### LANG-380 Random is determined by its seed, the absolute Tick and its own Position

`~? seed minimum maximum` answers a Number between its bounds inclusive; reversed bounds describe the
same range. Each answer is the first `u64` of a fresh ChaCha8 stream whose 32-byte seed, zeroed, holds
the seed at byte 0, the absolute Tick as little-endian `u64` at bytes 1–8, the Function's column and
row as little-endian `i64` at bytes 9–16 and 17–24, and zeros after. With `low` and `high` the
normalized bounds, the answer is `low + word % (high - low + 1)` computed in 16 bits, so `00`–`FF`
is 256 values. Golden vectors pin the layout, the word and the mapping. The generator uses no OS
randomness and behaves identically native and in WebAssembly.

Reason: identical Snapshots at the same Tick plan identically, and Functions at different Positions
have independent streams.
Not: `StdRng`, activation history.
Sources: [ADR 0013](../adr/0013-random-is-deterministic-per-position.md).
Evidence: `lang/src/functions/tick.rs` `random`, `chacha_word`; dependency in `lang/Cargo.toml`; tests `lang/src/functions/tick.rs` `random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping`, `two_randoms_at_different_anchors_differ_and_moving_one_changes_its_stream`, `random_normalizes_reversed_bounds_and_returns_an_equal_bound`.

## Chords

### LANG-390 A Chord is a root Note and an Interval Set written as two operands, and is not a value

`C4M7` is two ordinary pairs, each carried and read as any pair. No Function answers a Chord and no
operand holds one.

Reason: an operand's width is fixed at parse, and `C7` is already a Note.
Sources: [ADR 0103](../adr/0103-one-chord-play-sounds-every-tone-of-its-chord.md).
Evidence: `lang/src/atom.rs` rows `RawChordPlay`, `TimedChordPlay`; test `lang/src/functions/chord.rs` `each_operand_reaches_the_role_its_signature_names`.

### LANG-400 Chord Play is Raw `!:` and Timed `!#`, each with its own fixed arity

Neither overloads Raw or Timed Play. There is no Monophonic Chord Play: a voice per channel cannot
hold several tones. What each Chord Play sounds is stated in [playback.md](playback.md).

Reason: arity is fixed at parse and a chord's signature differs from a note's.
Sources: [ADR 0103](../adr/0103-one-chord-play-sounds-every-tone-of-its-chord.md).
Evidence: tests `lang/src/functions/chord.rs` `each_chord_play_requires_exactly_its_declared_operands`; `orcvs/src/source/tick/chord_play.rs` `a_chord_play_is_invalid_where_a_value_is_required`.

### LANG-410 A chord's tones are built in one order, and a chord that cannot be built whole is not sent

The Voicing removes tones (`SH` keeps the 1st, 2nd and 4th; `RL` removes the 1st); the Inversion
then moves the lowest tone up an octave as many times as its Number says; the Voicing then moves
tones (`OP` raises every second tone an octave; `D2`, `D3`, `23`, `24` lower the positions they name,
counted from the top; `CL` moves nothing). An Interval Set with too few tones for its Voicing (`OP`
and `D2` need 2, `D3` and `23` need 3, `24` and `SH` need 4, `RL` needs 2), or any tone outside
`00`–`7F`, diagnoses the whole Chord Play once and sends nothing. A chord holds at most twelve
tones, and evaluating one allocates nothing. The tones exist only inside the Turn.

Reason: voicing and inversion compose the same way every time, and no partial chord is ever sent.
Sources: [ADR 0103](../adr/0103-one-chord-play-sounds-every-tone-of-its-chord.md), [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `lang/src/functions/chord.rs`; tests `every_voicing_with_inversion_applies_removal_then_inversion_then_movement`, `too_few_tones_for_a_voicing_diagnoses_the_whole_chord`, `a_tone_outside_the_note_range_diagnoses_the_whole_chord`, `a_chord_holds_at_most_twelve_tones_and_the_chromatic_scale_fills_them`; `lang/tests/allocation.rs` `a_chord_play_allocates_nothing_to_read_or_evaluate`.

## Scope of the vocabulary

### LANG-420 Orcvs keeps Orca's performative capabilities, not its encoding or compatibility

Each Orca capability maps onto an Orcvs Function family, with syntax and behaviour following Orcvs
rules. Orca's hidden variable table has no counterpart: Konkat is a Read of visible Source. The
capability table in ADR 0019, as amended, is the audit index; it is a tracking view, not a separate
source of language truth.

Reason: two-Cell, behaviour-first families express the same capabilities over the character Source.
Sources: [ADR 0019](../adr/0019-map-orca-capabilities-onto-orcvs-language-families.md), [ADR 0070](../adr/0070-read-write-and-copy-are-separate-families.md).
Evidence: index decision; no direct test.

### LANG-430 No Function exists only to serve tests

Every Function represents user-facing capability; `id` is not a Function and diagnoses as an
unknown spelling.

Reason: terse two-Cell names should carry meaningful behaviour.
Sources: [ADR 0015](../adr/0015-retire-the-identity-test-function.md).
Evidence: test `orcvs/src/source/model.rs` `test_retired_id_receives_the_unknown_function_diagnostic` (assertions inspected).

## Accepted but not built, or built differently

- LANG-460: Application Command `!$` is not in the Function table, so `!$` diagnoses as an unknown
  spelling. ADR 0008 itself defers its encoding. No tracking issue found.

## Open decisions

These are not binding.

- **Standalone Operand Literals.** A 2026-10-16 ruling would stop diagnosing a Number or Note pair
  where an Expression could start (amending LANG-130 and ADR 0021). It needs an ADR, tracked by
  [source-diagnostics 01](../../.scratch/source-diagnostics/issues/01-record-that-a-standalone-operand-literal-is-not-diagnosed.md);
  implementation by
  [source-diagnostics 04](../../.scratch/source-diagnostics/issues/04-stop-diagnosing-standalone-operand-literals.md).
  Until an ADR lands, LANG-130 binds.
- **Chord Play on an empty pattern row.** Whether a Chord Play's root, Interval Set, Voicing,
  inversion and length become optional so an empty row is silent is undecided (ADR 0108,
  Consequences). Being decided in
  [optional-play-note 06](../../.scratch/optional-play-note/issues/06-decide-chord-play-roots.md).
