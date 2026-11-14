# Trace: Playback and outputs (PLAY)

Pinned source: `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4`. Inventory only; rule IDs are provisional.
Owned ADRs: 0001, 0002, 0016, 0041, 0055, 0056, 0059, 0076, 0085, 0087 (synth clauses), 0088, 0090,
0099, 0103 (sounding clauses). Paths are repository-relative.

## Source disposition

Basis: **S** = source-stated (an ADR body or Status line says so), **P** = proposed (this inventory's
inference).

### ADR 0001 — Separate Source interpretation from the Playback Engine

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0001 ¶1 | Timing, ordering and Bang-lifetime clauses | superseded | ADR 0032 (EVAL), per 0001's own preamble | S |
| 0001 ¶2 | Interpretation produces the Tick Plan and commits its Source writes before handing the ordered Play Commands to the engine for the same Tick | retained | PLAY-010 Interpretation commits before Playback dispatches (commit mechanics: EVAL, 0057) | S |
| 0001 ¶2 | Engine owns timing, lifecycle and output adapters; never parses Source or infers musical intent; dispatches each Play Command as supplied; schedules Note Off only for a command carrying a Tick lifetime | retained | PLAY-010 | S |
| 0001 ¶2 | "all-notes-off on Playback stop or adapter disconnect remains the transport-safety exception" | superseded | ADR 0016 widens the action to CC 123 + CC 121 + centred bend ("widened here rather than left standing"); 0001 carries no Status line recording it | S |

### ADR 0002 — Playback Engine owns lifecycle concurrency

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0002 ¶1 | Engine owns clock, synchronisation, cancellation, stale-clock protection, shutdown; callers coordinate none | retained | PLAY-020 Engine owns its lifecycle; mechanisms refined by 0041 (cancellation and stale-clock guard deleted because state has one owner) | S |
| 0002 ¶1 | Cloneable handle; idempotent start and stop | retained | PLAY-020 | S |
| 0002 ¶1 | Stop synchronously prevents further Ticks before it returns | retained | PLAY-030 Stop admits no Tick after it returns | S |
| 0002 ¶1 | Atomically observe lifecycle state while draining ordered diagnostics | retained (amended) | PLAY-050 (observation published) and PLAY-140 (ordered drain, at most once); "every diagnostic since last drain" amended by 0055 | S |
| 0002 ¶2 | Each run has one fixed Tick period | retained (amended) | PLAY-040; 0041 restates as "one fixed Tick period at a time", retune per 0037 (EVAL) | S |
| 0002 ¶2 | First Tick executes immediately; output failures and Overruns become diagnostics without rolling back Source writes or stopping later Ticks | retained | PLAY-040 Run start and failure containment | S |

### ADR 0016 — Raw and Timed Play, MIDI terminal family, Monophonic Play, safety action

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0016 ¶1 | Spellings `!>` and `!~`; activation by a Bang on the root; fixed arity | redirected | LANG (Function families, activation); PLAY cites | P |
| 0016 ¶2 | Channel `00`–`0F`, velocity `00`–`7F`, note a Note, length `00`–`FF`; out-of-range or mistyped operands diagnose and emit nothing; no base-36 scaling or clamping; no implicit Number→Note | retained | PLAY-200 MIDI operand domains | S |
| 0016 ¶2 | "Missing … operands diagnose" as applied to a blank note | superseded | ADR 0108 (LANG): blank note binds to no note, sends nothing, no diagnostic | S |
| 0016 ¶3 | Timed: velocity `00` stops in Tick Plan order, schedules nothing; length `00` with nonzero velocity is a no-op leaving prior claim; length N starts in order, Note Off at the start of executed Tick T+N before that Tick's new commands; replacement stops prior instance and invalidates its expiry; Raw notes never owned | retained | PLAY-220 Timed Play lifetime and ownership | S |
| 0016 ¶1, ¶3 | Raw Play leaves Note Off to Source via velocity `00` | retained | PLAY-210 Raw Play velocity `00` is a Note Off; Raw notes unowned | S |
| 0016 ¶4–5 | `!c channel controller value`, `!b channel lsb msb`, domains, wire bytes direct | retained | PLAY-200 | S |
| 0016 ¶6 | Mono voice and expiry first given to the output adapter | historical | Corrected in the same ADR: engine owns it | S |
| 0016 ¶6 | One Mono voice per channel in the schedule that holds Timed claims; key channel vs channel+note; every Mono command sends Note Off for prior Mono note; vel/len `00` replaces voice with silence; T+length expiry; generation token per ownership change; Raw/Timed/Mono mutually unowned; last command in a Tick owns the channel | retained | PLAY-230 Monophonic voice; PLAY-250 Shared schedule and generation tokens | S |
| 0016 ¶6 | Stop, disconnect, destination change, teardown after refused delivery each send the safety action and clear Timed and Mono ownership | retained (conflict on teardown clause) | PLAY-260 Safety action triggers; see Conflict C1 | S |
| 0016 ¶6 | Safety action = CC 123, CC 121, bend `[0xE0\|ch,0x00,0x40]`, channel by channel, 16 channels; no explicit Note Offs; schedule discarded not played out | retained | PLAY-260; PLAY-270 Schedule discarded on lifecycle actions | S |
| 0016 ¶6 | OSC and UDP deferred | superseded | ADR 0085, and 0016's 2026-10-26 amendment | S |
| 0016 am. 2026-10-26 | UDP/OSC send nothing on stop; safety action is MIDI's alone | retained | PLAY-400 | S |
| 0016 am. 2026-11-14 | `!%` gains `legato`, Defaulted Number, blank binds `00` | redirected | LANG (0109 Defaulted Operand, arity) | S |
| 0016 am. 2026-11-14 | `00`/blank retriggers (Off then On); other = On then Off; legato at sounding pitch sends On and no Off, replacement claims to its own T+length; legato on the Tick prior stop is due takes that stop over; vel/len `00` starts nothing whatever legato and still stops prior | retained (confirmation pending) | PLAY-240 Monophonic legato ordering; see Open question O3 | S |

### ADR 0041 — The Playback Engine owns its state in one task

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0041 | Renumbered from 0040 | historical | Citation note only | S |
| 0041 ¶3 | State lives in one task reached by a cloneable message handle, not behind a lock; task owns its clock, waiting on messages and deadline together | retained | PLAY-050 One task owns engine state | S |
| 0041 ¶5 | Cancellation token, generation counter, `ClockRunGuard` deleted | historical | Rationale for 0041; no obligation beyond PLAY-050 | P |
| 0041 ¶6 | Lifecycle state and selected destination published through a watch read without awaiting; diagnostics drained without blocking | retained | PLAY-050 | S |
| 0041 ¶6 | Diagnostics through an unbounded channel | superseded | ADR 0055: bounded log shared by task and handles | S |
| 0041 ¶7 | Stop sets a one-way flag before sending; task checks it before each Tick; "asked to stop" distinct from "playing" | retained | PLAY-030 | S |
| 0041 ¶8 | Keep-the-lock alternative | historical | Rejected alternative | S |
| 0041 ¶10 | Seam of 0001 and Tick Grid rule of 0037 unchanged; first Tick immediate; failures don't roll back | retained | PLAY-040 (Tick Grid: EVAL) | S |
| 0041 ¶11 | Only MIDI construction publishes destinations; MIDI construction returns lifecycle handle plus `MidiSelectionHandle`; output-only construction has no destination channel | retained | PLAY-060 MIDI selection is a capability of MIDI construction | S |
| 0041 ¶12 | Install request travels on the same ordered queue as lifecycle requests | superseded | ADR 0056 (bounded destination slot) | S |
| 0041 ¶12 | Console opens the port; task is sole owner of adapter, connection, schedule; queue acceptance ≠ installation; refused delivery is an ordered diagnostic; installation clears schedule and sends safety action; discovery not published | retained | PLAY-060; PLAY-260 | S |
| 0041 ¶13 | Request type parameterised (`Infallible` for output-only); dispatcher fixed; no closures in messages; test request type lives below shipped constructors | retained | PLAY-060 | S |
| 0041 ¶14 | Selection handle holds a weak sender, cannot keep Playback alive or gain lifecycle control | retained | PLAY-060 | S |

### ADR 0055 — Playback diagnostics are retained within a bound

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0055 ¶1 | At most 32 retained, each message cut to 256 bytes on a char boundary with `…`, allocation made once | retained | PLAY-110 Bounded diagnostic retention | S |
| 0055 ¶2–3 | Eviction classes Overrun < start/retune < output < clock; evict oldest of lowest class present if no higher than new; else omit new; order kept | retained (class list incomplete) | PLAY-120 Class-ordered eviction; see Open question O1 (network failure class) | S |
| 0055 ¶4 | Every eviction/omission counted per class; drain ends with one `Omitted`; reset on drain; saturating u32 | retained | PLAY-130 Omitted summary | S |
| 0055 ¶5 | One lock-protected log, held for one append or one drain, never across a Tick or delivery; preserves cross-writer order | retained | PLAY-140 Ordered single log | S |
| 0055 Rejected | Three alternatives | historical | | S |
| 0055 Cons. | `Omitted` built only by a drain; console maps it to no message | retained / redirected | PLAY-130 (engine half); console mapping → CONS | S |
| 0055 Cons. | `MidiDeviceSelection::observe_diagnostics` status shows last failure | redirected | CONS (0092 routing, 0100 Toasts) | S |

### ADR 0056 — Playback requests wait in slots, not a queue

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0056 ¶1 | Mailbox holds one stop flag, one start period, one retune period, one destination change; `Notify` single permit; gate ≤2 outstanding; ≤2 uninstalled connections | retained | PLAY-070 Bounded request slots (Rewind flag added by 0076) | S |
| 0056 ¶2 | Backlog taken whole and applied stop → (Rewind, 0076) → tempo → destination | retained | PLAY-070 | S |
| 0056 ¶2–3 | Stop empties both tempo slots; start/retune write the slot for the state they change; newest start period wins; retune while start pending sets that start's period; destination commutes; listed exceptions and doubled-safety cases accepted | retained | PLAY-080 Request coalescing | S |
| 0056 ¶4 | Stop shuts the gate before leaving its flag, under the mailbox lock; admitted Tick completes | retained | PLAY-030 | S |
| 0056 ¶5 | Destination change replaces pending one, replaced connection dropped on caller's thread; late offers refused and dropped; task end drops pending connection | retained | PLAY-070 | S |
| 0056 ¶6 | First Tick follows exactly the backlog containing the start; requests after wait; engine with no handles runs no Tick | retained | PLAY-090 First Tick and progress | S |
| 0056 ¶7 | 64-backlog budget before a reached deadline is taken first | retained | PLAY-090 | S |
| 0056 Caller | No call waits; `start`/`retune` answer `EngineUnavailable` only when task gone, start also diagnoses; `stop`/`disconnect` return nothing; `install` error text; last handle drop applies pending, sends safety action, exits | retained | PLAY-100 Caller contract | S |
| 0056 Rejected | Three alternatives | historical | | S |

### ADR 0059 — Browser MIDI discovery from held access

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0059 ¶1 | Access requested once when backend built; discovery/connect synchronous; Pending → pending error (`MidiError::pending`); Granted → connected outputs in map order, nameless by id; Unavailable → empty list, logged once, final | retained | PLAY-430 Browser MIDI access states | S |
| 0059 ¶2 | Settlement stores answer and wakes Panel; each frame whose last discovery was pending rediscovers; other failures final until Scan; callback writes no menu state | retained | PLAY-440 Settlement and rediscovery | S |
| 0059 ¶3 | Entry point waits ≤250 ms for the answer before starting the console | retained | PLAY-450 Bounded access wait | S |
| 0059 ¶4 | `connect` returns `MidiConnectionRequest` (open or pending); failed open reported while installed destination plays on with no safety action | retained | PLAY-460 Owned connection request | S |
| 0059 ¶5–6 | Page-scoped claims count pending requests and connections; lifecycle module tracks unfinished opens, closes ownerless success, shares opens, waits for in-flight close | retained | PLAY-470 Page-scoped port ownership | S |
| 0059 ¶7 | `MidiConnection` names its output by id via thread-local `MIDIAccess`; no `unsafe impl Send`; disconnected output refuses next send | retained | PLAY-480 Connection by id | S |
| 0059 ¶8 | Sysex not requested | retained | PLAY-490 No sysex | S |
| 0059 ¶9 | Test seam below lifecycle policy; fake shares shipped module | retained | PLAY-470 (test-shape constraint) | S |
| 0059 Dest. | Explicit choice belongs to selection module; choosing installed A drops pending B, no reinstall, no safety; repeated choice keeps the same request | retained | PLAY-500 Explicit destination choice | S |
| 0059 Dest. | Output readout names installed destination; `Opening B…` status browser-only | redirected | CONS (Panel readout, Output list 0095) | P |
| 0059 Cons. | Non-native non-browser target builds the silent backend | retained | PLAY-430 | S |
| 0059 am. 2026-11-01 | Scan re-requests access while unanswered; latest request's refusal counts, stale refusal ignored, any grant wins; frames never re-ask | retained | PLAY-510 Scan re-requests access | S |
| 0059 am. 2026-11-01 | While pending, automatic selection picks first listed (Built-in Synth) until discovery settles; then first destination replaces it; explicit pick stays | retained | PLAY-520 Pending access plays through the synth | S |

### ADR 0076 — Stopping Playback holds its Tick

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0076 Status | Amends 0012 first-Tick clause | retained | PLAY-150; EVAL cites | S |
| 0076 ¶1 | Stop ends the run, sends safety action, discards schedule; absolute Tick survives; next run begins at Tick after last executed; no Paused state | retained | PLAY-150 Stop holds the absolute Tick | S |
| 0076 ¶3 | Resumed run's first Tick immediate, own grid; stopped interval neither skipped as Overruns nor owed | retained | PLAY-150 (grid rule: EVAL 0037) | S |
| 0076 ¶4 | Rewind of stopped engine: stays stopped, next Tick 0; of live run: stop + safety + discard, begin again at once at Tick 0 at same period; panic in safety → stays stopped; Rewind shuts no gate | retained | PLAY-160 Rewind | S |
| 0076 ¶5 | One Rewind flag, applied after stop; stop leaves it standing; tempo untouched | retained | PLAY-070 | S |
| 0076 ¶6 | Run Clock held while stopped, counts on, zero after Rewind | retained | PLAY-170 Run Clock | S |
| 0076 ¶6 | Panel shows held Tick at T, Run Clock at C | redirected | CONS | P |
| 0076 ¶7 | Command Shift R rewinds on native; web binds none | redirected / superseded in part | CONS; web clause amended by 0089 | S |
| 0076 Options | Four considered options | historical | | S |

### ADR 0085 — UDP and OSC send the Cells up to their closer

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0085 D1 | Payload = Cells after `!u`/`!o` spelling through next `!` on its row, else row end; found by parse search; never parsed; no escapes | retained | PLAY-300 Payload extent (parse mechanics shared with LANG Comment, 0074) | S |
| 0085 D2 | Payload Cells are ordinary; occupied Cells are spelling and closer; read at the Function's Turn after writers; extent fixed for the Tick; a written `!` closes only from next parse | retained | PLAY-310 Payload read at its Turn | S |
| 0085 D3 | One byte per Cell, space for empty, one datagram, no terminator; `!u!` sends nothing | retained | PLAY-320 UDP datagram | S |
| 0085 D4 | OSC: address `/…`, optional `,` type string of `i f s`, argument shape guessing (1–8 uppercase hex → int32, decimal with point → float32, else string); malformed → diagnose, send nothing; `C4` → int32 196 | retained | PLAY-330 OSC message encoding | S |
| 0085 D5 | One plain message per activation in Tick Plan order; no bundle/timetag; terminal-family invariants | retained | PLAY-340 One plain message per activation | S |
| 0085 D6 | Destination is a console setting (host, two ports), defaults `127.0.0.1`, 49161, 49162 | retained | PLAY-350 Network destination; settings UI → CONS | S |
| 0085 D7 | Sends via the engine to a network adapter; failure diagnoses, never stops; stop sends nothing on UDP/OSC | retained | PLAY-400 | S |
| 0085 D7 | Report every failed native send (implementation) | superseded | ADR 0090: native failures latched | S |
| 0085 D8 | Web parses/paints identically; activation diagnoses "cannot send" | retained | PLAY-360 Web build diagnoses each activation (paint → CONS) | S |
| 0085 D9 | Orcvs only sends; UDP input deferred with `!$` | retained | PLAY-370 Send only | S |
| 0085 Options | Nine considered options | historical | | S |
| 0085 Cons. | `!` dual role; stray Cells after closer diagnose as invalid Source; Paint draws Payload as text | redirected | LANG (parse), CONS (paint) | P |
| 0085 Cons. | Windows `ConnectionReset` ignored, other failures diagnose | retained | PLAY-410 | S |
| 0085 Cons. | v1 keeps UDP/OSC as accepted deferrals | historical | Release scope, not behaviour | P |

### ADR 0087 — Built-in Synth (synth clauses; site clauses → SITE)

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0087 D1 | `orcvs.synth` announcement, `orcvs.midi` posting from console origin to framing page only | redirected | SITE (page protocol, per run-record split) | S |
| 0087 D1 | Synth offered after every MIDI port, chosen like any destination, auto-picked only when no port exists | retained | PLAY-530 Built-in Synth as a destination (list drawing → CONS 0095) | S |
| 0087 D1 | Linked or small console always picks it | redirected | SITE (0086 small console) | P |
| 0087 D2 | Message carries Tick's planned instant (`timeOrigin + now`); synth plays a fixed delay after, within 50 ms target; past-due by more than delay sounds at once | retained | PLAY-540 Planned-time delivery | S |
| 0087 Cons. | Playback hands the planned Tick time to the destination with each message; others may ignore | retained | PLAY-540 | S |
| 0087 D3 | One Rust core, no audio I/O; web: own WASM in an AudioWorklet; native later through cpal | retained | PLAY-550 Synth core | S |
| 0087 D4 | Tones `00`–`07`: sine, triangle, saw, square, 2×FM, 2×AM; shared envelope; 150 ms release; chords | retained | PLAY-570 Tone channels | S |
| 0087 D4 | Kit on `08`–`0F` | superseded | 0099 (acid on `08`), 0099 am. 2026-11-13 (kit `09`–`0F`) | S |
| 0087 D4 | Kit by pitch class | superseded | 0087 am. 2026-11-13: General MIDI drum keys | S |
| 0087 D4 | Drum ignores Note Off, rings its decay; closed hat chokes open hat on its channel | retained | PLAY-580 Kit | S |
| 0087 D4 | Velocity sets loudness; no CC but 120/123 (release channel's tones); bend and others ignored | retained (amended) | PLAY-590; CC set widened by 0088 am. and 0099 | S |
| 0087 D4 | Sixteen voices; Note On past that takes oldest | retained | PLAY-590 Voices and messages | S |
| 0087 D5 | `orcvs.wake`, `orcvs.audio`, 500 ms start hold | redirected | SITE (amended by 0089: resume inside gesture) | S |
| 0087 D6 | Page without sound announces `available:false` + reason; choosing it answers reason as a connect failure; Playback runs silently | retained | PLAY-630 Unavailable synth | S |
| 0087 Rejected | Five alternatives | historical | | S |
| 0087 Cons. | Console has no audio code; no-page console offers no synth until native core | retained | PLAY-550 | S |
| 0087 Cons. | Stop's safety action reaches synth as All Notes Off | retained | PLAY-590 (with PLAY-620 for CC 121) | S |
| 0087 Cons. | Core WASM and worklet shipped in `console-web` | redirected | SITE (deploy) | P |
| 0087 am. 2026-11-13 | GM key map (35/36 kick … 62–64 conga), unlisted keys silent; closed/pedal hat chokes open hat | retained | PLAY-580; per-key table → `docs/functions.md` | S |

### ADR 0088 — Sound Module and synth effects

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0088 D1–3 | Sound Module drawn as Cells; channel lights 150 ms; effect columns | redirected | SITE (site page UI) | P |
| 0088 D3/D4 | Eight global effects on CC 1–8 | superseded | 0088 am. 2026-11-14 | S |
| 0088 D5 | Reader types values; refuses past `7F`; sends CC on page | redirected | SITE | P |
| 0088 D6 | Values reset on every load | retained / redirected | PLAY-600 (synth load values); module display → SITE | S |
| 0088 D7 | Theme colour `sound.magnitude`; console Theme gains key when console draws the module | redirected | SITE / CONS | S |
| 0088 D8 | Effects land one at a time | historical | All effect tickets (`orcvs-site/17`–`23`) resolved | P |
| 0088 Rejected | Including "Effects per channel" | historical | Last one withdrawn by the amendment | S |
| 0088 am. 2026-11-14 | Per-channel effects on CC 1, 74, 75, 85, 91, 92, 93, 94 with load values; CC 2–8 set nothing; MD/EQ pre-send, OF/LN per new sound; four shared send buses; on `08` CC 74/75 are acid cutoff/decay | retained | PLAY-600 Per-channel effects | S |
| 0088 am. 2026-11-14 | Module channel choice, outline, follow-the-stream, `CU`/`DC` columns on `08` | redirected | SITE | S |

### ADR 0090 — Each output destination keeps its policy in one module

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0090 D1 | One Outputs module owns MIDI adapter, schedule, connection state, latches, network output | retained | PLAY-380 Outputs module owns destination policy | S |
| 0090 D2 | Network Sends in their own ordered list; Play Command is MIDI only; a Tick's MIDI before its Network Sends | retained | PLAY-390 Network Sends beside Play Commands | S |
| 0090 D3 | Network keeps sending while MIDI disconnected; stop/rewind/disconnect/safety reach MIDI alone | retained | PLAY-400 Network independent of MIDI lifecycle | S |
| 0090 D4 | Native network failure latched like MIDI; cleared by successful send, new destination, new run; published on observation; web reports every activation | retained | PLAY-410 Failure latches | S |
| 0090 D5 | Seam takes a Network Send and destination; web refuses without encoding; pure encoder; one loopback test | retained | PLAY-420 Network seam | S |
| 0090 Options | Five considered options | historical | | S |

### ADR 0099 — Acid voice

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0099 D1 | Channel `09` | superseded | 0099 am. 2026-11-13: `08` | S |
| 0099 D1 | Mono voice; Note On while held slides ~60 ms, no envelope restart; stale Note Off ignored; polyBLEP saw/square; 24 dB resonant ladder capped below self-oscillation; filter envelope; accent ≥ `78`; CC 70/71/74/75/79 on that channel with load values | retained | PLAY-610 Acid voice | S |
| 0099 D1 | CC 121 returns the five to load values | superseded | 0099 am. 2026-11-14 (RP-015) | S |
| 0099 D2 | "Monophonic Play … never slides" | superseded in part | 0099 am. 2026-11-14 (legato); true only for legato `00`/blank | S |
| 0099 Rejected | Five alternatives | historical | | S |
| 0099 Cons. | Sound Module does not show acid controllers | superseded | 0088 am. 2026-11-14 shows `CU`/`DC` on `08` (SITE) | S |
| 0099 am. 2026-11-14 (a) | Acid CC 74/75 win over tilt/length on `08`; other effects apply | retained | PLAY-600 / PLAY-610 | S |
| 0099 am. 2026-11-14 (b) | CC 121 clears only Modulation; leaves acid controls and every other effect | retained | PLAY-620 Reset All Controllers follows RP-015 | S |

### ADR 0103 — Chord Play (sounding clauses; construction → LANG)

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| 0103 D1–2, D4–5, D7 | Chord operands, variants, build order, voicing domains, one diagnostic, width | redirected | LANG | S |
| 0103 D3 | One Turn emits one ordinary Raw/Timed Play Command per tone, lowest first; velocity `00`, length `00`, Note Off scheduling and safety apply tone by tone; engine gains no behaviour | retained | PLAY-290 Chord tones are ordinary Play Commands | S |
| 0103 Cons. | Answer bounded at twelve tones, no allocation | redirected | EVAL/LANG | P |

### CONTEXT.md entries (behaviour clauses)

Rust comments citing these entries: only `orcvs/src/playback.rs:3142` (test comment, Playback Engine
entry's refused-submission promise). `console/src/function_reference.rs:98,982` cite Terminal Output
Function (LANG/CONS).

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| CONTEXT Orcvs (running instance) | Choosing an output device is console configuration, not part of a running Orcvs | retained | PLAY-060 (selection is a capability beside lifecycle) | P |
| CONTEXT Playback | Requests a Tick Plan per Tick, dispatches; does not parse | retained | PLAY-010 | S |
| CONTEXT Playback Engine | ADR 0090 policy, schedule, generation tokens, clears, stop holds Tick | retained | PLAY-220…270, PLAY-380…400, PLAY-150 | S |
| CONTEXT Playback Engine | "The schedule records only what an output adapter accepted, so a refused submission leaves it standing and the stop is delivered again at the next executed Tick" | open | Conflict C1; ruling in `.scratch/midi-output-family/issues/02-schedule-timed-play-note-off.md` | P |
| CONTEXT Playback Engine | Refused channel during safety action does not end it; first refusal reported, remaining channels attempted | retained | PLAY-260 (no ADR states it; basis CONTEXT + code) | P |
| CONTEXT Rewind | ADR 0076 restated | retained | PLAY-160 | S |
| CONTEXT Tick Grid, Overrun | Grid, stall, Overrun consumes no Tick | redirected | EVAL (0037) | S |
| CONTEXT Omitted Diagnostics | Per-class counts, saturation | retained | PLAY-130 | S |
| CONTEXT Live Editing | | redirected | EVAL (0057) | S |
| CONTEXT Play Command | Explicit variant per spelling, validated values not wire bytes; per-role field types non-assignable; ordered list beside Network Sends; velocity `00` delivered as Note Off | retained | PLAY-280 Play and Output Commands (role-typed fields: no ADR; basis CONTEXT) | P |
| CONTEXT Network Send | Payload already read and typed; own list; nothing MIDI reaches it | retained | PLAY-390/400 | S |
| CONTEXT Output Command | One message, no lifetime; Note Off as `[0x80\|ch, note, 0x00]`; never a zero-velocity Note On; CC/PB unresolved | retained | PLAY-280 (wire encoding: CONTEXT only) | P |
| CONTEXT Terminal Output Function | Family list, activation gates evaluation, one output per Expression, twelve-tone bound | redirected | LANG/EVAL; PLAY-290 for per-tone delivery | S |
| CONTEXT Play / Timed / Mono / CC / PB entries | Domains, lifetimes, legato | retained | PLAY-200…250 | S |
| CONTEXT Play Function | Blank note optional (0108) | redirected | LANG | S |
| CONTEXT Raw / Timed Chord Play | Per-tone ownership; Timed Chord vs Mono separate ownership | retained | PLAY-290 | S |
| CONTEXT Application Command Function | Encoding deferred | redirected | LANG (not built; no owning ADR here) | P |
| CONTEXT UDP / OSC / Payload | ADR 0085 restated | retained | PLAY-300…340, PLAY-360 | S |
| CONTEXT Built-in Synth | Channel map, effects CCs, acid CCs, RP-015, listed after ports, Linked picks | retained / redirected | PLAY-530…630; Linked → SITE | S |
| CONTEXT Sound Module | | redirected | SITE | S |
| CONTEXT Run Clock | Held, counted on, zero after Rewind | retained | PLAY-170 | S |
| CONTEXT BPM, Readout | | redirected | CONS | P |

### Non-ADR sources

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| `docs/functions.md` § MIDI output, § Network output, § Built-in Synth | Per-Function operands, examples, GM key table, controller table | retained, redirected-to | Requirements point here for per-Function and per-key tables (run record) | S |
| `.scratch/midi-output-family/issues/02-schedule-timed-play-note-off.md` | Refused submission adopts no schedule changes; stop retried next Tick | open | Ruling not in an ADR; see C1 | P |
| `.scratch/musical-terminology/issues/13-add-legato-to-monophonic-play.md` § Decisions | Implementer's legato edge rules "for Toby to confirm"; no confirmation recorded | open | See O3 | S |

## Evidence and gaps

"Asserts read" = I read the test's assertions; otherwise matched by name only.

| Rule | Code | Tests | Status |
| --- | --- | --- | --- |
| PLAY-010 Interpretation commits before Playback dispatches | `orcvs/src/playback.rs` `PlaybackInner::execute_tick`; `orcvs/src/playback/outputs.rs` `Outputs::deliver` | `playback.rs` `a_tick_commits_source_before_submitting_play_commands`, `repeated_commands_are_dispatched_as_exact_tick_lists` (name) | built and covered |
| PLAY-020 Engine owns its lifecycle | `PlaybackEngine` (clone, `start`, `stop`) | `start_is_idempotent_and_draining_takes_the_diagnostics`, `stopping_and_disconnecting_each_send_the_safety_action` (name) | built and covered |
| PLAY-030 Stop admits no Tick after it returns | `orcvs/src/playback/gate.rs`; `PlaybackEngine::stop` | `gate.rs` 9 tests; `a_requested_stop_prevents_a_tick_before_the_message_arrives`, `stop_returns_without_waiting_for_a_tick_and_no_tick_follows_it`, `a_deadline_that_overtakes_a_pending_stop_meets_a_shut_gate` (name) | built and covered |
| PLAY-040 Run start and failure containment | `run_engine`, `begin_run` | `playback_begins_at_the_first_tick_and_advances_one_per_executed_tick`, `adapter_failure_does_not_roll_back_source_or_stop_playback` (name) | built and covered |
| PLAY-050 One task owns engine state; observation published | `run_engine`, `observation_watch`, `PlaybackObservation` | `beginning_a_run_publishes_tick_zero_playing_and_a_run_origin`, `clock_failure_remains_observable_after_output_panics` (name) | built and covered |
| PLAY-060 MIDI selection is a capability of MIDI construction | `PlaybackEngine::new` (`Infallible`), `with_midi_output_adapter`; `orcvs/src/midi/selection.rs` `MidiSelectionHandle` (`Weak`) | `orcvs/tests/midi_selection_handle.rs` 4 tests; `selection.rs` `install_honours_publication_unavailability` (name) | built and covered |
| PLAY-070 Bounded request slots and apply order | `orcvs/src/playback/mailbox.rs` `Backlog`, `Slots` | `mailbox.rs` 11 tests incl. `a_rewind_survives_a_stop_on_either_side_and_leaves_the_tempo`, `a_newer_destination_releases_the_one_it_replaces_at_once` (name) | built and covered. Network destination travels on a `watch` outside the mailbox (`PlaybackEngine::set_network_destination`), see O2 |
| PLAY-080 Request coalescing | `mailbox.rs` | `repeated_starts_keep_the_newest_period`, `a_retune_after_a_pending_start_sets_the_period_that_run_begins_at`, `a_stop_empties_the_tempo_asked_before_it_and_keeps_what_follows` (name) | built and covered |
| PLAY-090 First Tick and progress | `next_playback_event`, `BACKLOGS_BEFORE_A_DEADLINE = 64` | `a_run_s_first_tick_follows_exactly_the_backlog_pending_when_it_began`, `a_first_tick_due_on_arrival_does_not_wait_for_pending_requests`, `a_run_keeps_ticking_while_messages_keep_arriving`, `fairness_spent_while_stopped_does_not_invert_a_run_s_first_tick` (name) | built and covered |
| PLAY-100 Caller contract | `PlaybackEngine::start/retune/stop/disconnect`; `MidiSelectionHandle::install` | `an_engine_whose_task_died_refuses_a_later_start`, `selection_requests_fail_when_the_engine_task_has_ended`, `dropping_the_last_handle_applies_what_was_pending_and_shuts_down` (name) | built and covered |
| PLAY-110 Bounded retention | `orcvs/src/playback/diagnostics.rs` `MAX_RETAINED_DIAGNOSTICS = 32`, `MAX_DIAGNOSTIC_MESSAGE_BYTES = 256` | `a_long_message_is_cut_on_a_character_boundary_within_the_bound`; `playback.rs` `undrained_*_hold_bounded_memory` (name) | built and covered |
| PLAY-120 Class-ordered eviction | `diagnostics.rs` `Class` (adds `NetworkFailure` second-lowest) | `a_higher_class_evicts_the_oldest_of_the_lowest_class_present`, `a_flood_of_network_failures_never_displaces_another_failure`, `a_log_full_of_clock_failures_keeps_the_newest` (name) | code differs (extra class not in 0055; O1). No tracking issue found |
| PLAY-130 Omitted summary | `OmittedDiagnostics`, `OmissionCount` | `an_omission_count_saturates_and_says_so`, `overruns_behind_a_log_of_higher_classes_are_counted_but_none_is_named` (name) | built and covered |
| PLAY-140 Ordered single log | `DiagnosticLog` | `diagnostics_drain_in_order_and_exactly_once`, `a_task_recorded_diagnostic_drains_before_a_handle_recorded_one` (name) | built and covered |
| PLAY-150 Stop holds the absolute Tick | `PlaybackInner::begin_run`, `stop` | `a_run_begun_after_a_stop_resumes_at_the_next_tick` (asserts read), `a_clock_resumes_at_the_step_it_stopped_on` (name) | built and covered |
| PLAY-160 Rewind | `PlaybackInner::rewind`; `PlaybackEngine::rewind` | `a_rewind_makes_tick_zero_the_next_tick_of_a_stopped_engine`, `a_rewind_of_a_live_run_silences_it_and_begins_again_at_tick_zero`, `a_rewind_whose_silence_panics_leaves_the_engine_stopped` (name) | built and covered |
| PLAY-170 Run Clock | `PlaybackObservation::run_clock` | `stopping_freezes_the_published_tick_and_run_clock`, `resuming_holds_the_published_tick_and_counts_the_run_clock_on`, `a_rewind_zeroes_the_published_tick_and_run_clock` (name) | built and covered |
| PLAY-200 MIDI operand domains | `lang/src/functions/mod.rs` (`play`, `control_change`, `pitch_bend`), `lang/src/stack.rs` | `lang/src/functions/mod.rs` `play_rejects_channels_outside_the_midi_range`, `play_rejects_velocities_outside_the_midi_data_byte_range`, `timed_play_takes_the_same_midi_domains_as_raw_play_and_a_whole_byte_of_length`, `control_change_and_pitch_bend_take_a_midi_channel_and_two_data_bytes`, `play_rejects_implicit_number_note_conversions`; `orcvs/src/source/model.rs` `test_control_change_and_pitch_bend_operands_outside_their_domains_emit_nothing` (name) | built and covered |
| PLAY-210 Raw velocity `00` | `orcvs/src/playback/schedule.rs` `OwnedNotes::deliver` | `schedule.rs` `a_raw_play_at_velocity_00_is_delivered_as_a_note_off`, `raw_play_notes_never_enter_timed_ownership` (name) | built and covered |
| PLAY-220 Timed lifetime and ownership | `schedule.rs` | `a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names`, `a_timed_play_with_velocity_zero_stops_the_note_and_schedules_nothing`, `a_timed_play_with_no_length_emits_nothing_and_leaves_the_note_it_finds_standing`, `a_stop_due_this_tick_is_delivered_before_the_play_commands_that_tick_plans`, `a_repeated_timed_play_stops_the_instance_it_replaces_and_retires_its_expiry` (name) | built and covered |
| PLAY-230 Monophonic voice | `schedule.rs` | `a_monophonic_play_stops_whatever_note_its_channel_was_sounding`, `a_monophonic_play_with_velocity_zero_replaces_the_voice_with_silence`, `a_monophonic_play_with_no_length_replaces_the_voice_with_silence`, `two_monophonic_plays_for_one_channel_within_one_tick_leave_the_second_owning_it` (name) | built and covered |
| PLAY-240 Monophonic legato ordering | `schedule.rs`; `lang` `PlayCommand::Mono` `Articulation` | `a_legato_monophonic_play_starts_its_note_before_it_stops_the_one_it_replaces` (asserts read), `…_on_the_tick_the_held_note_ends_slides_into_it`, `…_at_the_pitch_sounding_restrikes_it_with_no_note_off`, `…_that_starts_nothing_still_stops_the_voice`; `playback.rs` `a_monophonic_play_line_delivers_each_legato_in_its_own_message_order`; `lang/src/functions/mod.rs` `monophonic_play_retriggers_at_legato_00_or_blank_and_slides_at_any_other` | built and covered |
| PLAY-250 Shared schedule, generation tokens | `schedule.rs` | `a_stale_expiry_cannot_stop_the_note_claimed_after_it`, `a_stale_mono_expiry_cannot_stop_the_voice_claimed_after_it`, `timed_and_mono_own_separately_and_neither_owns_a_raw_note` (name) | built and covered |
| PLAY-260 Safety action content and triggers | `orcvs/src/midi.rs` `safety_reset_messages`, `MidiOutputAdapter::safety_reset`; `outputs.rs` `silence`, `disconnect` | `midi.rs` `the_safety_action_clears_notes_controllers_and_bend_on_every_channel` (asserts read), `a_refused_safety_message_reports_the_first_error_and_attempts_the_rest`, `a_destination_change_sends_the_safety_action_to_the_destination_it_leaves`, `delivery_failure_attempts_the_safety_action_and_reselection_reconnects` (asserts read) | built and covered; teardown-clears-ownership clause differs (C1) |
| PLAY-270 Schedule discarded on lifecycle actions | `outputs.rs` `release`, `disconnect`, `begin_run` | `every_lifecycle_action_that_silences_output_clears_the_note_schedule`, `schedule.rs` `clearing_forgets_every_claim_and_every_scheduled_stop` (name) | built and covered |
| PLAY-280 Play and Output Commands | `orcvs/src/playback.rs` `OutputCommand`; `lang` `PlayCommand` | `midi.rs` `submits_note_off_as_its_wire_bytes`, `submits_control_change_and_pitch_bend_as_their_wire_bytes`; `schedule.rs` `control_change_and_pitch_bend_are_delivered_unresolved_and_in_tick_plan_order` (name) | built and covered |
| PLAY-290 Chord tones are ordinary Play Commands | `orcvs/src/source/tick/chord_play.rs`; `lang/src/functions/chord.rs` | `orcvs/src/playback/tests/chord_play.rs` 8 tests incl. `every_lifecycle_action_that_silences_output_clears_a_sounding_chord`, `a_chord_and_a_monophonic_play_on_one_channel_keep_separate_ownership` (name) | built and covered |
| PLAY-300 Payload extent | `lang/src/parser.rs` (payload claim), `lang/src/expression.rs` `add_payload` | `parser.rs` `a_payload_claims_through_the_next_bang_or_the_end_of_the_source`; `orcvs/src/source/tick/payload.rs` `the_cells_after_an_early_closer_are_source_again` (name) | built and covered |
| PLAY-310 Payload read at its Turn | `orcvs/src/source/portal.rs` `read_payload` | `payload.rs` `a_write_into_a_payload_lands_in_the_same_tick`, `a_closer_a_write_puts_in_a_payload_is_sent_and_closes_it_from_the_next_parse`, `a_write_over_the_closer_moves_it_from_the_next_parse` (name) | built and covered |
| PLAY-320 UDP datagram | `lang/src/functions/network.rs`; `orcvs/src/network.rs` | `network.rs` (lang) `udp_sends_each_cell_as_one_byte_and_an_empty_payload_rests`; `orcvs/src/network.rs` `a_udp_send_arrives_as_its_exact_bytes`; `payload.rs` `an_empty_payload_sends_nothing_and_reports_nothing` (name) | built and covered |
| PLAY-330 OSC message encoding | `lang/src/functions/network.rs`; `orcvs/src/network.rs` encoder | `osc_guesses_each_argument_from_its_shape`, `a_type_string_decides_each_argument`, `a_malformed_osc_payload_diagnoses_and_names_the_word`; `encodes_the_osc_specifications_first_example`; `orcvs/tests/network_loopback.rs` 3 tests (name) | built and covered |
| PLAY-340 One plain message per activation | `TickPlan::network_sends` | `outputs.rs` `sends_keep_their_order_and_go_to_the_latest_destination`; `payload.rs` `an_unbanged_payload_function_sends_nothing`, `a_payload_function_is_invalid_where_a_value_is_required` (name) | built and covered |
| PLAY-350 Network destination | `orcvs/src/network.rs` `NetworkDestination`; `console/src/network.rs` | `the_default_destination_is_orcas`; `console/src/network.rs` `a_stored_destination_is_read_back`, `an_invalid_value_is_reported_and_changes_nothing` (name) | built and covered |
| PLAY-360 Web diagnoses each activation | `orcvs/src/network.rs:154` web adapter | `outputs.rs` `the_web_build_reports_every_refused_send`; `console/tests/wasm.rs` `web_playback_diagnoses_each_udp_send_it_cannot_make_and_plays_on` (name) | built and covered |
| PLAY-370 Send only | (absence of any receive path) | none | built, no verified test (negative property) |
| PLAY-380 Outputs module owns destination policy | `orcvs/src/playback/outputs.rs` `Outputs` | `outputs.rs` 9 tests (name) | built and covered |
| PLAY-390 Network Sends beside Play Commands | `Outputs::deliver` | `a_ticks_midi_is_delivered_before_its_network_sends` (name) | built and covered |
| PLAY-400 Network independent of MIDI lifecycle | `Outputs::silence`, `disconnect` | `network_sends_go_on_while_midi_is_disconnected`, `stopping_and_disconnecting_send_the_network_nothing` (name) | built and covered |
| PLAY-410 Failure latches | `Outputs::record_midi_failure`, `record_network_failure`, `begin_run` | `a_native_network_failure_is_reported_once_until_it_changes`, `a_send_that_leaves_clears_the_network_failure`, `a_new_destination_and_a_new_run_each_report_the_failure_again`, `the_observation_says_whether_the_last_network_sends_failed`, `a_run_that_begins_after_a_failed_run_reports_the_failure_again`; `orcvs/src/network.rs` `only_connection_reset_goes_unreported` (name) | built and covered |
| PLAY-420 Network seam | `orcvs/src/network.rs` `NetworkAdapter` | `a_refused_send_names_where_it_went`; loopback tests (name) | built and covered |
| PLAY-430 Browser MIDI access states | `console/src/web_midi.rs`; `console/src/console_midi.rs` silent backend | `a_pending_request_answers_that_access_is_awaited`, `a_browser_without_midi_access_offers_an_empty_list_rather_than_an_error`, `granted_access_lists_the_connected_outputs`; `console_midi.rs` `a_target_without_a_midi_service_offers_an_empty_list_rather_than_an_error`; `console/tests/wasm.rs` `web_midi_answers_discovery_and_connect_without_waiting` (name) | built and covered |
| PLAY-440 Settlement and rediscovery | `console/src/web_midi.rs`, `console/src/midi.rs` | `the_frame_after_the_browser_answers_catches_up_without_a_scan`, `a_failed_discovery_is_not_repeated_by_later_frames` (name) | built and covered |
| PLAY-450 Bounded access wait | `console/src/main.rs` `MIDI_ACCESS_WAIT` (250 ms) | `console/tests/wasm.rs` `the_midi_access_wait_ends_whether_or_not_the_browser_answers`; `a_console_built_after_the_answer_never_shows_access_pending` (name) | built and covered |
| PLAY-460 Owned connection request | `MidiConnectionRequest`; `console/src/web_midi/ports.rs` | `an_output_that_fails_to_open_leaves_the_playing_destination`, `the_frame_after_an_output_opens_installs_it` (name) | built and covered |
| PLAY-470 Page-scoped port ownership | `console/src/web_midi/ports.rs` | `an_abandoned_open_releases_its_output`, `a_new_claim_waits_for_an_in_flight_close_before_reopening`, `dropping_one_of_two_pending_owners_keeps_the_other_request`, `replacing_a_source_does_not_close_the_new_backends_connection` (name) | built and covered |
| PLAY-480 Connection by id | `console/src/web_midi.rs:200` `thread_local!` | `a_connect_to_a_gone_or_disconnected_output_is_refused` (name) | built and covered |
| PLAY-490 No sysex | `web_midi.rs` `request_midi_access()` with no options | none | built, no verified test |
| PLAY-500 Explicit destination choice | `console/src/midi.rs`, `web_midi.rs` | `choosing_the_installed_output_cancels_without_safety_messages`, `choosing_the_pending_output_keeps_its_completed_request`, `choosing_an_unpublished_ready_destination_again_does_not_reconnect_it` (name) | built and covered |
| PLAY-510 Scan re-requests access | `console/src/web_midi/requests.rs`; `MidiBackend::request_access_again` | `requests.rs` 5 tests; `a_scan_asks_the_browser_again_only_while_access_is_unanswered` (name) | built and covered |
| PLAY-520 Pending access plays through the synth | `console/src/built_in_synth.rs` | `pending_midi_access_plays_through_the_synth_until_a_port_is_granted`, `refused_midi_access_keeps_the_synth_picked_while_pending`, `a_synth_the_performer_picked_while_pending_stays_picked` (name) | built and covered |
| PLAY-530 Built-in Synth as a destination | `console/src/built_in_synth.rs` | `the_synth_is_listed_after_the_ports_only_while_a_page_announces_it`, `an_unlinked_console_picks_the_synth_only_without_a_port`, `playback_posts_its_notes_to_the_page` (name) | built and covered |
| PLAY-540 Planned-time delivery | `orcvs/src/playback/outputs.rs` (`since_due`); `console/src/built_in_synth.rs:277`; `synth/web/synth-host.js` `DELAY_MS = 50` | `midi.rs` `a_submission_hands_each_message_how_late_its_tick_was`; `built_in_synth.rs` `each_message_is_posted_with_how_late_its_tick_was`; `synth/src/lib.rs` `a_message_for_a_frame_already_rendered_sounds_at_once`; `site/tests/synth.spec.ts` "a framed console plays the page's synth at a constant offset from its Ticks" (asserts read) | built and covered |
| PLAY-550 Synth core | `synth/src/lib.rs` `Synth`; `synth/src/web.rs`; `synth/web/synth-worklet.js` | `synth/src/lib.rs` `a_message_sounds_from_the_frame_it_is_scheduled_at` (name) | web built and covered; native not built — `.scratch/orcvs-site/issues/14-give-native-the-built-in-synth.md` (needs-triage) |
| PLAY-570 Tone channels | `synth/src/tone.rs`, `lib.rs` | `channel_zero_plays_a_sine_at_the_notes_pitch`, `each_tone_channel_sounds_and_each_differs_from_the_others`, `a_tone_releases_over_150_ms_after_its_note_off`, `a_channel_plays_a_chord` (name) | built and covered |
| PLAY-580 Kit | `synth/src/drum.rs`, `lib.rs` `KIT_KEYS` | `the_keys_of_a_family_sound_alike`, `a_key_outside_the_kit_is_silent`, `a_closed_or_pedal_hat_chokes_an_open_hat_on_its_channel_only`, `every_drum_sounds_ignores_note_off_and_falls_silent` (name) | built and covered |
| PLAY-590 Voices and messages | `synth/src/lib.rs` `VOICES = 16`, CC 120/123 at `lib.rs:427` | `a_seventeenth_note_takes_the_oldest_voice`, `velocity_sets_loudness`, `all_notes_off_releases_tones_and_leaves_drums_ringing` (asserts read; loops CC 120 and 123), `other_messages_change_nothing` (name) | built and covered |
| PLAY-600 Per-channel effects | `synth/src/controller.rs` (CC constants), effect modules | `each_effect_changes_the_channel_it_is_set_on`, `an_effect_set_on_one_channel_leaves_another_alone`, `a_send_returns_only_what_its_own_channel_plays`, `*_at_its_load_value_changes_nothing`, `controllers_74_and_75_on_the_acid_channel_set_its_cutoff_and_decay` (name) | built and covered |
| PLAY-610 Acid voice | `synth/src/acid.rs` (`ACCENT = 0x78`) | `the_acid_channel_holds_one_note_at_its_pitch`, `a_note_struck_while_the_last_is_held_slides_to_its_pitch`, `a_late_note_off_for_the_note_slid_from_changes_nothing`, `velocity_from_78_accents_the_note_louder_and_brighter`, `control_change_70_from_40_plays_a_square` (name) | built and covered |
| PLAY-620 Reset All Controllers follows RP-015 | `synth/src/controller.rs` `RESET` | `reset_all_controllers_leaves_the_acid_voices_sound_alone`, `reset_all_controllers_leaves_every_effect_but_modulation_alone`, `reset_all_controllers_clears_modulation_on_its_channel_only` (name) | built and covered |
| PLAY-630 Unavailable synth | `console/src/built_in_synth.rs` | `an_unavailable_synth_answers_its_reason_on_connect` (name) | built and covered |

Other open tracking in this area: `.scratch/midi-port-ownership/issues/08-a-tick-still-lands-while-a-destination-change-is-processed.md`
(needs-triage: the outgoing connection's safety action runs synchronously inside the Tick-owning
task, so a slow send can delay a Tick; no ADR states a timing guarantee for installation);
`.scratch/playback-actor/issues/08-…`, `10-…`, `16-…` (needs-triage; type and test-harness work,
no rule at stake).

## Conflicts and open questions

**C1. Does teardown after a refused delivery clear note ownership?**
- ADR 0016 ¶6: "Playback stop, adapter disconnect, destination change, and teardown after a refused
  delivery each send the safety action and clear Timed and Mono ownership."
- `CONTEXT.md` Playback Engine: "The schedule records only what an output adapter accepted, so a
  refused submission leaves it standing and the stop is delivered again at the next executed Tick";
  its list of what clears the schedule omits teardown, though teardown still triggers the safety
  action. Ruling source: `.scratch/midi-output-family/issues/02-schedule-timed-play-note-off.md`
  (resolved), not an ADR.
- Code: `orcvs/src/playback/outputs.rs` `Outputs::deliver` adopts the resolved schedule only on
  `Ok`; `MidiOutputAdapter` itself sends the safety action and drops its connection on a failed send
  (`midi.rs` `delivery_failure_attempts_the_safety_action_and_reselection_reconnects`), and the next
  install clears the schedule. `playback.rs`
  `a_refused_submission_leaves_the_schedule_standing_for_the_next_tick` pins CONTEXT's version with
  an adapter that survives refusal. For the MIDI adapter the audible result matches 0016 (no
  delivery succeeds until a destination change, which clears); for a surviving adapter it differs.
- Recommendation: state PLAY-260 as 0016 has it for the MIDI destination (teardown sends the
  safety action; ownership ends with the connection) and record CONTEXT's retry rule as the
  engine's behaviour for an adapter that survives a refusal, flagged as resting on a `.scratch`
  ruling with no ADR. Consequence: the snapshot cannot make the retry rule binding; an ADR (or an
  amendment to 0016) is needed if it should be.

**O1. Where does a network failure rank in diagnostic eviction?** ADR 0055 names four classes
(Overrun, start/retune, output, clock). ADR 0085/0090 add `PlaybackDiagnostic::NetworkFailure`
but assign it no class. Code (`orcvs/src/playback/diagnostics.rs` `Class`) ranks it just above
Overrun, below start/retune, and counts it in `OmittedDiagnostics::network_failures`; tested by
`a_flood_of_network_failures_never_displaces_another_failure`. No ADR or `.scratch` ruling found.
Recommendation: record the code's rank as the snapshot's statement with basis "implementation, no
ADR" and flag for an ADR 0055 amendment. Consequence: until then the rank is not binding.

**O2. Is the network destination one of the mailbox's slots?** ADR 0056 says "every request a
Playback handle … makes" has a slot and lists stop, two tempo slots and one MIDI destination change.
`PlaybackEngine::set_network_destination` writes a `watch` read at each send instead
(`orcvs/src/playback.rs:1090`). It is bounded by shape (one value) and newest-wins, so it keeps
0056's intent; 0056 predates 0085. Recommendation: state it as a separate newest-wins channel
outside the mailbox, citing 0085/0090; not a conflict that changes behaviour.

**O3. Are the legato edge rules confirmed?** ADR 0016's 2026-11-14 amendment says the rules "at
the pitch sounding, on a silent channel and at the Tick a note ends are the implementer's, listed
under the ticket's Decisions for Toby to confirm". The ticket
(`.scratch/musical-terminology/issues/13-add-legato-to-monophonic-play.md`) is resolved but records
no confirmation. The ADR is accepted and its text states the rules. Recommendation: retain PLAY-240
as binding (accepted ADR text) and note the pending confirmation in the snapshot. Consequence: a
later amendment may change them; code and tests already follow the amendment.

**O4. ADR 0001 has no Status line for the safety-action widening.** 0001 still says
"all-notes-off"; ADR 0016 states it widens that action. Settled by 0016's text; noted only so the
snapshot cites 0016, not 0001, for the action's content.

## Fresh-reader questions

1. **"After I press Stop, my hardware synth's sustain pedal and pitch wheel reset. Why, and what
   exactly is sent?"** Stop sends the safety action on all sixteen channels, channel by channel:
   CC 123 All Notes Off, CC 121 Reset All Controllers, then a centred Pitch Bend `[0xE0|ch, 0x00,
   0x40]`; no per-note Note Offs, and the Note Off schedule is discarded. UDP/OSC receive nothing.
   Sources: ADR 0016 ¶6 and 2026-10-26 amendment, ADR 0076 ¶2, ADR 0090 D3;
   `orcvs/src/midi.rs` `safety_reset_messages`.
2. **"If I pause with Space and play again, does a Clock `~.` start over?"** No. A stop holds the
   absolute Tick; the next run begins at the Tick after the last executed, with its own Tick Grid
   and its first Tick immediate. Only a Rewind (Command Shift R on native, the Panel control on
   every target) makes Tick `0` next. Sources: ADR 0076, ADR 0089 (CONS) for the web control,
   ADR 0037 (EVAL) for the grid; `orcvs/src/playback.rs` `a_clock_resumes_at_the_step_it_stopped_on`.
3. **(crosses LANG) "My `!%` bassline has blank notes on some steps and a blank last pair. What
   sounds?"** A blank note binds to no note: that step sends nothing and is not diagnosed, so the
   channel's Mono voice keeps sounding to its own Note Off (ADR 0108, LANG). A blank `legato` binds
   `00` and retriggers (ADR 0109, LANG; ADR 0016 2026-11-14 amendment). Any other `legato` sends the
   new Note On before the old Note Off, so the acid voice on `08` slides (ADR 0099 amendment).
   Sources: `docs/functions.md` § MIDI output; `orcvs/src/playback.rs`
   `a_monophonic_play_whose_note_goes_blank_leaves_its_voice_sounding`.
4. **"No MIDI device is selected. Do my `!o` sends still go out, and in what order relative to
   `!>` on the same Tick?"** Yes: the MIDI connection gates MIDI alone. A Tick's MIDI is delivered
   before its Network Sends, whatever their Source order; each list keeps Tick Plan order. A failed
   native send is reported once until it changes. In the web build every activation diagnoses.
   Sources: ADR 0090 D2–D4, ADR 0085 D7–D8; `orcvs/src/playback/outputs.rs`.
5. **(ordinary) "I want to add a third output destination. Where does it go?"** As one more entry in
   the Playback Engine's Outputs module, which owns every destination and its policy; Tick Plans
   carry per-destination ordered lists (Play Commands for MIDI, Network Sends for the network), and
   the destination sits behind a seam taking its own command type. It does not receive the MIDI
   safety action unless its policy says so. Sources: ADR 0090; `orcvs/src/playback/outputs.rs`
   `Outputs`; `orcvs/src/network.rs` `NetworkAdapter`.
6. **(crosses CONS) "After a long stall the console didn't show my device error. Was it lost?"**
   The engine retains at most 32 diagnostics between drains, evicting Overruns first and clock
   failures last; anything evicted or omitted is counted in a final `Omitted` summary. The console
   drains every frame, so an ordinary run never reaches the bound; how failures are shown (Toasts,
   one routing point) is the console's. Sources: ADR 0055; ADR 0092, ADR 0100 (CONS);
   `orcvs/src/playback/diagnostics.rs`.

## Final rule mapping (drafting phase)

Document: `docs/requirements/playback.md`. Every provisional rule ID above is kept as the final ID
except as listed here; each retained row maps to the rule its row names.

- **Added:** PLAY-180 (Tick Grid) and PLAY-190 (Overrun), from ADR 0037, which EVAL redirected to
  PLAY per decisions.md D2. The trace rows "CONTEXT Tick Grid, Overrun → EVAL" now map to them.
- **Merged:** PLAY-530 and PLAY-630 into **PLAY-520** (Built-in Synth as a destination, pending-access
  pick, unavailable synth). PLAY-490 into **PLAY-480** (connection by id; no sysex).
- **Moved:** the 0087/0099 channel-map clauses (trace row PLAY-560 ref.) state in **PLAY-570** (tone
  channels and channel map).
- **Re-dispositioned per decisions.md:** C1 → gap under PLAY-260 plus an open decision (D9);
  O1 → gap under PLAY-120 (D9); O2 → open decision; O3 → binding with a note in PLAY-240 (D10);
  O4 → PLAY-260 cites 0016 (D7).
- **No-ADR sources stated with their `.scratch` origin (D3):** PLAY-260 remaining-channels clause
  (`.scratch/midi-output-family/issues/06-…`); PLAY-280 Note Off encoding and role types
  (`CONTEXT.md`, `.scratch/midi-output-family/issues/07-…`).
- **Redirected rows expected elsewhere:** LANG — Play spellings/activation, blank note (0108),
  `legato` Defaulted Operand (0109), Chord building (0103), Payload `!` dual role, twelve-tone
  bound; EVAL — commit mechanics (0057), Live Editing, one-output rule (0028); CONS — 0055 status
  mapping, Toasts/routing (0092, 0100), Run Clock/Tick Readout display, Command Shift R and Panel
  Rewind (0076, 0089), `Opening B…` readout (0059), Output list drawing (0095), Payload paint,
  network settings UI, BPM; SITE — page messages `orcvs.synth/midi/wake/audio`, 500 ms start hold
  (0087, 0089), Linked/small console pick, Sound Module (0088), `sound.magnitude`, `console-web`
  shipping.

### Audit fix (D2)

- PLAY-520 keeps listing order and auto-pick only; the unavailable synth (former PLAY-630 content)
  and the unframed console now link to SITE-046 and SITE-048.
- PLAY-540 keeps only "each message carries its Tick's planned time"; the 50 ms delay and
  late-message handling link to SITE-042/SITE-043.
- Intro no longer claims site.md states 0088's Sound Module; it says SITE owns it.

- PLAY-230 reworded: only a Monophonic Play that emits a Play Command replaces the claim; a blank note leaves the voice sounding (ADR 0108).
