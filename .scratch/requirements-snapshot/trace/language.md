# Trace: Language (LANG)

Inventory at `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4`. Owned ADRs: 0005, 0007, 0008, 0010, 0011,
0013, 0015, 0017, 0018, 0019, 0021, 0023, 0024, 0025, 0026, 0029, 0030, 0033, 0035, 0039, 0043,
0049, 0054, 0061, 0062, 0063, 0067, 0069, 0070, 0071, 0072, 0073, 0074, 0075, 0103 (language
clauses), 0104, 0105, 0107, 0108, 0109. Per-Function spellings, operands, domains and examples stay
in `docs/functions.md` (held to `lang/src/atom.rs` by `orcvs/tests/function_reference_doc.rs`);
rows below marked "redirected (functions.md)" are per-Function detail, not cross-Function rules.
Provisional rule IDs are placeholders for drafting.

## Source disposition

### Wholly superseded ADRs

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| ADR 0005 | Whole decision (defer addressing; `&` prefix and four directional Address Functions) | superseded | Addressing by 0049; `&` arrows are now Reads and the Address Functions are the Copy `=` family by 0070. 0005's status line still says the `&` prefix and the four Address Functions "stand" (see Conflicts C2) | proposed |
| ADR 0007 | Sequence is a first-class value | superseded | 0063 | source-stated |
| ADR 0023 | Range Functions `:-`/`:#`; `##` Comment; conversion "sole type-directed exception" | superseded | Range and Sequence by 0063; Comment by 0035; the remaining paragraph only restates 0061 | source-stated |
| ADR 0025 | Sequence membership checked at one construction point | superseded | 0063 | source-stated |
| ADR 0026 | Every value as a Sequence (conceptual) | superseded | 0063 | source-stated |
| ADR 0029 | Sequence membership admits only value Functions | superseded | 0063. The value/effect declaration its body leans on survives through ADR 0028 (EVAL) and 0061; CONTEXT.md and code still cite 0029 (Conflicts C4) | source-stated |
| ADR 0030 | Terminal Output Functions extend over Sequences | superseded | 0063 | source-stated |
| ADR 0039 | Pulse Functions refuse a Sequence operand | superseded | 0063 | source-stated |
| ADR 0062 | A blank operand gives a blank answer | superseded | chain 0062 -> 0066 -> 0069 | source-stated |
| ADR 0075 | A Play with a cleared note rests | superseded | 0107, then 0108 | source-stated |
| ADR 0104 | A blank note or root is a Rest however it got blank | superseded | 0107, then 0108 | source-stated |
| ADR 0107 | A blank note is invalid input however it got blank | superseded | 0108 | source-stated |

### Partly superseded or retained ADRs, by claim

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| ADR 0008 | Symbolic Function names put the behaviour-family glyph first, operation glyph second; a glyph may recur across families | retained LANG-180 Family-first two-Cell spellings | | source-stated |
| ADR 0008 | Family list `. ~ * & @ : !` | retained LANG-180 (as amended) | `:` retired by 0063; `&`/`@` redefined and `=` added by 0070; `\|\|`/doubled-glyph operandless group by 0035; `#` Pitch family by `.scratch/chords-and-pitch/spec.md` (C5) | source-stated |
| ADR 0008 | `!` Terminal Output family covers MIDI, UDP, OSC, Application Command, answers no value | retained LANG-180 | Membership only; activation EVAL, output PLAY | source-stated |
| ADR 0008 | MIDI spellings `!> !~ !% !c !b`, LSB-first Pitch Bend, unscaled bytes, channel `00`–`0F`, data `00`–`7F` | redirected PLAY | MIDI output; per-Function in functions.md | source-stated |
| ADR 0008 | UDP/OSC deferred | superseded | ADR 0085 (PLAY), 2026-10-26 amendment | source-stated |
| ADR 0008 | `!$` Application Command: host dispatcher only, never a shell or executable; encoding deferred | retained LANG-460 Application Command reserved | | source-stated |
| ADR 0008 | Sequence Function glyphs; `@<`/`@>` reserved as Source Read/Write | superseded | 0063; 0070 withdraws the reservation | source-stated |
| ADR 0008 | Numeric spellings `.+ .- .\| .x ./ .% .< .> .= .v .^`; `x` for multiply | redirected (functions.md) | Per-Function spelling | source-stated |
| ADR 0008 | Time spellings `~. ~* ~+ ~? ~% ~>` and names Delay/Interpolation | redirected EVAL | ADR 0012 and its 2026-11-14 amendment (Every, Slew) | source-stated |
| ADR 0008 | Function names are standard ASCII; Unicode deferred | retained LANG-010 | One ASCII byte per Cell | source-stated |
| ADR 0010 | Every Number is one byte spelled as two uppercase hex Cells `00`–`FF` | retained LANG-100 Two-Cell hexadecimal Number | Base-36 rationale historical | source-stated |
| ADR 0011 | `.+ .- .x` take two Numbers and wrap mod 256; no saturation | retained LANG-360 Arithmetic over bytes | Per-operation detail in functions.md | source-stated |
| ADR 0011 | General arithmetic accepts no Note and infers no type; conversion is explicit | retained LANG-360 | | source-stated |
| ADR 0011 | Division and Modulo by zero diagnose and produce no value | retained LANG-360 | Message wording per functions.md | source-stated |
| ADR 0011 | Equality broadcasting, vacuous all-equal, Sequence length diagnostics | superseded | 0063; Equality's Bang/Clear by 0071/0073 (LANG-330) | source-stated |
| ADR 0013 | Random derives from explicit seed, absolute Tick and Function Position; normalized inclusive bounds | retained LANG-380 Random is deterministic per Position | | source-stated |
| ADR 0013 | ChaCha8 seed layout, first `u64`, widened width mapping, golden vectors | retained LANG-380 | Sequence-index bytes now always zero | source-stated |
| ADR 0013 | `rand_chacha` with default features off, no OS randomness, native and wasm | retained LANG-380 | Dependency constraint | source-stated |
| ADR 0013 | Sequence index in identity; broadcasting | superseded | 0063 | source-stated |
| ADR 0015 | `id` is not a Function; no Function exists only for tests | retained LANG-430 No test-only Function | | source-stated |
| ADR 0017 | Orca Track/Generator/Query/Konkat/Push as Sequence operations | superseded | 0063; mappings restated by 0067/0070 (0019 amendment) | source-stated |
| ADR 0017 | Variable's hidden table omitted; Source Snapshot holds all persistent state | redirected EVAL | ADR 0003 | source-stated |
| ADR 0017 | `@<`/`@>` "stay reserved" | superseded | 0070 withdraws (Conflicts C3) | proposed |
| ADR 0018 | Stored program state is a Grid of single-character Cells; one Language Map derived per revision, no second grid or coordinate system | retained LANG-050 Language Map derived from Source | | source-stated |
| ADR 0018 | Each row partitions left to right into non-overlapping complete units; unknown spelling costs one Cell and the walk resumes; `***` is `**` plus invalid `*` | retained LANG-060 Row partition | | source-stated |
| ADR 0018 | Incomplete or invalid Live Edits are valid Source but form no Language Unit | retained LANG-060 | | source-stated |
| ADR 0018 | Language Map is a deep module; callers ask it about units | historical | Design rationale | proposed |
| ADR 0019 | Orcvs keeps Orca's performative capabilities without its one-letter encoding or case scheduling; behaviour follows Orcvs rules | retained LANG-420 Capabilities, not compatibility | | source-stated |
| ADR 0019 | Capability table (as amended) is the audit index; the map is a tracking view, not language truth | retained LANG-420 | Rows restated by 0070 and 0085 amendments | source-stated |
| ADR 0019 | Sequence rows (Range, Reverse, Concatenate) | superseded | 0063 | source-stated |
| ADR 0019 | UDP/OSC rows | redirected PLAY | ADR 0085 | source-stated |
| ADR 0021 | A literal takes exactly one Atom type from the consuming slot; Source stores characters, no type provenance; no global coercion | retained LANG-120 Operand Literal typed by its slot | | source-stated |
| ADR 0021 | A standalone operand literal without typed context is invalid | retained LANG-130 Standalone operand literal diagnosed | Contested (Conflicts C1) | source-stated |
| ADR 0021 | `.v` Note -> Number total; `.^` Number -> Note, diagnoses `80`–`FF`; monomorphic; no pass-through | retained LANG-370 Explicit Number/Note conversion | | source-stated |
| ADR 0021 | Nested Return loses the child type | retained LANG-260 | As amended by 0061 | source-stated |
| ADR 0021 | Note spelling `C/` through `G9` | retained LANG-110 Note spelling | | source-stated |
| ADR 0021 | 2026-11-10 amendment: operands named `note`/`number` | redirected (functions.md) | Role names | source-stated |
| ADR 0024 | Language Map derives units from the Parser; no independent spelling classifier; `**` in a Number slot is invalid input, not a Bang unit | retained LANG-050 | As amended by 0034 (EVAL) | source-stated |
| ADR 0024 | A literal keeps one kind; no Atom refinement recorded in the map | retained LANG-120 | | source-stated |
| ADR 0024 | Units ascend strictly by anchor in row-major order; Cell index and unit index are distinct types; `CellIndex` minted only by the Grid | retained LANG-050 / LANG-030 | Implementation invariant | source-stated |
| ADR 0033 | A row is divided into Expressions by parse, not spaces; the Parser reports the Grid's own Cell addresses | retained LANG-060 | | source-stated |
| ADR 0033 | An Expression claims what its arity declares; nested Function fills its slot and extends the claim | retained LANG-070 Arity claim | | source-stated |
| ADR 0033 | No record limit; long Expression stays one; Operand Stack bound by widest signature | retained LANG-070; stack bound redirected EVAL | ADR 0028 | source-stated |
| ADR 0033 | Empty Cell between Expressions is skipped silently; inside a claim it is a character | retained LANG-070 | | source-stated |
| ADR 0033 | Nothing spans a row edge; a slot past the edge is refused like an unknown spelling | retained LANG-070 | | source-stated |
| ADR 0033 | Comment pre-pass over `##`; claim stops at a Comment | superseded | 0035 | source-stated |
| ADR 0035 | Comment spelled `\|\|`, a Language Unit the Parser establishes; one `\|` is invalid | retained LANG-080 Comment delimiters | | source-stated |
| ADR 0035 | Comment claims the rest of its row | superseded | 0074 | source-stated |
| ADR 0035 | `\|\|` opens a Comment only where an Expression could start; inside an arity claim it is an operand Cell that fails to bind | retained LANG-080 | | source-stated |
| ADR 0035 | Comment records a Token and no Atom; answers no value, performs no effect, never scheduled | retained LANG-090 Comment is never evaluated | | source-stated |
| ADR 0035 | Reasons for `\|\|`, refusal of `!!` and `//`, unsound pre-pass | historical | | source-stated |
| ADR 0035 | `:` sigil listing | superseded | 0063 | source-stated |
| ADR 0043 | The Grid is part of the language; a Position outside it does not exist | retained LANG-020 The one Grid | | source-stated |
| ADR 0043 | Leaving the Grid is terminal: Self-Banging out of Grid becomes `**`; Copy out of Grid diagnoses | redirected EVAL | Placement and execution | source-stated |
| ADR 0043 | Presentation (Render Frame, Paint, viewport) is not the Grid | redirected CONS | ADR 0038 | source-stated |
| ADR 0043 | Reservation through end of row (ADR 0036) | superseded | 0063 retires variable-width Reservations | source-stated |
| ADR 0043 | Rejected "bounded for now" | historical | | source-stated |
| ADR 0049 | A Position is two Numbers, column then row, `00`–`FF`, from `00 00` top-left | retained LANG-030 Position is two Numbers | | source-stated |
| ADR 0049 | Grid at most 256x256, may be smaller | superseded | 0054 | source-stated |
| ADR 0049 | A pair the Grid does not reach names nothing; Portals resolve or refuse | retained LANG-030 | Unreachable under 0054 | source-stated |
| ADR 0049 | 2026-10-10 amendment: absolute forms are `&$ @$ =$`, each Position two Number operands | retained LANG-210 Absolute forms | | source-stated |
| ADR 0049 | Rejected alternatives | historical | | source-stated |
| ADR 0054 | Every Grid is 256x256; every pair `00 00`–`FF FF` is a Position | retained LANG-020 | | source-stated |
| ADR 0054 | A Source File states Cells, never a shape; line n = row n, char m = column m; short/missing lines are empty Cells; text past 256 is not a Source File | retained LANG-040 Source File | | source-stated |
| ADR 0054 | Stored Source of another shape discarded via persistence, not migrated; fresh console opens top-left; File > New | redirected CONS | Persistence and console | source-stated |
| ADR 0054 | Amends ADR 0047 default Grid | redirected CONS | | source-stated |
| ADR 0061 | Inline input portals are operands claimed greedily in signature order | retained LANG-070 | | source-stated |
| ADR 0061 | A nested Function hands its answer to its slot as two-Cell encoding (Return); the slot decodes it by its declared type, as a Portal write | retained LANG-260 Return decoded by the receiving operand | | source-stated |
| ADR 0061 | A nested Function also writes through its own Output Portal; Return is additional delivery | retained LANG-270 Nested Function writes its own Output Portal | Ordering/Reservation consequences redirected EVAL | source-stated |
| ADR 0061 | A nested Function that answers Bang writes `**` and activates aligned roots; its Return is `**` | retained LANG-270 | Activation semantics EVAL | source-stated |
| ADR 0061 | A Function that cannot produce a two-Cell Return cannot nest; the Parser refuses it from Source alone | retained LANG-250 Only value Functions nest | List clause superseded by 0067 | source-stated |
| ADR 0061 | Source Paint shows the refusal while typing | redirected CONS | | source-stated |
| ADR 0063 | Orcvs has no Sequence value; no Function accepts or answers a series | retained LANG-350 No series values | | source-stated |
| ADR 0063 | `:` family retired; Range, Reverse, Concatenate, Select, Replace removed; pervasive extension removed | retained LANG-350 | | source-stated |
| ADR 0063 | A chord is several Play roots one Bang activates | retained LANG-350 | Still valid beside Chord Play (0103) | source-stated |
| ADR 0063 | List clauses (List in a claim, literal count, untyped Items, Track `@t`) | superseded | 0067, 0070 | source-stated |
| ADR 0067 | Track is a value Function with a dynamic Input Portal, `index % count` pairs east of its last operand; count `00` diagnoses | redirected (functions.md) | Per-Function; spelling moved to `&t` by 0070 | source-stated |
| ADR 0067 | Track and every dynamic-Input-Portal Function read their pair exactly as a Copy reads its Input Portal; no reading rules of their own | retained LANG-220 Pass-through reading rules | | source-stated |
| ADR 0067 | Cells east of Track are ordinary Source; Track claims only its operands | retained LANG-440 Read Cells and lanes are ordinary Source | | source-stated |
| ADR 0067 | Whether a bare value standing alone is diagnosed is a question for the Parser | open | Conflicts C1 | source-stated |
| ADR 0067 | Read ordering at the Turn; dynamic Input/Output Portals; dynamic writer ordering; feedback; missed transient effects; cycles; schedule reuse | redirected EVAL | ADR 0032/0065/0068 | source-stated |
| ADR 0067 | Missed-Bang restricted to dynamic writes | superseded | 0072 | source-stated |
| ADR 0069 | An all-empty inline slot is invalid input like a partly written one; Function does not evaluate, write or return; diagnosed; nested invalid fails parent | retained LANG-280 Unwritten operand is invalid input | Amended by type: 0108/0109 | source-stated |
| ADR 0069 | An invalid Function writes nothing; its earlier output stands | retained LANG-340 No answer leaves the last write | Shared with EVAL error containment | source-stated |
| ADR 0069 | Pending is not a state | retained LANG-280 | | source-stated |
| ADR 0069 | An empty Cell is a character; Functions that copy Cells copy it (Copy, Track, Read) | retained LANG-220 | | source-stated |
| ADR 0069 | A Bang reaching a Play with an unwritten note finds it invalid | superseded | 0108 | source-stated |
| ADR 0069 | Absence Marker is an answer | superseded | 0073 | source-stated |
| ADR 0069 | Expression with an unwritten slot is diagnosed by the Language Map and not a root | retained LANG-290 Unbound required operand: diagnosed, not a root | Amended by 0108 for optional slots | source-stated |
| ADR 0069 | Source Paint shows an unwritten operand in its type colour | redirected CONS | | source-stated |
| ADR 0069 | Rejected: empty reads as zero, invalid clears its Output Portal | historical | | source-stated |
| ADR 0070 | Prefix names the action on Source: `&` reads, `@` writes, `=` copies | retained LANG-190 Read, Write and Copy families | | source-stated |
| ADR 0070 | Jump renamed Copy `=^ =v =< =>`, behaviour unchanged | retained LANG-190 | | source-stated |
| ADR 0070 | An arrow names the Cell acted on; same arrow, same direction in every family | retained LANG-190 | | source-stated |
| ADR 0070 | Directional distance counts Portals from the `distance` operand: `00` is the operand; one pair east/west or one row north/south in the operand's columns; nested distance counts from its slot | retained LANG-200 Directional distance | | source-stated |
| ADR 0070 | Track counts from the end of its last operand, nested Functions included | retained LANG-200 | | source-stated |
| ADR 0070 | Absolute forms take `$` second; Position per 0049; a pair starting at column `FF` is cut short | retained LANG-210 | | source-stated |
| ADR 0070 | Track is `&t`, Push is `@t`; Push writes into a lane of `count` pairs below it; lane below the Grid diagnoses | redirected (functions.md) | Per-Function | source-stated |
| ADR 0070 | Generator is a Read nested in a Write's `value` | retained LANG-230 | | source-stated |
| ADR 0070 | Ordering unchanged / per 0067 | redirected EVAL | | source-stated |
| ADR 0070 | Old spellings read under the new grammar; autosave discarded once | historical | One-time migration; persistence path is CONS | proposed |
| ADR 0070 | Function Replacement between Functions with different Portal rules is refused | redirected EVAL | Function Replacement | source-stated |
| ADR 0070 | A Write's/Push's `value` takes no type and carries its pair unchanged; destination reader decodes; empty `value` invalid; typed Function spelling in `value` is nested | retained LANG-230 Untyped `value` | | source-stated |
| ADR 0070 | Push lane is ordinary Source; Push claims only its operands | retained LANG-440 | | source-stated |
| ADR 0070 | 2026-11-10 amendment: a Position is one Role (`from`, `to`, `at`); `n` named `distance` | retained LANG-210 | Role names per Function in functions.md | source-stated |
| ADR 0070 | Rejected options | historical | | source-stated |
| ADR 0071 | A Copy, Read, Track, Write or Push carries `**` as any pair and writes it over its complete destination | retained LANG-220 / LANG-230 | Activation at destination and overwrite-not-activate redirected EVAL | source-stated |
| ADR 0071 | ADR 0014 relay clause withdrawn | superseded (of 0014) | Recorded here for chain only; 0014 is EVAL | source-stated |
| ADR 0071 | Pulses write their Output Portal every Turn | retained LANG-330 | As amended by 0073 (Clear) | source-stated |
| ADR 0071 | Bang display record; typed vs written `**` | superseded | 0072 | source-stated |
| ADR 0071 | Orca reading notes | historical | | source-stated |
| ADR 0072 | A standalone `**` is the Bang Function: no operands, root-only, intrinsically active, can emit Bang | retained LANG-240 Three readings of `**` | Turn, clearing, suppression, Halt, placement redirected EVAL | source-stated |
| ADR 0072 | Operand parsing reads `**` as the Bang value in an untyped operand; invalid syntax in a typed one | retained LANG-240 | | source-stated |
| ADR 0072 | Nothing about Bangs is remembered between Ticks | redirected EVAL | ADR 0003 | source-stated |
| ADR 0072 | Suppression, surviving-Bang, Halt as Producer, placement, missed Bang | redirected EVAL | | source-stated |
| ADR 0072 | Departures from Orca, rejected options | historical | | source-stated |
| ADR 0073 | A value Function answers a Language Unit or a Clear; a Clear writes two spaces at its Output Portal, root or nested | retained LANG-330 Clear | | source-stated |
| ADR 0073 | A nested Clear leaves its slot unwritten; parent invalid; diagnostic names the child | retained LANG-330 | Amended by type: optional binds none (0108), defaulted binds BLANK (0109) | source-stated |
| ADR 0073 | Equality, Every, Euclidean answer a Clear on non-Bang Turns; Copy, Read, Track answer a Clear for two empty Cells | retained LANG-330 | | source-stated |
| ADR 0073 | A Clear is not an Atom; no operand holds it | retained LANG-330 | | source-stated |
| ADR 0073 | "No output this Tick" is no answer: no Turn or invalid writes nothing | retained LANG-340 | | source-stated |
| ADR 0073 | Clear adds no dependency (pulse clears its reserved pair) | redirected EVAL | | source-stated |
| ADR 0074 | Comment closes at the leftmost `\|\|` after its introducer, else row end; `\|\|\|` unclosed, `\|\|\|\|` empty closed | retained LANG-080 | | source-stated |
| ADR 0074 | Text Cells are data: a Read, Track or Copy selecting them reads them as uncovered Cells; a pair touching a delimiter is partial input | retained LANG-090 | | source-stated |
| ADR 0074 | Only delimiters are occupied; Halt at text locks nothing; mover meets text as Cells; Write writes text; overwritten closer obscures until re-parse | retained LANG-090 | Halt/mover/write effects tested in EVAL area | source-stated |
| ADR 0103 | A Chord is a root Note then an Interval Set as two operands; not a value | retained LANG-390 Chord is two operands | | source-stated |
| ADR 0103 | Raw `!:` and Timed `!#` Chord Play; no Monophonic variant; no overloading of `!>`/`!~` | retained LANG-400 Chord Play variants | Operand lists in functions.md | source-stated |
| ADR 0103 | One Turn emits one ordinary Play Command per tone, lowest first; Playback Engine gains nothing | redirected PLAY | Sounding clauses | source-stated |
| ADR 0103 | Tones built in fixed order: Voicing removes, Inversion raises lowest, Voicing moves | retained LANG-410 Chord building | `D5`/`D6` renamed `23`/`24` by 0105 | source-stated |
| ADR 0103 | Tone outside `00`–`7F` or too few tones for the Voicing diagnoses the whole Chord Play; no partial chord | retained LANG-410 | | source-stated |
| ADR 0103 | Chord Play diagnosed as one Expression | retained LANG-410 | | source-stated |
| ADR 0103 | At most twelve tones; evaluation allocates nothing | retained LANG-410 | | source-stated |
| ADR 0103 | Nested Read width accounting; tracker layout notes | historical | | source-stated |
| ADR 0105 | Interval Set is one literal type naming semitones above a root; 39 names (25 chords, 14 scales) per 2026-11-13 amendment | retained LANG-140 Interval Set literal | Name list in functions.md | source-stated |
| ADR 0105 | Every name fills both Cells; half-blank is partial input; no filler `_` | retained LANG-140 | `_` retired by amendment | source-stated |
| ADR 0105 | A scale name is valid wherever a chord name is | retained LANG-140 | | source-stated |
| ADR 0105 | Names are case-sensitive | retained LANG-140 | | source-stated |
| ADR 0105 | Voicing is a separate literal type; 8 names `CL OP D2 D3 23 24 SH RL`; inversion is not part of it | retained LANG-150 Voicing literal | | source-stated |
| ADR 0105 | Both are carried as Numbers and Notes are; operand position decides `69`, `13`, `07`, `09`, `d7`, `D2`, `23`, `24` | retained LANG-120 / LANG-220 | | source-stated |
| ADR 0105 | Uniqueness across Functions, Interval Sets and Voicings; no names differing only by `0`/`o`; build refuses filler or look-alike; one-Cell slips accepted, not diagnosed | retained LANG-160 Name-table integrity | | source-stated |
| ADR 0105 | Stack answers an Interval Set; diagnoses when no name fits; operand `index` | redirected (functions.md) | Pitch family has no ADR (C5) | source-stated |
| ADR 0105 | Old names `M_`, `MA` as scale, `D5`, `D6`: no aliases | historical | Migration note | source-stated |
| ADR 0105 | Atom gains Interval Set and Voicing | retained LANG-170 Atom kinds | | source-stated |
| ADR 0105 | Language Map and Source Paint gain an operand type each | redirected CONS (paint) | | source-stated |
| ADR 0108 | `Optional<O>` admits a slot holding no value; present value binds as `O`; blank or nested-Clear binds "no value" | retained LANG-300 Optional operand | | source-stated |
| ADR 0108 | A Play's note (`!> !~ !%`) is `Optional<Note>`: no note sends nothing, no diagnostic, by any route; half-typed note diagnosed | retained LANG-300 | | source-stated |
| ADR 0108 | Admission from the signature alone; Parser, Language Map and Tick carry absent without interpreting it | retained LANG-310 Admission from the signature | | source-stated |
| ADR 0108 | Every other operand still binds on a blank step; Play keeps its Turn | retained LANG-300 | Turn order EVAL | source-stated |
| ADR 0108 | Blank-note Play allocates nothing to read or take its Turn | retained LANG-300 | `lang/tests/allocation.rs` | source-stated |
| ADR 0108 | Chord Play root stays a plain Note; empty pattern row for Chord Play is its own decision | open | Conflicts C6 | source-stated |
| ADR 0109 | `Defaulted<O, BLANK>`: blank or nested Clear binds as if `BLANK` were written, through `O`'s bind | retained LANG-320 Defaulted operand | | source-stated |
| ADR 0109 | Table row states `role: Defaulted<Number, 0x..>` | retained LANG-320 | | source-stated |
| ADR 0109 | A `lang` test binds every blank and fails on one `O` refuses | retained LANG-320 | | source-stated |
| ADR 0109 | Neither adapter wraps one that admits a blank (compile-time assert) | retained LANG-320 | | source-stated |
| ADR 0109 | Only a Number has a fixed value | retained LANG-320 | | source-stated |
| ADR 0109 | Monophonic Play `legato` is the first defaulted operand | redirected (functions.md) / PLAY | Mono semantics PLAY | source-stated |

### CONTEXT.md behaviour clauses

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| CONTEXT.md Source, Cell | Cell holds one printable single-byte ASCII character; space is empty | retained LANG-010 | Not cited by Rust beyond the noun | proposed |
| CONTEXT.md Source File | States Cells, never a shape | retained LANG-040 | ADR 0054 | source-stated |
| CONTEXT.md Grid | 256x256, outside does not exist | retained LANG-020 | `orcvs/src/grid.rs:985` cites "Grid laws CONTEXT.md states" but quotes a superseded sentence ("at least one column and one row") | source-stated |
| CONTEXT.md Position | Obtained only from its Grid; two Numbers column then row; absolute Functions take two Number operands per Position | retained LANG-030 / LANG-210 | Cited by `grid.rs:987` (older wording) | source-stated |
| CONTEXT.md Language Unit, Span | One anchor, contiguous run within one row; Comment records a Token and no Atom | retained LANG-060 / LANG-090 | | source-stated |
| CONTEXT.md Language Map | Derived from Parser, no persistent state | retained LANG-050 | `orcvs/src/source/language_map.rs:2009,2231` cite CONTEXT.md for the left-to-right partition, which is in ADR 0018/0033, not in the entry | source-stated |
| CONTEXT.md Atom | Number, Note, Interval Set, Voicing, Bang, Function; Clear is not an Atom | retained LANG-170 | | source-stated |
| CONTEXT.md Interval Set, Voicing | Name lists, slips accepted, operand position decides | retained LANG-140/150/160 | | source-stated |
| CONTEXT.md Chord, Inversion | Chord not a value; Inversion applies after removal, before movement | retained LANG-390 / LANG-410 | | source-stated |
| CONTEXT.md Operand Literal | Typed by consuming slot; Write/Push `value` untyped; standalone literal "open (ADR 0067)", diagnosed today | retained LANG-120/230; standalone clause open | Conflicts C1 | source-stated |
| CONTEXT.md Defaulted Operand | Blank binds as the Number written there | retained LANG-320 | | source-stated |
| CONTEXT.md Spatial Output, Return | Same encoding however it arrives; Return is additional delivery; refused Output Portal write keeps the Return; parent failure does not undo child write; Clear under optional note | retained LANG-260/270/330 | `orcvs/src/source/encoding.rs:3` cites Spatial Output | source-stated |
| CONTEXT.md Pending, Pending Operand Encoding | Not a state; Play with unwritten note is valid | retained LANG-280 / LANG-300 | | source-stated |
| CONTEXT.md Expression | Contiguous, never crosses a row edge; Comment extent from closer | retained LANG-070 / LANG-080 | | source-stated |
| CONTEXT.md Function | Declares value or effect; nesting refusal and activation gate read that declaration "per ADR 0029" | retained LANG-250 | Cites superseded ADR 0029 (Conflicts C4) | source-stated |
| CONTEXT.md Role | Operand Roles named in the table; Position's two Numbers one Role | retained LANG-210 | | source-stated |
| CONTEXT.md Number, Note, Numeric Conversion, Arithmetic, Atomic Function | Byte domain, wrap, Note spelling, monomorphic conversion | retained LANG-100/110/360/370 | | source-stated |
| CONTEXT.md Pitch Function, Up, Down, Pick, Stack | `#` family; Notes never wrap or clamp; Stack diagnoses when no name fits | open | No ADR (Conflicts C5); per-Function detail in functions.md | proposed |
| CONTEXT.md Random | Seed, Tick, Position; ChaCha8 | retained LANG-380 | | source-stated |
| CONTEXT.md Clear | Two spaces; not an Atom; nested Clear invalidates parent unless Play note | retained LANG-330 | Entry omits the Defaulted case (ADR 0109) | source-stated |
| CONTEXT.md Comment | Delimiters occupied; text data; `\|\|` only where an Expression could start | retained LANG-080 / LANG-090 | | source-stated |
| CONTEXT.md Bang, Bang Function (reading clauses) | `**` value in untyped operand, invalid in typed, standalone is Bang Function | retained LANG-240 | Activation clauses EVAL | source-stated |
| CONTEXT.md Read, Write, Push, Track, Copy Function | Family definitions; reading by Copy rules; claims only operands | retained LANG-190–230/440 | Per-Function detail functions.md; ordering EVAL | source-stated |
| CONTEXT.md Terminal Output Function | Family membership incl. `!$`; Chord Play one command per tone; bounded twelve tones | retained LANG-180/410/460; output redirected PLAY | Cited by `console/src/function_reference.rs:98,982` ("never writes a Cell result") | source-stated |
| CONTEXT.md Raw Chord Play, Timed Chord Play | Build order, whole-or-nothing | retained LANG-410; Timed tone lifetimes PLAY | | source-stated |
| CONTEXT.md Play Function (note clause) | Note optional by type | retained LANG-300 | | source-stated |
| CONTEXT.md Operand Stack | Bounded by widest signature | redirected EVAL | ADR 0028 | source-stated |

### `.scratch/` rulings not captured in an ADR

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| `.scratch/chords-and-pitch/spec.md` "Implementation Decisions" | `#` Pitch family: Pick `#p`, Up `#+`, Down `#-`, Stack `#s`; `.` stays numeric; Pick and Stack formulas | open | No ADR; built and in CONTEXT.md and functions.md (C5) | source-stated |
| `.scratch/chords-and-pitch/spec.md` | Chord Play inversion is a Number `00`–`FF`; each step raises the lowest tone, climbing past the tone count | redirected (functions.md) | Per-Function detail consistent with 0103 | source-stated |
| `.scratch/source-diagnostics/issues/01-*.md` | Decided 2026-10-16 (Toby): a two-Cell Number or Note where an Expression could start is not diagnosed; unrecorded as ADR | open | Conflicts C1; implementation is issue 04 | source-stated |

## Evidence and gaps

Tick-level tests mostly run on `Grid::with_shape` test Grids (`orcvs/src/grid.rs:183`, compiled
only for tests and the `test-grid-shapes` feature); the shipped Grid is always 256x256.

| Rule | Code | Tests (assertions inspected?) | Status |
| --- | --- | --- | --- |
| LANG-010 Cells are printable ASCII | `orcvs/src/source/file.rs` `read`; `orcvs/src/source/cell.rs` | `file.rs` `reading_refuses_every_byte_that_is_not_printable_ascii_or_space` (names only); `lang/src/parser.rs` `source_that_is_not_ascii_is_refused_rather_than_panicking` | built and covered |
| LANG-020 The one Grid | `orcvs/src/grid.rs` `COL_COUNT`, `ROW_COUNT`, `Grid::new` | `grid.rs` `test_every_grid_is_as_large_as_two_numbers_address`, `test_a_persisted_grid_of_any_shape_but_the_one_is_refused` (inspected) | built and covered |
| LANG-030 Position | `grid.rs` `Position`, `CellIndex`, `Grid::position`, `position_at` | `grid.rs` `test_grid_mints_positions_inside_it`, `test_grid_refuses_positions_outside_it`, property `a_position_exists_exactly_where_the_grid_has_a_cell`; `orcvs/tests/language_map.rs` `token_at_refuses_a_position_minted_by_another_grid` (names) | built and covered |
| LANG-040 Source File | `orcvs/src/source/file.rs` `read`, `write` | `reading_treats_short_and_missing_lines_as_empty_cells`, `reading_refuses_a_257th_line`, `reading_refuses_a_257th_character` (names) | built and covered |
| LANG-050 Language Map derived | `orcvs/src/source/language_map.rs`; `lang/src/parser.rs` `Parser::analyze` | `language_map.rs` `language_map_builds_cohesive_expression_state`, `invalid_operand_bang_spellings_are_not_parsed_bang_values` (names) | built, no verified test (names only) |
| LANG-060 Row partition | `lang/src/parser.rs` `take_language_unit`; `language_map.rs` row walk | `parser.rs` `an_unrecognized_function_consumes_one_cell_and_records_one_invalid_slot`; `language_map.rs` `language_map_partitions_complete_units_left_to_right_without_overlap`, `an_odd_standalone_run_costs_one_cell_and_leaves_the_rest_readable` (names) | built and covered |
| LANG-070 Arity claim | `parser.rs` `Parser::analyze` | `language_map.rs` `a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary`, `language_map_never_forms_a_unit_across_a_row_edge`, `a_partly_written_or_edge_cut_operand_is_diagnosed`; `orcvs/tests/language_map.rs` `truncated_operand_owns_the_available_row_tail` (names) | built and covered |
| LANG-080 Comment delimiters | `parser.rs` Comment arm | `parser.rs` `a_comment_ends_at_the_next_comment_introducer`, `a_comment_introducer_does_not_close_itself`, `a_run_of_pipes_closes_a_comment_at_its_first_two`, `a_comment_introducer_inside_an_operand_claim_is_a_refused_operand`, `a_lone_vertical_rule_is_not_a_comment` (inspected) | built and covered |
| LANG-090 Comment never evaluated; text is data | `parser.rs`; `orcvs/src/source/portal.rs` occupancy | `parser.rs` `strict_parsing_refuses_a_comment`; `orcvs/src/source/tick/read.rs` `a_pair_of_a_comments_text_reads_as_data` (inspected), `a_comment_closer_at_the_selected_pair_diagnoses`; `track.rs` `track_reads_a_pair_of_a_comments_text`; `write.rs` `a_write_into_comment_text_writes_its_cells`; `halt.rs` `a_halt_aimed_at_comment_text_locks_nothing_and_does_not_diagnose`; `portal.rs` `occupancy_of_a_comment_is_its_delimiters_and_not_its_text` | built and covered |
| LANG-100 Number | `lang/src/lib.rs` number decode; `atom.rs` `number_atom_from_spelling` | `lib.rs` `str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings`, `every_number_has_one_canonical_two_cell_source_encoding` (names) | built and covered |
| LANG-110 Note spelling | `lib.rs`, `atom.rs` `Note`, `note_atom_from_spelling` | `lib.rs` `midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range`, `note_source_encoding_covers_the_documented_boundaries` (names) | built and covered |
| LANG-120 Literal typed by slot | `lang/src/expression.rs` / `Token::decode`; `atom.rs` | `atom.rs` `a_name_that_also_spells_a_number_or_note_is_read_by_the_position_receiving_it`; `driven_chord.rs` `the_same_cells_mean_what_each_operand_receiving_them_reads`; `parser.rs` `conversion_literal_operands_are_monomorphic` (names) | built and covered |
| LANG-130 Standalone literal diagnosed | `parser.rs` `take_language_unit` (no literal fallback) | `parser.rs` `an_operand_literal_outside_a_slot_is_not_an_expression` (inspected); `orcvs/src/source/model.rs` assertion "a lone Number reads as an unknown Function on the next Tick" (inspected) | built and covered per ADR 0021; pending reversal tracked by `.scratch/source-diagnostics/issues/01-record-that-a-standalone-operand-literal-is-not-diagnosed.md` and `04-stop-diagnosing-standalone-operand-literals.md` (both ready-for-agent) |
| LANG-140 Interval Set | `atom.rs` `define_interval_sets!`, `IntervalSet` | `atom.rs` `every_interval_set_name_decodes_to_its_semitones`, `an_unknown_or_half_blank_name_is_refused_by_both_literal_types` (names) | built and covered |
| LANG-150 Voicing | `atom.rs` `define_named_literal!` (Voicing) | `atom.rs` `every_voicing_name_decodes_and_spells_itself_back`; `chord.rs` `every_voicing_gives_the_tones_it_names` (names) | built and covered |
| LANG-160 Name-table integrity | `atom.rs` `const _` block (no `_`, no `0`/`o` look-alikes), `shares_a_spelling` | compile-time asserts (inspected `atom.rs:340-375`); `an_interval_set_and_a_voicing_refuse_each_others_names` | built and covered |
| LANG-170 Atom kinds | `atom.rs` `enum Atom` | `parser.rs` `every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it` (names) | built and covered |
| LANG-180 Families | `atom.rs` `define_functions!` table | `atom.rs` `*_display_with_the_*_family_spellings` tests; `parser.rs` `no_function_is_spelled_in_the_colon_family`, `every_real_function_parses_and_renders_from_its_canonical_spelling` (names) | built and covered |
| LANG-190 Read/Write/Copy families, arrows | `atom.rs` rows `ReadNorth`…`WriteWest`, `Copy*`, `direction()` | `parser.rs` `the_ampersand_arrows_parse_as_reads_of_one_number`; `atom.rs` `every_function_names_its_input_portal` (names) | built and covered |
| LANG-200 Directional distance | `atom.rs` `Function::distance`; `lang/src/portal.rs` | `orcvs/src/source/tick/read.rs` `a_distance_counts_portals_from_the_distance_operand`, `every_read_at_distance_zero_answers_its_own_operand`, `a_nested_distance_counts_from_the_slot_it_occupies` (names); `track.rs` | built and covered |
| LANG-210 Absolute forms, Position as one Role | `atom.rs` `Function::position`, `function_roles!`, `RoleGroup` | `read.rs` `an_absolute_read_of_column_fe_reads_and_of_column_ff_is_cut_short`; `absolute_copy.rs`; `atom.rs` `a_position_grouped_as_one_role_is_accepted` (names) | built and covered |
| LANG-220 Pass-through reading rules | `lang/src/functions/copy.rs`, `read.rs`, `track.rs`; `orcvs/src/source/tick/inputs.rs` | `read.rs` `a_read_answers_what_a_copy_answers_for_the_cells_it_reads`; `carried_names.rs` `a_read_answers_each_name_unchanged`, `a_read_or_track_diagnoses_a_half_written_name_and_writes_nothing`; `track.rs` `all_empty_pairs_clear_the_output_portal_without_a_diagnostic` (names) | built and covered |
| LANG-230 Untyped `value` | `atom.rs` `Untyped` token; `functions/write.rs` | `parser.rs` `a_writes_value_slot_carries_any_language_unit_a_literal_spells`; `carried_names.rs` `a_write_carries_each_name_to_its_pair_unchanged`, `a_push_carries_each_name_to_its_pair_unchanged` (names) | built and covered |
| LANG-240 Three readings of `**` | `parser.rs` root-only `**` arm; `atom.rs` `Bang` named row | `parser.rs` `bang_and_self_banging_functions_parse_as_complete_language_units`, `a_bang_in_a_typed_operand_is_still_invalid_syntax`; `language_map.rs` `a_bang_outside_every_claim_is_a_bang_however_the_source_around_it_reads` (names) | built and covered |
| LANG-250 Only value Functions nest | `parser.rs` `SyntaxError::NestedEffectFunction`; `atom.rs` `answers_value` | `parser.rs` `a_nested_function_that_answers_no_value_is_refused_from_source_alone`, `a_nested_play_with_a_blank_note_is_refused_as_nested` (inspected) | built and covered |
| LANG-260 Return decoded by receiver | `orcvs/src/source/encoding.rs`; `Token::decode` | `orcvs/src/source/tick/nesting.rs` `a_return_and_a_portal_decode_the_same_characters_the_same_way` (inspected); `numeric_conversion.rs` `nested_conversions_compose_and_read_their_return_by_the_receiving_operand` | built and covered |
| LANG-270 Nested writes own Output Portal; nested Bang | `orcvs/src/source/tick/execution.rs` | `nesting.rs` `nested_feedback_functions_keep_their_state_in_their_own_output_portals`, `a_nested_bang_writes_its_bang_and_activates_its_aligned_roots`, `a_child_write_survives_its_parents_failure`; `live_editing.rs` `live_child_write_survives_parent_failure_and_rejected_portal_keeps_its_return` (names) | built and covered |
| LANG-280 Unwritten operand invalid | `lang/src/operand.rs` `Operand::ABSENT` (None for plain); `parser.rs` `is_blank_next` | `operand.rs` `a_plain_operand_still_refuses_a_blank_slot`; `clear.rs` `a_nested_function_invalid_on_an_unwritten_slot_fails_its_parent`; `functions/mod.rs` `a_plain_operand_refuses_an_absent_value_as_it_refuses_a_blank_slot` (names) | built and covered |
| LANG-290 Not a root | `language_map.rs` roots | `language_map.rs` `an_expression_waiting_on_unwritten_operands_is_diagnosed_and_not_a_root`, `a_play_whose_only_blank_operand_is_its_note_is_a_root` (inspected: covers `!%007F  04  ` with blank legato), `a_play_with_anything_else_unbound_is_diagnosed_and_not_a_root` | built and covered |
| LANG-300 Optional note | `atom.rs` rows `RawPlay`, `TimedPlay`, `MonophonicPlay` `note: Optional<Note>`; `operand.rs` `Optional` | `parser.rs` `a_blank_raw_play_note_is_recorded_absent_and_reports_nothing` (inspected); `blank_note.rs` all three routes, `a_raw_play_with_a_blank_note_still_diagnoses_its_channel_and_velocity`; `lang/tests/allocation.rs` `reading_and_taking_the_turn_of_a_play_with_a_blank_note_allocates_nothing` | built and covered |
| LANG-310 Admission from the signature | `atom.rs` `blank_admitted`, `operand.rs` `ABSENT` | `atom.rs` `every_blank_a_function_admits_binds` (names) | built and covered |
| LANG-320 Defaulted operand | `operand.rs` `Defaulted`; `atom.rs` `legato: Defaulted<Number, 0x00>` | `operand.rs` `a_blank_defaulted_slot_binds_its_fixed_value`, `a_fixed_value_outside_the_domain_fails_the_table_check_as_if_written`; compile_fail doctests at `operand.rs:300,338`; `blank_note.rs` `a_nested_clear_under_a_bang_driven_output_binds_absent_in_a_note_and_as_00_in_legato` (names) | built and covered |
| LANG-330 Clear | `orcvs/src/source/tick/execution.rs`; `functions/math.rs`, `tick.rs` | `interpreter.rs` `only_the_pulses_and_the_functions_that_copy_cells_answer_a_clear`; `clear.rs` `a_nested_equality_that_does_not_bang_leaves_its_parent_operand_unwritten` (inspected message "cleared its operand"), `every_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang` | built and covered |
| LANG-340 No answer leaves the last write | `execution.rs` | `clear.rs` `an_unwritten_operand_in_the_source_blocks_its_function_as_the_map_reports`; `bang_function.rs` tests (names) | built, no verified test (names only) |
| LANG-350 No series values | absence of `:` rows | `parser.rs` `no_function_is_spelled_in_the_colon_family`; `language_map.rs` `a_retired_colon_spelling_reserves_nothing` (names) | built and covered |
| LANG-360 Arithmetic over bytes | `lang/src/functions/math.rs` | `interpreter.rs` `general_arithmetic_wraps_for_every_pair_of_bytes`, `general_arithmetic_rejects_note_operands`, `division_is_the_asymmetry_that_diagnoses_every_zero_divisor`, `modulo_by_zero_diagnoses_distinctly_from_division_by_zero` (names) | built and covered |
| LANG-370 Explicit conversion | `lang/src/functions/numeric_conversion.rs` | `conversion_to_note_refuses_every_number_above_the_range`, `each_conversion_refuses_a_value_of_the_type_it_answers`; `nesting.rs` (inspected `.v.vC4` diagnoses) | built and covered |
| LANG-380 Random | `lang/src/functions/tick.rs` `random`, `chacha_word`; `lang/Cargo.toml:27` | `tick.rs` `random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping`, `two_randoms_at_different_anchors_differ_and_moving_one_changes_its_stream`, `random_normalizes_reversed_bounds_and_returns_an_equal_bound` (names) | built and covered |
| LANG-390 Chord is two operands | `atom.rs` `RawChordPlay`, `TimedChordPlay` rows | `chord.rs` `each_operand_reaches_the_role_its_signature_names` (names) | built and covered |
| LANG-400 Chord Play variants | `atom.rs` rows `!:`, `!#` | `chord.rs` `each_chord_play_requires_exactly_its_declared_operands`; `chord_play.rs` `a_chord_play_is_invalid_where_a_value_is_required` (names) | built and covered |
| LANG-410 Chord building | `lang/src/functions/chord.rs` | `every_voicing_with_inversion_applies_removal_then_inversion_then_movement`, `too_few_tones_for_a_voicing_diagnoses_the_whole_chord`, `a_tone_outside_the_note_range_diagnoses_the_whole_chord`, `a_chord_holds_at_most_twelve_tones_and_the_chromatic_scale_fills_them` (partly inspected); `lang/tests/allocation.rs` `a_chord_play_allocates_nothing_to_read_or_evaluate` | built and covered |
| LANG-420 Capabilities, not compatibility | n/a (index ADR) | none | n/a (index) |
| LANG-430 No test-only Function | absence of `id` | `orcvs/src/source/model.rs` `test_retired_id_receives_the_unknown_function_diagnostic` (inspected) | built and covered |
| LANG-440 Read Cells and lanes are ordinary Source | `atom.rs` Track/Push rows claim only operands | `language_map.rs` tests; `push.rs`, `track.rs` (names) | built, no verified test (names only) |
| LANG-460 Application Command reserved | no `!$` row in `atom.rs` (deferred by ADR 0008) | `console/src/function_reference.rs:978` notes `!$` absent | not built (deferred by source); none found |

Documentation and citation gaps (not code gaps):

- `docs/functions.md:32` ("An empty operand diagnoses, except a Play's `note`") and `:40` ("Nested, a Clear leaves its parent invalid, except in a Play's `note`") omit ADR 0109's Defaulted `legato`; the `!%` entry at `:1251` states it correctly.
- CONTEXT.md Clear entry omits the Defaulted case for a nested Clear (ADR 0109); the Defaulted Operand entry covers it.
- `orcvs/src/grid.rs:985-991` quotes Grid/Position wording CONTEXT.md no longer holds ("at least one column and one row").
- `orcvs/src/source/language_map.rs:2009,2231` attribute the left-to-right partition rule to CONTEXT.md; it lives in ADRs 0018/0033.
- `lang/src/atom.rs:482,1079`, `lang/src/lib.rs:356` and CONTEXT.md Function entry cite superseded ADR 0029 (C4).
- `.scratch/chords-and-pitch/issues/05-rest-a-chord-play-on-a-blank-root.md` (needs-triage) implements superseded ADR 0104; its own comment says rewrite or close.

## Conflicts and open questions

**C1 Standalone operand literal.** ADR 0021: "a standalone operand literal without typed Expression
context is invalid." ADR 0067: "Whether a bare value such as `C4` standing on its own is diagnosed is
a question for the Parser ... this decision does not answer it." CONTEXT.md Operand Literal:
"Whether a standalone operand literal is diagnosed is open (ADR 0067); the Language Map diagnoses one
today." `.scratch/source-diagnostics/issues/01` records Toby's 2026-10-16 ruling to stop diagnosing
Number/Note pairs, with no ADR yet. Code and tests diagnose (`parser.rs`
`an_operand_literal_outside_a_slot_is_not_an_expression`; `model.rs:2957`). ADR 0021's status line
does not name 0067. Recommendation: state LANG-130 from ADR 0021 as the binding rule, mark it
"pending reversal (source-diagnostics 01/04)", and fix CONTEXT.md's "open" wording. Consequence:
the snapshot agrees with code now and must be refreshed when that ADR lands.

**C2 ADR 0005's surviving clause.** 0005 status: "The `&` prefix and the four directional Address
Functions stand." ADR 0070: "`&` reads Source ... The four Jumps `&^ &v &< &>` become the Copy
Functions `=^ =v =< =>`" and `&` arrows are directional Reads. 0070's status amends 0008, 0019, 0049
and 0067 but not 0005. Code: `ReadNorth => ("&^", ...)`, `CopyNorth => ("=^", ...)`.
Recommendation: dispose 0005 as wholly superseded (0049 + 0070, basis proposed) and list a status-line
follow-up. Consequence: no LANG rule cites 0005.

**C3 ADR 0017 reservation of `@<`/`@>`.** 0017 body (with its 0049 note): "both stay reserved."
ADR 0070 (in 0008's amendment): "The reservation of `@<` for Source Read and `@>` for Source Write
... is withdrawn: `@<` and `@>` are the west and east Writes." 0017's status line names 0063 and 0067
only. Code: `WriteWest => ("@<", ...)`. Recommendation: 0070 governs as the later, explicit decision;
status-line follow-up. Consequence: none for behaviour.

**C4 ADR 0029 cited as live authority.** 0029 is wholly superseded by 0063, yet CONTEXT.md Function
says "per ADR 0029 that declaration is what the Parser's nesting refusal and the activation gate each
read", and `lang/src/atom.rs:482` and `lang/src/lib.rs:356` cite 0029. The rule survives through ADR
0028 (EVAL: every Function answers a value or an effect) and ADR 0061 (a Function that cannot produce
a Return cannot nest, refused by the Parser from its declaration). Recommendation: LANG-250 cites
0061 and 0028, not 0029; flag the CONTEXT.md and comment citations for repointing. Consequence: a
reader following the citation lands on a superseded ADR until then.

**C5 Pitch family has no ADR.** `.scratch/chords-and-pitch/spec.md`: "ADR 0103 decides Chord Play,
... ADR 0105 the Interval Set and Voicing literals. The Pitch family is decided here." ADR 0035 only
leaves `#` "available as a sigil for a pitch family should one be declared"; ADR 0105 names Stack.
CONTEXT.md and `docs/functions.md` state `#+ #- #p #s`; `lang/src/functions/pitch.rs` builds them with
tests (`pick_wraps_an_index_past_the_end_of_the_set_up_an_octave`, `up_diagnoses_an_answer_above_the_note_range_rather_than_wrapping`).
Recommendation: record the `#` family and its cross-Function rules (Notes never wrap or clamp; `.`
stays numeric) as LANG-450 with basis CONTEXT.md plus the `.scratch` ruling, explicitly marked as
not ADR-backed, and propose an ADR. Consequence: the snapshot cites a non-ADR source for one family.

**C6 Chord Play on an empty pattern row.** ADR 0108: "A Chord Play's root stays a plain Note ...
whether those become optional too is its own decision." Open in
`.scratch/optional-play-note/issues/06-decide-chord-play-roots.md` (needs-triage); stale
`.scratch/chords-and-pitch/issues/05-*` implements superseded 0104. Code: `root: Note` plain. Not a
conflict, an open decision. Recommendation: LANG-390/410 state root required; list the open decision.

## Fresh-reader questions

1. **Ordinary: "I typed `.+01 02` and get a diagnostic; `.+0102` works. Why?"** Inside an Expression's
   claim an empty Cell is a character, so the second operand is `" 0"`, not a Number; the Addition is
   refused and `2` opens the next Expression. Spacing matters only between Expressions. Sources: ADR
   0033 ("An empty Cell is a character like any other"), ADR 0069; `orcvs/src/source/language_map.rs`
   `a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary`.
2. **"Can I make Timed Play's blank `length` mean `01`?"** Yes, by declaring it
   `length: Defaulted<Length, 0x01>` (Length reads as a Number) in its Function-table row; no Parser, Language Map or Tick
   change. The blank of every declared operand is bound by a `lang` test that fails if `0x01` is outside
   the operand's domain; it cannot also be `Optional`; only a Number can be defaulted. Changing the
   meaning of blank is language design (pre-release). Sources: ADR 0109, ADR 0108; `lang/src/operand.rs`
   `Defaulted`; `lang/src/atom.rs` `every_blank_a_function_admits_binds`; per-Function entry in
   `docs/functions.md`.
3. **Cross-topic (EVAL/PLAY): "A Read nested in a Play's note reads two empty Cells. Is anything
   diagnosed, and what happens to the Cells under the Read?"** The Read copies empty Cells, answering a
   Clear: it writes two spaces at its own Output Portal and leaves the note slot unwritten. The note is
   `Optional<Note>`, so the Play binds no note, sends nothing and is not diagnosed; channel and velocity
   still bind, and the Play keeps its place in Tick order. Sources: ADR 0073, 0061, 0108, 0069 (copy of
   empty Cells), ADR 0065 (EVAL); `orcvs/src/source/tick/blank_note.rs`
   `a_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing_and_reports_nothing`,
   `a_cleared_note_leaves_the_clear_under_the_nested_read`.
4. **"Why do the Note pairs east of my Track show diagnostics?"** A standalone Operand Literal has no
   slot to type it and is invalid under ADR 0021; the Parser reports each as an unknown Function (two
   per pair, because recovery advances one Cell). ADR 0067 left the question to the Parser; a ruling
   to stop diagnosing Number/Note pairs is recorded in `.scratch/source-diagnostics/issues/01` and 04,
   not yet an ADR or code. Sources: ADR 0021, 0067, 0018; CONTEXT.md Operand Literal;
   `lang/src/parser.rs` `an_operand_literal_outside_a_slot_is_not_an_expression`.
5. **"Old Source used `C4MA` to sound a major scale; now it sounds three notes. Bug?"** No. Since
   ADR 0105's 2026-11-13 amendment `MA` is the major triad and the major scale is `io` (Ionian); no
   old name is an alias, and the reinterpretation is not diagnosed. Sources: ADR 0105 (amendment
   table), CONTEXT.md Interval Set, `docs/functions.md` "Interval Sets", `lang/src/atom.rs`
   `define_interval_sets!`.
6. **Cross-topic (EVAL): "Can I keep a melody in a Comment, `|| C4D4E4 ||`, and have a Track read
   it?"** Yes. A Comment closes at the next `||`; its text Cells are data a Read, Track or Copy reads
   as uncovered Cells; a pair touching a delimiter is partial input and diagnoses. The Track's read
   is ordered after any writer of the selected pair at its Turn (EVAL). Sources: ADR 0074, 0067 (and
   0070 amendment), ADR 0032 (EVAL); `orcvs/src/source/tick/track.rs`
   `track_reads_a_pair_of_a_comments_text`, `read.rs` `a_comment_closer_at_the_selected_pair_diagnoses`.

## Final rule mapping (drafting phase)

Drafted in `docs/requirements/language.md`. Every retained row above lands in the rule whose
provisional ID it carries; each provisional ID is final, and no rules were merged. Where a row
names two IDs (for example "LANG-220 / LANG-230"), the clause is stated once in each named rule
for its own part. Changes applied from `decisions.md`:

- D3: the Pitch family rows (CONTEXT.md Pitch Function and `.scratch/chords-and-pitch/spec.md`,
  previously "open") are retained as LANG-450, cited "ruled 2026-11-10, no ADR". The `#` family
  also appears in LANG-180's family list.
- D4: LANG-130 states ADR 0021's rule; the reversal is listed under Open decisions (C1).
- D7: ADR 0005 and the `@<`/`@>` clause of ADR 0017 follow ADR 0070 (C2, C3); the document cites
  neither.
- C4: LANG-250 cites ADRs 0061 and 0028, not 0029.
- C6: listed under Open decisions; LANG-300 states that a Chord Play's root is a plain Note.
- The "not built" gap for LANG-460 (`!$`) is listed under "Accepted but not built". Build status
  is otherwise not recorded in the document.

In the document, rules are ordered by heading rather than by ID: LANG-440 and LANG-460 sit under
"Function families", LANG-250–270 under "Answers, nesting and Returns", LANG-360–380 and LANG-450
under "Numeric, pitch and random Functions", LANG-390–410 under "Chords", and LANG-420–430 under
"Scope of the vocabulary".
