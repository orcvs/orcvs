# Playback diagnostics are retained within a bound

Status: accepted. Amends [ADR 0002](0002-playback-engine-owns-lifecycle-concurrency.md): diagnostics are still drained in order and each at most once, but a drain no longer promises every diagnostic recorded since the last one. Amends [ADR 0041](0041-the-playback-engine-owns-its-state-in-one-task.md): the ordered diagnostics are a bounded log shared by the task and its handles rather than an unbounded channel.

**What a Playback Engine retains between two drains is bounded, in entries and in bytes.** At most 32 diagnostics are retained (`MAX_RETAINED_DIAGNOSTICS`), each message cut to at most 256 bytes on a character boundary and marked with `…` (`MAX_DIAGNOSTIC_MESSAGE_BYTES`), in an entry allocation made once when the engine is built. An engine nobody drains therefore holds at most 32 entries plus 32 bounded messages, whatever its run does: an overloaded run declining every Tick, a caller repeating a start the engine refuses, or a device alternating between two errors so the identical-failure latch never suppresses one. Each of those produces one diagnostic per event, and an unbounded queue grew by one entry per event for as long as nobody drained it.

**A full log evicts by class, least serious first.** The classes, in the order they are evicted:

1. Overrun. It carries no user-facing message and only says a Tick was late.
2. Start and retune failures, as one tier. Each is also returned to the caller that asked, as an `Err`, so the diagnostic is the second copy.
3. Output failure. It is the only word a user gets that their device refused a Tick.
4. Clock failure. It says the run ended by itself, and `Orcvs` withdraws its request to play only on finding one.

A diagnostic recorded into a full log evicts the oldest retained entry of the lowest class present, provided that class is no higher than its own. If everything retained is of a higher class, the new diagnostic is the one omitted. A flood of one class therefore never displaces a more serious diagnostic, and the newest diagnostic of the most serious class recorded since the last drain is always retained. Retained entries keep the order they were recorded in.

**Nothing is omitted silently.** Every eviction and every omission is counted against its class in `OmittedDiagnostics`. A drain that omitted anything ends with one `PlaybackDiagnostic::Omitted` carrying the counts, and a drain resets them. Each count is a `u32` that saturates rather than wrapping; a saturated count says so, and is then a lower bound. The Overruns since the last drain are the Overruns the drain answers plus the omitted Overrun count, so a drain after a long undrained stall still says how many Ticks were declined and names the latest.

**The log is one lock-protected value, held for one append or one drain.** The task and every handle append to it, and a drain takes everything it holds. The lock is never held across a Tick or a delivery, so a drain waits on at most one append and a Tick on at most one drain. A single shared value is what keeps the two writers' diagnostics in one order, which ADR 0002 asks for; a bounded channel with the same capacity would have refused the newest diagnostic, whatever its class, once full.

## Rejected alternatives

**Bound only the Overruns.** Coalescing Overruns into a counter bounds the loudest class but leaves repeated refused starts and alternating device errors unbounded, and an adapter's error text has no length of its own.

**A bounded channel that drops when full.** It drops the newest diagnostic, which may be the clock failure that ends the run, to keep older Overruns.

**Drop the oldest entry regardless of class.** A stall long enough to fill the log would evict the device failure that preceded it, and the user would never learn their device refused a Tick.

## Consequences

`PlaybackDiagnostic` gains the `Omitted` variant. Only a drain builds it: the engine records its diagnostics as a type with no `Omitted` variant, so the log never retains or merges a summary. The console maps it, like an Overrun, to no user-facing message.

A full log can omit a newer start or retune failure behind retained output or clock failures, because those are the more serious classes. The console's `MidiDeviceSelection::observe_diagnostics` shows the last failure a drain answers, so in that case its status names an older output failure rather than the newer refused start. The caller that asked to start still received that refusal as an `Err`. This happens only when 32 or more diagnostics accumulate between two drains, which a console draining every frame does not reach in an ordinary run.

The console drains every frame, so an ordinary run never reaches the bound, and a drain within the bound answers exactly what it did before.
