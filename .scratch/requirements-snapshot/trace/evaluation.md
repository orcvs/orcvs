# Trace — Evaluation (EVAL)

Pinned source: `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4`. Inventory only; rule IDs are provisional.
Owned ADRs: 0003, 0004, 0006, 0009, 0012, 0014, 0020, 0028, 0031, 0032, 0034, 0036, 0037, 0057,
0060, 0065, 0066, 0068, 0077. Paths are repository-relative. "ss" = source-stated, "pr" = proposed.

## Source disposition

### Wholly superseded ADRs

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| ADR 0031 | Row-major turns against working Source; Bang cleanup turns | superseded | by ADR 0032 (and 0034 for the binding seam); "No implementation of this candidate survives" | ss (0031 status) |
| ADR 0066 | Unwritten operand leaves its Function pending | superseded | by ADR 0069 ("Orcvs has no pending state") | ss (0066 status) |

### ADR 0003 — Source Snapshot is the complete language state

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0003 ¶2 | Snapshot is complete initial state; no hidden persistent state in Interpreter or Playback Engine; behaviour reproducible from Snapshot + Tick | retained | EVAL-010 Source Snapshot is the whole language state | ss (restored by 0072) |
| 0003 ¶2; 0072 | Identical Snapshots at the same absolute Tick plan the same Tick | retained | EVAL-011 Determinism | ss (0072 status: "restores 0003") |
| 0003 ¶2 | Frozen-read / timing / Bang-lifetime clauses | superseded | by 0032 (working Source), Bang lifetime by 0072 | ss (0003 header) |
| 0003 ¶2–3 | Function Atom returned at T is generated code; executes only from a later Snapshot; original-anchor replacement exception (0034) | retained | EVAL-012 Generated code waits; exception in EVAL-040 | ss |
| 0003 ¶3 | Pending encodings and "typed nested answers" are Tick-local | retained (amended) | EVAL-034; nested answers are encodings per 0061 | ss |

### ADR 0004 — Source Functions plan spatial effects

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0004 ¶2 | Source Function reads working Source at its Turn; may resolve any Cell as a Portal | retained | EVAL-020 / EVAL-030 | ss (0032) |
| 0004 ¶2 | Source-writing Function validates its whole bundle before any write; one invalid destination rejects the bundle | retained | EVAL-033 Source Bundle validated whole | ss |
| 0004 ¶2 | Clears are spaces written to Source, not Empty values | retained | EVAL-033 | ss |
| 0004 ¶3 | Writes keep producer anchor and emission order; later write wins at shared Cell | retained | EVAL-032 Cell-wise conflict order | ss |
| 0004 ¶3 | Admitted change visible to later Turns; an earlier root is never revisited | retained | EVAL-020 / EVAL-052 | ss |
| 0004 ¶3 | Source-reading Functions may nest; Source-writing and external-output Functions are root-only | redirected | LANG (nesting / root-only by declared kind, 0016, 0028) | pr |
| 0004 ¶1 | Timing / ordering / Bang-lifetime clauses | superseded | by 0032 | ss (0004 header) |

### ADR 0006 — Bang activation includes Self-Banging Functions

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0006 ¶3 | Bang is `**`; one canonical spelling per Function | redirected | LANG (spellings, Bang value 0071/0072) | pr |
| 0006 ¶3 | Ordinary root inert until Bang activation; Self-Banging is intrinsically activated | retained (conflict) | EVAL-050 Activation source — see Conflict C1 | ss / conflict |
| 0006 ¶4 | Aligned cardinal anchors N (x,y-1), S (x,y+1), W (x-2,y), E (x+2,y); only a complete root activates | retained | EVAL-051 Bang activation geometry | ss |
| 0006 ¶4 | A Portal may deliver Bang directly to a root without overwriting it | superseded | by 0071 (Bang travels as `**`, overwrites, activates aligned roots); see C3 | pr |
| 0006 ¶4 | Activation recursively includes nested Functions | retained | EVAL-042 | ss |
| 0006 ¶4 | Starting-Snapshot `**` cleared before evaluation, activates nothing | superseded | by 0072 (standalone `**` is the Bang Function) | ss (0006 header) |
| 0006 ¶4 | Invalid operand spellings (typed `**`) neither activate nor are cleaned up | retained (cleanup half superseded) | EVAL-051; cleanup gone with 0072 | ss |
| 0006 ¶5 | One row-major pass decides whether a root has a turn | superseded | by 0032 dependency order | ss (0032 status) |
| 0006 ¶5 | Multiple Bangs never make one root evaluate twice; passed root not revisited | retained | EVAL-052 One Turn per Producer | ss |
| 0006 ¶6 | Directional Bang emits matching Self-Banging Function at N/S/W/E offsets; destination complete, empty, in-Grid or diagnose and emit nothing; emitted code first acts next Snapshot | retained | EVAL-055 Emit | ss (amended by 0060) |
| 0006 ¶7 | Advance tests only newly entered Cells; empty → spaces over Span then spelling; blocked/out-of-Grid → `**` over Span | retained | EVAL-054 Advance | ss |
| 0006 ¶7 | Complete aligned root contact delivers Bang | superseded | by 0072 ("contact activation"): blocked mover writes `**`, activates nothing until it fires next Tick | ss |
| 0006 ¶7 | Partial Language Unit contact diagnoses; complete non-root contact adds no diagnostic | retained | EVAL-054 | ss |
| 0006 ¶8 | Collisions add no rule; outcomes follow Turn order (odd gap, even gap, trains, emission vs mover) | retained | EVAL-056 Collisions follow Turn order | ss |
| 0006 ¶9 | Halt locks root one row south before its Turn; no Source change; empty target no-op; occupied non-root diagnoses; not revisited; suppressed Halt locks nothing | retained | EVAL-053 Halt | ss |
| 0006 ¶9 | Halt over occupied non-root diagnoses *as applied to `**`* | superseded | by 0072: Halt over `**` locks it | ss |
| 0006 header | Placement admission vs value readiness | retained (amended) | by 0060 → EVAL-025 | ss |

### ADR 0009 — Portals resolve Tick Plan destinations

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0009 ¶2 | Portal is one internal Cell destination resolved during a Tick; not a value or persistent state | retained | EVAL-030 Portal is Tick-local | ss (reaffirmed 0077) |
| 0009 ¶2 | Future addressing (infinite canvas) changes resolution only; closed by 0049 (pair of Numbers → Position or refusal) | redirected | LANG (Positions, 0049) | ss |
| 0009 ¶3 | Bundle validated whole before any write; expanded in emission order; clear is a space | retained | EVAL-033 | ss |
| 0009 ¶3 | Ordinary result writes only its current encoding; never clears a stale tail | retained | EVAL-036 No stale-tail clear | ss |
| 0009 ¶3 | Admitted writes join ADR 0020 order and are visible to later Turns before atomic commit | retained | EVAL-032 / EVAL-020 | ss |
| 0009 | Sequence clauses (intact Sequence, empty Sequence) | superseded | by 0063 | ss |

### ADR 0012 — Playback Tick inputs and Portal feedback

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0012 ¶2 | Absolute Tick is an interpretation input supplied outside Source; unsigned, +1 per Tick | retained | EVAL-060 Absolute Tick input | ss |
| 0012 ¶2 | First Playback Tick is 0 | superseded (amended) | by 0076: only first run / after Rewind; stop resumes at next Tick → PLAY | ss |
| 0012 ¶2 + amendments | Clock `~. division steps` = floor(Tick/division) % steps; zero diagnoses (division first); wide integer | retained | EVAL-061 Clock formula | ss |
| 0012 ¶2 + amendments | Every `~* division steps` Bangs when Tick % (division*steps) == 0, incl. 0; product in wide integer | retained | EVAL-062 Every formula | ss |
| 0012 ¶3 | Euclidean bucket distribution and phase | superseded | by 0012's own 2026-11-14 amendment (Tidal `bjorklund`) | ss |
| 0012 amendment 2 | Euclidean Bangs at Tidal onset for position t % steps; zero steps diagnoses before hits>steps; zero hits silent | retained | EVAL-063 Euclidean = Tidal | ss |
| 0012 ¶4 | Increment/Slew feedback: Input and Output Portal are the same site, read from working Source at Turn incl. earlier same-Tick writes; feedback is Source, not a `TickInputs` channel | retained | EVAL-064 Feedback lives in Source | ss |
| 0012 ¶4 | Portal input validated like cell operands; operand faults precede Portal faults | retained | EVAL-064 | ss |
| 0012 ¶5 | Empty feedback Cells = `00`; other present Language Unit diagnoses; Increment (prev+amount)%modulus wide; zero modulus diagnoses | retained | EVAL-065 Increment arithmetic | ss |
| 0012 ¶5 | Slew moves at most `amount`, never overshoots; non-negative branch subtraction; `00` holds | retained | EVAL-066 Slew arithmetic | ss |
| 0012 ¶6 | Scalar exception to Sequence broadcasting; "a Note ... diagnoses" | superseded / narrowed | Sequence part by 0063; Note-diagnosis narrowed by 0024/0034 contextual decoding (Source `C4` reads as Number; `G4` diagnoses) | ss (0063) / pr (Note) |
| 0012 ¶1 | Same-Tick Bang delivery of pulses; non-Bang answer | retained (amended) | Bang delivery EVAL-051; non-Bang answers a Clear per 0073 → LANG | ss |
| 0012 amendments | Names Every, Slew, `division`; rationale | redirected | LANG / docs/functions.md (spellings, Role names); rationale historical | pr |

### ADR 0014 — Directional Bang and Jump preserve performative behaviour

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0014 ¶2–3 | Emitted Self-Banging Function first acts next Snapshot; Advance is one validated bundle; newly entered Cells; `**` on refusal | retained | EVAL-054 / EVAL-055 (duplicates 0006) | ss |
| 0014 ¶3 | Complete aligned root contact delivers Bang | superseded | by 0072 (0014's status line does not record it); see C2 | pr |
| 0014 ¶4–6 | Jump (now Copy) spellings, Portals, composition, empty/partial input | redirected | LANG (Copy family 0070, 0071, 0067) | pr |
| 0014 ¶7 | Jump relays Bang without overwriting | superseded | by 0071 | ss |
| 0014 ¶8 | Producer below consumer reaches it same Tick; each root at most once | retained | EVAL-020 | ss |
| 0014 ¶8 | Same-Tick cycle publishes nothing for the Tick | superseded (amended) | by 0065 → EVAL-071 | ss |
| 0014 ¶9 | Halt pointer to 0006 | retained | EVAL-053 | ss |

### ADR 0020 — Order Tick effects by Source Position

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0020 ¶2 | Scheduled expiries dispatched before the Tick Plan's new Play Commands | redirected | PLAY (Note Off schedule) | pr |
| 0020 ¶3 | Candidates from starting Snapshot, ordered by dependency; Position breaks ties; one producer Position = anchor; emission order kept | retained | EVAL-020 / EVAL-021 / EVAL-032 | ss |
| 0020 ¶4 | Only original Function candidates get Turns; generated values/Bangs visible to dependents | retained | EVAL-012 | ss |
| 0020 ¶4 | Source-resident `**` is display, cleared before evaluation | superseded | via 0032 → 0072 | ss |
| 0020 ¶5 | Whole-destination validation; Cell-wise conflict; resulting malformed Source allowed | retained | EVAL-031 / EVAL-032 | ss |
| 0020 ¶6 | Mover: spaces before spelling; blocked → `**` | retained | EVAL-033 / EVAL-054 | ss |
| 0020 | Sequence clauses | superseded | by 0063 | ss |

### ADR 0028 — The Orcvs evaluation machine

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0028 ¶2 | Evaluator evaluates one Function at a time over resolved operands; push last-to-first; pop by signature; answer returned, never pushed | retained | EVAL-080 Evaluator invocation | ss |
| 0028 ¶3 | No memory of its own: stack per evaluation; no registers/heap/PC; "not a VM" | retained | EVAL-080 / EVAL-010 | ss |
| 0028 ¶4 | One instruction-set declaration (`define_functions!`) from which spellings, kinds, signatures, roles, parser are derived; dispatch hand-written, exhaustive | retained | EVAL-082 Single declaration | ss |
| 0028 ¶4 | Conversion identity exception | superseded | by 0061 | ss |
| 0028 ¶5 | Per-Function test with distinct operand values guards transposition | retained | EVAL-083 Operand-order test obligation | ss |
| 0028 ¶6 | Value or effect, never both/neither; terminal root-only | redirected | LANG (Function entry, 0016, 0029/0063) | pr |
| 0028 ¶6 | Root that answers no value answers the absence marker | superseded | by 0073 (no Absence Marker) | ss |
| 0028 ¶7 | Row bounds Expression extent; `EXP_LEN` retired | redirected | LANG (parse, 0033/0043) | pr |
| 0028 ¶8 | Operand Stack holds one Function's operands; arity refused before stack exists; capacity = widest signature; checked push diagnoses | retained | EVAL-081 Operand Stack bound | ss |
| 0028 ¶9 | Value model / declaration form left open | superseded / open | value model settled by 0063; declaration form stays open (constrains only singularity) | ss |
| 0028 status | Terminal answers one Play Command (chord plays one per tone) | redirected | PLAY / LANG (0103) | ss |

### ADR 0032 — Schedule Tick execution by dependency

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0032 ¶3 | Note + Bang supplied in T play in T, incl. producer below consumer; parse, establish data and activation dependencies, execute each root once, publish | retained | EVAL-020 Tick pipeline | ss |
| 0032 ¶3 | Position breaks ties only; fixed upward Portal is an ordinary dependency, not delayed feedback | retained | EVAL-021 | ss |
| 0032 ¶4 | Parsing owns identities, operand positions, nested connections, Bang validity; missing slots kept for current-Tick writes | retained | EVAL-020 (parse-owned structure) | ss |
| 0032 ¶4 | Scheduling behind the Source Tick interface; Playback receives ordered commands for T | retained | EVAL-020; boundary with PLAY | ss |
| 0032 ¶5 | Bang activates neighbours in its production Tick | retained | EVAL-051 | ss |
| 0032 ¶5 | Manual `**` is a no-op; display does not re-activate; Bang not a standalone Function | superseded | by 0072 (value half stands) | ss |
| 0032 ¶6 | Supplier settles before consumer even when absent; absence of Bang ≠ unfinished dependency | retained | EVAL-022 | ss |
| 0032 ¶6 | Unwritten inline operand → Function invalid, no write | redirected | LANG (0069/0108) | ss |
| 0032 ¶7–8 | Initial restrictions (competing writers, partial inputs, variable width) | superseded | by 0034, 0036, 0063 | ss |
| 0032 ¶8 | Deferred: newly anchored code, computed destinations, delayed feedback, activation/output-changing replacement | retained (partly) / redirected | Computed destinations now exist for dynamic Portals (0067 → LANG); rest remains deferred, recorded in EVAL-040 / EVAL-012 | ss |
| 0032 ¶9 | Value roots calculate each Tick; terminal roots need Bang ("first implementation") | retained (conflict) | EVAL-050 — see C1 | ss |
| 0032 ¶9 | Atomic Tick publication; Source/Playback separation (0001) | retained | EVAL-020; separation → PLAY | ss |

### ADR 0034 — Execute against live typed expressions

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0034 ¶2 | Parser output is the typed structure for rendering and execution; no reparse between executions | retained | EVAL-020 | ss |
| 0034 bullet 3 | Writer that ordinarily overwrites a Snapshot computation executes first | retained | EVAL-024 Overwrite ordering | ss (amended 0060, 0067) |
| 0034 bullet 3 | Overwrite reaching an attempted computation is an ordering bug | superseded (amended) | by 0065 (refuse that Turn's write, continue) and 0067 (dynamic write = next-Tick feedback) | ss |
| 0034 bullets 4–5 | Return and spatial write are the same encoding, decoded by receiving operand | redirected | LANG (0061 Return, Spatial Output); delivery timing in EVAL-034 | pr |
| 0034 bullet 6 | Cycle stops its Expressions and dependants; others publish; no partial, no delay | retained | EVAL-071 | ss (0065) |
| 0034 bullet 7 | Nested Portal does not independently activate; inactive parent or replacement suppresses child and its write | retained | EVAL-042 | ss |
| 0034 bullet 8 | Refused spatial write keeps the answer for parent Return | retained | EVAL-035 | ss |
| 0034 bullet 9 | Failed producer: no write; spatial consumers proceed on surviving encodings; nested consumer fails | retained | EVAL-035 | ss |
| 0034 bullet 10 | Parent error does not undo child's write | retained | EVAL-035 | ss |
| 0034 bullet 11 | Spatial writes compose Cell-wise; all writers settle before consumer | retained | EVAL-032 / EVAL-022 | ss |
| 0034 bullet 12 | Pending encodings decoded only at consumption by declared literal type | retained | EVAL-034 Pending encodings decode at consumption | ss |
| 0034 bullets 13–17 | Original-anchor Function Replacement retains positions/connections; replacement signature decodes retained literals; arity/type checked; Grid-positioned, malformed results allowed; cross-boundary parameter updates | retained | EVAL-040 Function Replacement | ss |
| 0034 bullet 18 | Value replacing nested Function replaces that input; replacing top-level suppresses its Turn | retained | EVAL-041 | ss |
| 0034 bullets 19, 23 + deferred list | Function-valued output, activation/output-changing replacement, changed destinations deferred | retained (as deferral) | EVAL-040 note; open in 0034 "deferred questions" | ss |
| 0034 ¶ struck sentence | No `cfg(test)` substitution in shipped scheduler; tests build inputs below planning entry point | redirected | engineering policy (AGENTS.md test-only inputs) — out of scope | pr |
| 0034 Render Frame amendment | Render Frame carries `Option<Token>`; Glyph retired; blank-Cell paint | redirected | CONS (Render Frame, paint) | pr |
| 0034 ¶ revisions | Revision of 0024/0031/0032 restrictions; prototype history | historical | reasoning | pr |

### ADR 0036 — Reserve result Cells before their width exists

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0036 ¶2 | Dependency edge names every Cell a write touches; schedule fixed before evaluation; reserve what a result can reach | retained | EVAL-023 Reservation | ss |
| 0036 ¶3–5 | Sequence-capability, rest-of-row reservation, pervasion columns | superseded | by 0063 | ss |
| 0036 ¶5 | Scalar computation reserves its pair and refuses other encoding widths | retained | EVAL-031 | ss |
| 0036 ¶6 | Reservation orders Turns; admitted write decides suppression/replacement/activation; admitted ⊆ reserved | retained | EVAL-023 | ss |
| 0036 ¶8 | Replacement that changes reserved width refused | superseded | moot under 0063 (every value Function reserves one pair); CONTEXT Function Replacement: "width ... is not among them" | pr |
| 0036 status | Nested Function reserves nothing | superseded | by 0061 (nested writes through its own Output Portal) | ss |
| 0036 rejected / consequences | Rest-of-row-for-all; late diagnosis | historical | reasoning | pr |

### ADR 0037 — A late Tick loses its turn

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0037 ¶3 | Ticks due on a Tick Grid from the run's start deadline; missed deadlines skipped; resume at first deadline ahead of wake; deadline at wake instant goes with the stall | redirected | PLAY (Playback clock; "completes ADR 0002") — see Q2 | pr |
| 0037 | One Overrun per stall, names the deadline, no user message | redirected | PLAY | pr |
| 0037 | `is_overrun` declines a Tick a whole period late; late-within-period executes | redirected | PLAY | pr |
| 0037 | Retune starts a new grid from the last executed Tick's deadline | redirected | PLAY | pr |
| 0037 | Both targets run one loop (`next_scheduled_at`); only the wait differs | redirected | PLAY | pr |
| 0037 | History of divergence; rejected backlog/moving-grid rules | historical | reasoning | pr |

### ADR 0057 — A Tick commits only the revision it planned

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0057 ¶1 | Plan from a snapshot taken under one short read guard (Grid, Cells, `Arc<LanguageMap>`, `RevisionId`); commit compares RevisionId and writes under one write guard | retained | EVAL-090 Optimistic plan, revision-checked commit | ss |
| 0057 ¶3 | Every write mints a RevisionId; mismatch refuses the plan whole (writes, Play Commands, diagnostics, locks) | retained | EVAL-091 Refused plan publishes nothing | ss |
| 0057 ¶4 | Refused Tick replanned at the same absolute Tick; counter advances only after `execute` | retained | EVAL-091 | ss (CONTEXT Live Editing) |
| 0057 ¶5 | Three optimistic attempts, then one under a write guard that cannot be refused | retained | EVAL-092 Bounded retries | ss |
| 0057 ¶5 | `is_overrun` runs before execute; a Tick slowed by retries still commits | redirected | PLAY (cross-ref EVAL-092) | pr |
| 0057 ¶7 | Snapshot shares Cells copy-on-write; uncontended Tick copies nothing | retained | EVAL-093 No copy on an uncontended Tick | ss |
| 0057 alternatives | Merge, unbounded retry, skip, upgradeable lock | historical | reasoning | pr |

### ADR 0060 — Value inputs wait; placement tests occupancy

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0060 rule 3a | Every potential writer whose Reservation covers a value input settles before the consumer; failed/no-write supplier still settles; nested input waits for Return; nested Output Portal is a potential writer; Input Portal read dependencies remain | retained | EVAL-022 Value inputs wait | ss |
| 0060 rule 3b | Advance/Emit read working Grid at Turn; no wait for occupant; occupancy creates no edge; refusal not retried | retained | EVAL-025 Placement tests occupancy at its Turn | ss |
| 0060 ¶ Advance/Emit | Advance tests entered Cells; Emit tests whole destination; both need in-Grid empty | retained | EVAL-054 / EVAL-055 | ss |
| 0060 ¶ vacated | Placement into vacated Cells overwrites no computation; executed-computation guard for Snapshot computations; later overwrite of a mover's new spelling is Source content | retained | EVAL-026 Vacated Cells | ss |
| 0060 ¶ contest | Emission/mover contest follows actual schedule; follower cannot demand leader run first | retained | EVAL-056 | ss |
| 0060 ¶ cycles | Cycles stop every Expression they reach | retained | EVAL-071 (0065 amendment) | ss |
| 0060 ¶ cycles | Emission/activation operand cycle is a separate investigation | open | `.scratch/placement-semantics/issues/04` (ready-for-agent) | ss |
| 0060 ¶ rejected | Snapshot-only admission; simultaneous vacancy solving; Orca parity not required | historical | reasoning | ss |

### ADR 0065 — An error never stops the performance

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0065 ¶3 | No error, in Source or Orcvs, rejects a whole Tick | retained | EVAL-070 Error containment | ss |
| 0065 ¶4 | Cycle stops its Expressions and every Expression depending on them (reads, activation, lock, covering write); others ordered as if absent | retained | EVAL-071 Cycle stops what it reaches | ss |
| 0065 ¶5 | No part of a stopped Expression takes a Turn; nothing delayed | retained (amended) | EVAL-071; discovered cycles amended by 0068 | ss |
| 0065 ¶6 | Each cycle diagnosed once at first computation in Parser order; waiting roots diagnosed only if activation reaches them | retained | EVAL-072 Cycle diagnostics | ss |
| 0065 ¶7 | Ordering defect refuses that Turn's write/lock, diagnosed, Tick continues | retained | EVAL-024 | ss |
| 0065 options | Whole-Tick reject, delay, arbitrary break | historical | reasoning | ss |

### ADR 0068 — A discovered cycle preserves completed Turns

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0068 | Cycle found at a Turn stops remaining computations in affected Expressions and dependants (incl. ready nested siblings); completed Turns keep Effects; consumer of completed Turn not stopped for that reason; no rollback, no delay | retained | EVAL-073 Discovered cycle keeps completed Turns | ss |

### ADR 0077 — A committed Tick records its Portal accesses

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0077 status | Portal stays Tick-local; record is an observation | retained | EVAL-030 | ss |
| 0077 Decision ¶1 | Tick Portals planned with the Tick Plan; immutable ordered list; execution sole writer; unconditional; beside not in plan | retained | EVAL-100 Tick Portals travel beside the plan | ss |
| 0077 ¶2 | Owner = anchor, subtree Cells, Function after any replacement, captured during execution | retained | EVAL-101 Entry owner as execution held it | ss |
| 0077 ¶3–5 | Read recorded after writer wait (once on retry; nothing when refused pre-interpretation); write per delivery site (Span or Write Refusal); lock verdicts; Source Bundle parts | retained | EVAL-102 What a Turn records | ss |
| 0077 ¶6 | Order = settlement order; within Turn read, writes, lock/bundle parts | retained | EVAL-103 | ss |
| 0077 ¶7 | Input/Output Span constructors private to execution | retained | EVAL-104 Spans minted only by execution | ss |
| 0077 ¶8 | Committed through the one commit path; refused attempt drops it; each commit replaces (empty if none) | retained | EVAL-105 | ss |
| 0077 ¶9 | Revision answers record as Current/Retained; stop keeps it; new/loaded Source has none; not persisted | retained | EVAL-106 Record freshness | ss |
| 0077 ¶ edited Cells | One bit per Cell, cleared by commit, shared copy-on-write | retained | EVAL-106 (implementation constraint) | ss |
| 0077 ¶10–12 | Render Frame roles, Output > Input > Locked precedence, retained-entry paint eligibility | redirected | CONS (Render Frame paint) | pr |
| 0077 consequences | Public types; allocation per Tick within `source_execute_tick*` benches | retained | EVAL-100 note | ss |
| 0077 options | Recorder, folding into plan, etc. | historical | reasoning | ss |

### CONTEXT.md entries (behaviour clauses)

Rust comments citing CONTEXT.md by entry in this topic: `orcvs/src/source/tick/execution.rs:630`
(Function Replacement), `orcvs/src/source/encoding.rs:3` (Spatial Output, quote current),
`orcvs/src/source/portal.rs:15` (Portal; quote "neither a language value, Source content, nor
persistent state" is stale — entry now reads "neither a language value nor persistent state"),
`orcvs/src/source/portal.rs:63` (Halt diagnose policy), `orcvs/src/source/tick/halt.rs:91` (Halt;
quote "establishes a dependency that locks ... before that root can execute" no longer in the entry).

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| CONTEXT Source Snapshot | Whole initial state; nothing about a Bang remembered outside Source; identical Snapshots plan the same Tick | retained | EVAL-010 / EVAL-011 | ss |
| CONTEXT Tick | Sixteenth note; four Ticks a beat | redirected | PLAY (musical time / BPM) | pr |
| CONTEXT Tick | Cycle stops remaining Turns in reached Expressions; completed Turns keep Effects | retained | EVAL-071 / EVAL-073 | ss |
| CONTEXT Tick Plan | Complete deterministic outcome: activation routing, writes, Play Commands, Network Sends, diagnostics | retained | EVAL-011 / EVAL-020 | ss |
| CONTEXT Turn, Producer | One Turn per original Producer after dependencies settle; Bang Function and Halt are Producers | retained | EVAL-052 | ss |
| CONTEXT Reservation | Every value Function reserves the pair at each write Portal; bundle Spans; Output Portal Reservation highlighted | retained | EVAL-023 (highlight → CONS) | ss |
| CONTEXT Dependency | Producer before every computation its Reservation, lock or Bang reaches; nested before parent; joined root brings its Dependencies | retained | EVAL-022 / EVAL-052 | ss |
| CONTEXT Contact | Covered Function + nested; anchor Contact = Replacement; same rule before and during Tick | retained | EVAL-024 / EVAL-040 | ss (CONTEXT only) |
| CONTEXT Effect | Execution order then emission order; whole-destination validation; Cell-wise | retained | EVAL-031 / EVAL-032 | ss |
| CONTEXT Function Replacement | Refused unless four facts agree (value-ness, activation source, can-Bang, declared Source write); compared against running Function; refusal names the fact | retained | EVAL-040 | ss (CONTEXT; derives from 0032/0034) |
| CONTEXT Portal / Output Portal / Input Portal | Static vs dynamic; dynamic writer goes before what does not feed it; earlier read sees write next Tick | redirected | LANG (0067) with scheduling cross-ref — see Q1 | pr |
| CONTEXT Pending Operand Encoding | Characters await interpretation at receiving Function | retained | EVAL-034 | ss |
| CONTEXT Source Function / Source Bundle | Value reads wait; placement tests occupancy; own Span first; refusal `**` vs diagnose | retained | EVAL-025 / EVAL-033 | ss |
| CONTEXT Self-Banging / Directional Bang | Inert vs intrinsic; blocked → `**`, fires next Tick; meets Bang Function in Source order | retained | EVAL-050 / EVAL-054 / EVAL-055 | ss |
| CONTEXT Halt Function | Lock ordered before target; holds only if a Bang that executed activated Halt first; Halt above its own `**` is a Missed Bang; Halt locks `**` | retained | EVAL-053 (Missed Bang → LANG 0072) | ss |
| CONTEXT Clock/Every/Euclidean/Increment/Slew | Formulas, wide integers, validation order, feedback site | retained | EVAL-061..066 (spellings/Roles → docs/functions.md) | ss |
| CONTEXT Evaluator / Operand Stack | One evaluation; stack bounded by widest signature | retained | EVAL-080 / EVAL-081 | ss |
| CONTEXT Live Editing | Edit affects next un-snapshotted Tick; edit during planning re-snapshots at same Tick | retained | EVAL-091 | ss |
| CONTEXT Tick Portals | Record rules, freshness; paint precedence | retained / redirected | EVAL-100..106; paint → CONS | ss |
| CONTEXT Tick Grid / Overrun | Deadline grid, one Overrun per stall | redirected | PLAY (see Q2) | pr |

### `.scratch/` rulings not in an ADR

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| `.scratch/bang-function/issues/08-*` (resolved) | A moving bundle orders no Contacts (mover does not precede the root it contacts) | retained | EVAL-025 (consistent with 0060 "occupancy alone creates no ordering edge") | pr |

## Evidence and gaps

Paths under `orcvs/src/source/tick/` abbreviated `tick/`. "Inspected" = assertions read.

| Rule | Code | Tests | Status |
| --- | --- | --- | --- |
| EVAL-010 Snapshot is whole state | `lang/src/stack.rs` `Stack` (per evaluation); `orcvs/src/source/planning.rs` snapshot | `tick/bang_function.rs` `identical_snapshots_at_the_same_tick_plan_the_same_tick` (inspected) | built and covered |
| EVAL-011 Determinism | `tick/execution.rs` `execute`; `planning.rs` | same; `planning.rs` `a_tick_planned_from_a_snapshot_is_the_tick_planned_under_the_lock`; `tick/tick_functions.rs` `moving_a_random_changes_its_stream_and_identical_inputs_reproduce_it` | built and covered |
| EVAL-012 Generated code waits | `tick/execution.rs` (no new anchors scheduled) | `tick/movers.rs` `an_emitted_self_banging_function_first_moves_on_the_following_tick`; `tick/replacement.rs` `live_replacement_checks_retained_arity_and_never_runs_new_anchors` | built and covered (names only) |
| EVAL-020 Tick pipeline | `tick/ordering.rs` `Progress`, `take_ready`; `tick/execution.rs` `execute` | `tick/ordering.rs` `a_dependency_goes_before_grid_order`; `tick/live_editing.rs` `live_cross_boundary_chain_uses_lower_producers`; `tick/halt.rs` `halt_is_interpreted_once_per_tick` | built and covered |
| EVAL-021 Position breaks ties | `tick/ordering.rs` `take_ready` | `tick/ordering.rs` `independent_nodes_take_their_turns_in_grid_order` | built and covered |
| EVAL-022 Value inputs wait | `tick/dependencies.rs` | `tick/live_editing.rs` `partial_writers_settle_before_consumption`, `live_pending_note_decodes_only_after_all_writers_settle`; `tick/movers.rs` `a_mover_supplies_a_claimed_operand_before_its_consumer` | built and covered |
| EVAL-023 Reservation | `tick/dependencies.rs`; `orcvs/src/source/portal.rs` | `tick/conflicts.rs` `write_claims_answer_overlapping_reservations`; `tick/nesting.rs` `a_nested_output_portal_orders_an_earlier_consumer_after_its_writer` | built and covered |
| EVAL-024 Overwrite ordering / defect | `tick/dependencies.rs` `refusal`; `tick/execution.rs:919,963` | `tick/live_editing.rs` `a_late_spatial_write_is_refused_and_the_tick_continues_even_when_the_earlier_turn_failed`; `tick/halt.rs` `a_late_halt_lock_is_refused_and_the_tick_continues` | built and covered |
| EVAL-025 Placement occupancy | `tick/execution.rs` `deliver_source_effect` (`self.working.vacant`) | `tick/movers.rs` `an_emission_waits_for_no_occupant_to_leave`, `a_mover_is_blocked_by_an_earlier_overwrite` | built and covered |
| EVAL-026 Vacated Cells | same | `tick/movers.rs` `an_emission_enters_cells_a_mover_vacated_earlier_in_the_tick`, `a_movers_new_cells_can_be_overwritten_after_its_turn` | built and covered |
| EVAL-030 Portal Tick-local | `orcvs/src/source/portal.rs` (module doc: no Atom carries one; not serialized) | none specific (structural) | built, no verified test |
| EVAL-031 Whole-destination validation | `portal.rs` `admit`; `tick/execution.rs` `width` | `tick/conflicts.rs` `test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write`; `tick/live_editing.rs` `live_rejected_complete_write_changes_no_cell`; `tick/outputs.rs` `a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick` | built and covered |
| EVAL-032 Cell-wise conflict | `tick/execution.rs` `write` | `tick/conflicts.rs` `test_later_effects_win_cell_conflicts_independently`, `the_cells_of_two_overlapping_writes_are_contested_one_by_one`, `test_play_commands_and_diagnostics_keep_producer_and_emission_order`; `tick/live_editing.rs` `live_writes_agree_with_an_independent_cell_overlay` (property) | built and covered |
| EVAL-033 Source Bundle | `tick/execution.rs` `deliver_source_effect`, `fire` | `tick/movers.rs` `a_self_banging_function_advances_its_whole_span_by_one_cell`, `a_move_that_leaves_the_grid_replaces_its_own_span_with_bang` | built and covered |
| EVAL-034 Pending encodings | `tick/inputs.rs`; `lang` `Token::decode` | `tick/live_editing.rs` `live_pending_note_decodes_only_after_all_writers_settle`; `tick/nesting.rs` `a_return_and_a_portal_decode_the_same_characters_the_same_way` | built and covered |
| EVAL-035 Failure locality | `tick/execution.rs` `deliver_value`, `project_value` | `tick/live_editing.rs` `live_child_write_survives_parent_failure_and_rejected_portal_keeps_its_return`, `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers`; `tick/outputs.rs` `an_activated_consumer_uses_surviving_cells_after_supplier_failure` | built and covered |
| EVAL-036 No stale-tail clear | `tick/execution.rs` | `tick/tick_functions.rs` `increment_writes_its_current_encoding_and_leaves_a_stale_tail` (inspected) | built and covered |
| EVAL-040 Function Replacement | `lang/src/atom.rs` (four facts, `:1440`); `tick/dependencies.rs` `Refusal::Replacement` | `tick/replacement.rs` (13 tests incl. `a_replacement_that_changes_only_the_activation_source_is_refused`, `live_the_second_replacement_at_one_anchor_replaces_the_first`, `live_function_replacement_keeps_nesting_and_reinterprets_only_literals`); `tick/live_editing.rs` `.+0021` case | built and covered |
| EVAL-041 Value replaces computation | `tick/execution.rs` | `tick/live_editing.rs` `a_value_replaces_nested_computation_and_preserves_next_tick_source` | built and covered |
| EVAL-042 Nested activation | `tick/ordering.rs`; `tick/execution.rs` | `tick/live_editing.rs` `live_inactive_ownership_and_silent_terminal_output_are_independent`; `tick/replacement.rs` `live_deep_and_top_level_replacement_suppress_descendant_portals` | built and covered |
| EVAL-050 Activation source | `lang/src/atom.rs` `ActivationSource` (value = Intrinsic; effect = Bang, except movers and `**`) | `tick/halt.rs` `an_active_halt_locks_the_complete_root_one_row_south` (inspected: Add "intrinsically active"); `tick/movers.rs` `an_inert_directional_bang_function_emits_nothing` | code differs from 0006 literal text — see C1; no tracking issue found |
| EVAL-051 Bang geometry | `tick/dependencies.rs` `bang_root`; `tick/bang_activation.rs` | `tick/bang_activation.rs` `a_bang_activates_its_aligned_neighbours_and_no_further_root`, `a_bang_rejected_in_a_typed_operand_never_activates` | built and covered |
| EVAL-052 One Turn; joined roots | `tick/ordering.rs` `activate`, `join` | `tick/ordering.rs` `a_root_that_joins_brings_its_dependencies_into_the_rest_of_the_tick`, `a_turn_that_waits_is_taken_again_after_its_writers` | built and covered |
| EVAL-053 Halt | `tick/execution.rs` `lock_portal`; `portal.rs` `Occupancy` | `tick/halt.rs` (22 tests incl. `an_empty_halt_target_is_a_noop`, `an_occupied_non_root_halt_target_diagnoses`, `an_active_halt_locks_a_bang_function`, `a_suppressed_halt_does_not_lock_its_own_target`) | built and covered |
| EVAL-054 Advance | `tick/execution.rs` `deliver_source_effect` | `tick/movers.rs` `a_mover_diagnoses_partial_contact_with_a_blocked_movers_bang`; `tick/bang_function.rs` `a_blocked_mover_activates_nothing_and_its_bang_plays_on_the_next_tick` | built and covered |
| EVAL-055 Emit | same | `tick/movers.rs` `an_active_directional_bang_function_emits_its_self_banging_function`, `a_refused_emission_diagnoses_and_writes_no_cell` | built and covered |
| EVAL-056 Collisions | same | `tick/movers.rs` `an_odd_gap_leaves_the_earlier_mover_the_cell_and_the_later_one_bangs`, `converging_movers_resolve_pairwise_in_source_order`, `a_mover_flush_against_a_directional_bang_function_bangs_rather_than_costing_the_tick`, `an_emission_contest_follows_the_order_its_activation_imposes` | built and covered |
| EVAL-060 Absolute Tick input | `lang/src/tick.rs` `Tick`, `TickInputs`; `orcvs/src/playback.rs` | `tick/tick_functions.rs` `the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`, `the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at` (inspected) | built and covered |
| EVAL-061 Clock | `lang/src/functions/tick.rs` | `clock_counts_one_step_per_division_ticks_and_wraps_at_its_steps`, `clock_counts_a_cycle_of_more_ticks_than_a_byte_holds`, `clock_diagnoses_a_zero_division_before_zero_steps` | built and covered |
| EVAL-062 Every | same | `every_bangs_once_per_cycle_beginning_at_the_first_tick`, `every_with_one_step_bangs_once_per_division_ticks_rather_than_every_tick`, `every_counts_a_cycle_product_wider_than_a_byte` | built and covered |
| EVAL-063 Euclidean | same `tidal_bjorklund` | `euclidean_matches_tidals_bjorklund_for_every_pattern_up_to_32_steps` (inspected: full sweep ≤32, sampled to FF via `euclidean_matches_tidals_bjorklund_up_to_a_byte_of_steps`); `tick/tick_functions.rs` `a_tick_function_with_no_cycle_diagnoses_and_writes_nothing` | built and covered (ADR says every steps 01–FF; test samples above 32) |
| EVAL-064 Feedback in Source | `lang/src/functions/tick.rs:251`; `tick/inputs.rs` | `tick/tick_functions.rs` `an_empty_portal_initialises_increment_and_slew_as_zero`, `feedback_reads_a_portal_write_from_an_earlier_turn`, `a_previous_that_is_not_a_number_diagnoses_and_writes_nothing` (inspected); `lang/src/functions/tick.rs` `cell_operand_faults_precede_an_invalid_portal_input` | built and covered |
| EVAL-065 Increment | `lang/src/functions/tick.rs` | `increment_adds_in_a_wider_integer_before_the_modulus`, `increment_diagnoses_a_zero_modulus` | built and covered |
| EVAL-066 Slew | same | `slew_moves_toward_target_without_overshoot`, `slew_holds_when_its_amount_is_zero` | built and covered |
| EVAL-070 Error containment | `tick/execution.rs`; `tick/ordering.rs` | `tick/outputs.rs` `a_row_edge_fragment_diagnoses_its_own_expression_and_leaves_the_tick_playing`; `tick/movers.rs` `a_blocked_pair_of_moves_leaves_the_rest_of_the_grid_running` | built and covered |
| EVAL-071 Cycle stops what it reaches | `tick/ordering.rs` `cycle_closure`, `stall` | `tick/cycles.rs` `a_cycle_leaves_an_independent_expression_publishing`, `live_cycles_stop_only_the_expressions_they_reach` (inspected) | built and covered |
| EVAL-072 Cycle diagnostics | same | `tick/cycles.rs` `each_separate_cycle_is_diagnosed_once`, `an_expression_a_cycle_stops_is_diagnosed_at_its_root_as_waiting`, `an_inactive_root_a_cycle_writes_over_is_not_reported_as_waiting` (inspected) | built and covered |
| EVAL-073 Discovered cycle | `tick/ordering.rs` `continue_ordering` | `tick/track.rs` `a_cycle_discovered_by_a_nested_track_stops_its_sibling` (inspected), `a_late_cycle_preserves_a_completed_nested_operands_write` | built and covered |
| EVAL-080 Evaluator invocation | `lang/src/stack.rs` `Stack::with_operands`, `extract`; `lang/src/interpreter.rs` | `lang/src/stack.rs` `operands_pop_back_in_signature_order` | built and covered |
| EVAL-081 Operand Stack bound | `lang/src/stack.rs` `Stack::new(limit)`, `push` | `no_function_declares_more_operands_than_the_inline_storage_holds`, `an_exhausted_operand_stack_diagnoses_rather_than_panicking`; `tick/nesting.rs` `a_tick_over_a_chain_deeper_than_any_signature_interprets_every_link` | built and covered |
| EVAL-082 Single declaration | `lang/src/atom.rs` `define_functions!` (`:961`, table `:1340ff`); `lang/src/interpreter.rs` match | compile-time (no wildcard arm; `:898` `_ =>` is inside a value match, not dispatch — not inspected further) | built, no verified test (compile-time property) |
| EVAL-083 Operand-order tests | — | `lang/src/functions/math.rs` `the_non_commutative_functions_read_left_and_right_in_signature_order`; `lang/src/functions/mod.rs` `play_carries_each_operand_into_the_role_its_signature_names` (+ timed, mono, CC, bend); `lang/src/functions/chord.rs` `each_operand_reaches_the_role_its_signature_names` | built and covered (Source-text half for Raw Play not verified) |
| EVAL-090 Revision-checked commit | `orcvs/src/source/planning.rs`; `SourceCommander::execute`, `PlannedTick::commit` | `planning.rs` `a_captured_snapshot_holds_no_source_lock`, `an_edit_between_plan_and_commit_is_never_overwritten` (inspected) | built and covered |
| EVAL-091 Refused plan publishes nothing | same | `planning.rs` `a_refused_plan_carrying_play_commands_delivers_none_of_them`, `consecutive_accepted_ticks_apply_in_order_across_refused_plans` | built and covered |
| EVAL-092 Bounded retries | `orcvs/src/source/mod.rs:46` `OPTIMISTIC_TICK_ATTEMPTS = 3` | `planning.rs` `a_tick_conflicting_on_every_optimistic_attempt_commits_once_under_the_lock`, `ticks_under_continuous_editing_terminate_and_count_once` | built and covered |
| EVAL-093 No copy uncontended | `orcvs/src/source/buffer.rs` | `planning.rs` `a_snapshot_shares_the_cells_rather_than_copying_them`; `orcvs/tests/allocation.rs` `a_playback_tick_allocates_no_more_than_the_locked_tick` | built and covered |
| EVAL-100..105 Tick Portals record | `tick/execution.rs` `record_read`, `record`, `record_write`, `record_bundle`; `orcvs/src/source/portal.rs` spans | `tick/halt.rs` `a_lock_is_recorded_with_its_site_and_what_it_did_there`; `tick/replacement.rs` `a_refused_replacement_keeps_its_reason_in_the_record_and_paints_nothing`; `tick/movers.rs` `reads_and_writes_keep_their_order_and_owners_when_a_mover_rewrites_the_source`; `planning.rs` `a_refused_plan_publishes_no_tick_portals_and_the_locked_attempt_does` | built and covered (names; one inspected) |
| EVAL-106 Freshness | `orcvs/src/source/mod.rs` `freshness`, `tick_portals`; `model.rs` `committed_tick_portals` | `source/mod.rs` `a_source_answers_no_tick_portals_before_its_first_tick`, `a_committed_tick_that_reads_nothing_replaces_the_record_with_an_empty_one`, `a_revision_an_edit_made_after_the_commit_answers_the_record_as_retained`, `an_older_revision_keeps_the_record_it_was_read_with` | built and covered |

Open tracking relevant to the topic: `.scratch/placement-semantics/issues/04-investigate-emission-activation-operand-cycles.md` (ready-for-agent; research, now bounded by 0065 to the Expressions involved); `.scratch/source-read-write/issues/13-reorder-a-tick-incrementally-as-roots-join.md` (needs-triage; performance of reordering, no rule gap); `.scratch/source-diagnostics/issues/03-publish-and-present-tick-diagnostics.md` (ready-for-agent; Tick diagnostics presentation → PLAY/CONS).

## Conflicts and open questions

**C1 — Which roots are inert until Bang.** ADR 0006 ¶3: "An ordinary root Expression is inert until
Bang activation." ADR 0032 ¶9: "The first implementation retains the current Function-family
activation policy—value roots calculate each Tick and terminal roots require a Bang". ADR 0031 (superseded)
called 0006's broader gating "separate work". Code (`lang/src/atom.rs` `ActivationSource`): every
value Function is `Intrinsic`; effect Functions are `Bang` except movers and `**`. `docs/functions.md`
"Runs": "Most Functions take a Turn every Tick. Some are inert until a Bang reaches them." CONTEXT
Directional Bang: "Like every ordinary Function it is inert" (ambiguous). Every later ADR (0065
"activation can reach", 0072 "intrinsically active writer") assumes intrinsic value roots. No ADR
retires 0006's wider gating. Recommendation: state EVAL-050 as built (value roots and Self-Banging
and Bang Functions intrinsic; effect roots inert) citing 0032 ¶9, and record 0006's universal gating
as not adopted pending an ADR; consequence: the requirement would read the 0032 interim policy as
current, which needs an owner ruling or a short ADR to close.

**C2 — ADR 0014 contact activation not marked superseded.** 0014 ¶3: "Complete aligned root contact
also delivers Bang activation". 0072 supersedes this for 0006 only ("ADR 0006's contact activation");
0014's status lines name 0060, 0065, 0071 but not 0072. Code and CONTEXT Self-Banging ("activates
nothing that Tick; on the next Tick that `**` is a Bang Function") follow 0072
(`tick/bang_function.rs` `a_blocked_mover_activates_nothing_and_its_bang_plays_on_the_next_tick`).
Recommendation: treat 0014 ¶3's clause as superseded by 0072 (same rule restated); consequence: an
ADR status note on 0014 would make the chain followable.

**C3 — ADR 0006 direct Portal delivery of Bang.** 0006 ¶4: "A Portal may instead deliver Bang directly
to one root without overwriting its Function Cells." 0071 supersedes 0014's equivalent relay clause
but 0006's status lines do not name 0071. CONTEXT Bang: a pair-passing Function "writes `**` over its
complete destination ... A root whose anchor the `**` covers is overwritten rather than activated."
Test `tick/movers.rs` `a_copied_bang_overwrites_an_edge_roots_anchor_rather_than_activating_it`.
Recommendation: superseded by 0071; consequence as C2.

**Q1 — Ownership of dynamic-Portal scheduling.** ADR 0067's amendments (dynamic Input Portal writers
found at the Turn; dynamic writers first; write onto a completed read is next-Tick feedback) are
scheduling rules but 0067 is assigned to LANG. Recommendation: LANG states Track/Read/Write, EVAL
states the scheduling clauses (or cross-references them); needs coordination with topic-language.

**Q2 — Ownership of ADR 0037.** Assigned to EVAL in the run record, but it governs the Playback clock
("completes ADR 0002", `orcvs/src/playback.rs` `next_scheduled_at`, `is_overrun`). Dispositioned
here as redirected to PLAY (proposed). Main should confirm; if kept in EVAL it becomes EVAL-110..114.

**Q3 — Stale CONTEXT.md quotes in Rust comments.** `tick/halt.rs:91` and `portal.rs:15` quote wording
the entries no longer carry. Not a rule conflict (behaviour agrees); flag for the requirements
routing step, which must keep entry citations followable.

**Q4 — 0012's "a Note ... diagnoses" for Increment/Slew.** Superseded in effect by contextual decoding
(0024, 0034): Source `C4` in a Number operand or feedback Portal is Number `C4`; `G4` diagnoses
(`a_previous_that_is_not_a_number_diagnoses_and_writes_nothing`). At the Atom level a Note operand
still diagnoses (`lang/src/functions/tick.rs` `a_note_operand_diagnoses_in_every_tick_function`).
Recommendation: state the rule in decoding terms; no conflict once 0034's precedence is applied.

## Fresh-reader questions

1. **"I put `.+0102` below a `>>` mover's path. Why does the mover sometimes bang and sometimes pass?"**
   A mover tests only newly entered Cells in working Source at its Turn, without waiting for an
   occupant to leave (0060 rule 3b); Turn order (dependency, then Position) decides who finds the
   Cells empty (0006 collisions, 0020). A refused move writes `**` over its own Span and activates
   nothing until that `**` fires as a Bang Function next Tick (0072). Code: `tick/execution.rs`
   `deliver_source_effect`; tests in `tick/movers.rs`.
2. **"A producer below my `!>` Play writes its Note. Does it play this Tick or next?"** This Tick:
   execution is ordered by dependency, not row; a fixed upward Portal is an ordinary dependency
   (0032 ¶3, 0060 rule 3a). Exception: a dynamic Output Portal whose Cells a completed Turn already
   read lands as next-Tick feedback (0067 amendment, LANG). Test: `tick/live_editing.rs`
   `live_cross_boundary_chain_uses_lower_producers`.
3. **"My Grid has a cycle in one corner; why do other voices still play?"** 0065: a cycle stops only
   the Expressions on it and those depending on them; it is diagnosed once at the first computation
   in Parser order and waiting roots are diagnosed if activation reaches them; a cycle found at a Turn
   keeps completed Turns (0068). Tests: `tick/cycles.rs`, `tick/track.rs`
   `a_cycle_discovered_by_a_nested_track_stops_its_sibling`.
4. **"I typed while playing and my edit seemed to land a Tick late — is the Tick overwriting me?"**
   No: a Tick commits only against the revision it planned; a mid-plan edit refuses the plan whole
   and it is replanned at the same absolute Tick, up to three optimistic attempts then once under the
   lock (0057, CONTEXT Live Editing). Code `orcvs/src/source/planning.rs`, `OPTIMISTIC_TICK_ATTEMPTS`
   in `orcvs/src/source/mod.rs`. Crossing to PLAY: a Tick slowed past a whole period declines later
   Ticks via `is_overrun` (0037).
5. **"After Stop then Space, my `~.` Clock resumed mid-cycle. Bug?"** No: Clock reads the absolute
   Tick (0012) and a stop holds it; only Rewind returns to Tick 0 (0076, PLAY). Increment/Slew state
   is the Cells at their feedback Portal, so it survives anyway (0012 ¶4). Tests:
   `tick/tick_functions.rs` `the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at`.
6. **"Why does the console still light a Read's Cells after I edited another voice while paused?"**
   The committed Tick's Tick Portals record survives as `Retained` on later revisions; entries stay lit
   unless an edit touched the owner's subtree (or, for a selected site, a descendant's read Cells)
   (0077). Record rules EVAL-100..106; paint eligibility and precedence belong to CONS
   (`orcvs/src/render_frame.rs` `owner_stands`, `descendant_read_edited`).

## Final rule mapping (drafting phase)

Every retained row above carries a provisional ID; it lands in the final rule of the same ID in
`docs/requirements/evaluation.md`, except:

- Merged: EVAL-103 (order) into EVAL-102; EVAL-104 (private Span constructors) into EVAL-101.
- Narrowed per decisions.md D5: EVAL-050 states only that effect roots are inert except Self-Banging
  and Bang Functions; whether value roots are inert is listed under "Open decisions" with the built
  behaviour as observation (C1).
- Added per decisions.md D2: EVAL-027 (dynamic Input Portal writers found at the Turn) and EVAL-028
  (dynamic writers first; write onto a completed Turn is next-Tick feedback; own-Expression dynamic
  write diagnosed; missed transient effects). These take the ADR 0067 scheduling rows and the CONTEXT
  Portal / Output Portal / Input Portal row previously marked redirected (Q1 closed).
- Added: EVAL-057 (the Bang Function's Turn), because `language.md` LANG-240 points here for it
  (ADR 0072, 0073).
- Widened: EVAL-031 now also states that a Portal site outside the Grid resolves nothing (LANG-020
  points here); EVAL-082 also states the value-or-effect declaration (LANG-250 points here; the
  0028 ¶6 row moves from redirected to EVAL-082).
- ADR 0037 rows: redirected to PLAY per D2 (Q2 closed). ADR 0057 ¶5 `is_overrun` row: PLAY.
- `.scratch/bang-function/issues/08` ruling: cited in EVAL-025 per D3.
- C2, C3 resolved per D7: the snapshot follows ADRs 0072 and 0071.

## Follow-up (coordinator, drafting phase)

- Added EVAL-094 (a Render Frame reads a revision, never advances Playback, is not a Tick). Basis:
  CONTEXT Render Frame entry; no ADR states it in those words. Backed by ADR 0002 (the Playback
  Engine owns its clock and musical time), ADR 0001 (interpretation/Playback seam) and ADR 0057
  (revision reads take no part in planning). No Rust comment cites the Render Frame entry.
- Tick period / Overrun / clock mentions now point to PLAY-180 (Tick Grid) and PLAY-190 (Overrun):
  the intro and EVAL-060. EVAL-092's "however fast" and EVAL-061 "clock divider" are not the
  Playback clock.
- ADR 0028 Expression length: 0028 ¶7 still binds "the Source row bounds an Expression's extent;
  the parser has no separate fixed Atom limit". It is a parse rule, stated by `language.md` LANG-070
  ("No claim crosses a row edge ... there is no record limit"), so EVAL does not restate it. The
  Operand Stack bound, which 0028 ties to that point, is EVAL-081.
