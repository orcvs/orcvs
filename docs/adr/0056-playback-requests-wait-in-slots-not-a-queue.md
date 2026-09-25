# Playback requests wait in slots, not a queue

Status: accepted. Amends [ADR 0041](0041-the-playback-engine-owns-its-state-in-one-task.md): the engine's task still owns its state and still receives every transition from its handles, but those transitions no longer travel on an unbounded ordered queue. [ADR 0002](0002-playback-engine-owns-lifecycle-concurrency.md)'s synchronous `stop` guarantee is unchanged.

**What a handle can leave for the engine is bounded by shape, not by rate.** Every request a Playback handle or a MIDI selection handle makes is one where only the newest matters, or where several collapse into one. Each kind has a slot in the engine's mailbox, and a later request overwrites or merges with the one already there. The mailbox holds at most:

- one stop flag, standing for every `stop` made since the task last looked;
- one period to begin a run at and one period to retune a live run to;
- one destination change — a `disconnect` or one already-open MIDI connection.

Nothing queues behind the slots. The task is woken by a `Notify`, which stores at most one permit. The stop gate carries at most two outstanding requests: one for a pending flag and one for a flag the task has taken and not yet answered. Connections follow the same pattern. Beyond the adapter's installed connection, Playback holds at most two connections that have not been installed: one the task has taken and not yet installed, and a newer one pending in the slot. A connection a request replaces is held only by the caller that replaced it, until that call returns.

**The task takes the whole backlog at once and applies it in a fixed order: stop, tempo, destination.** The slots are shaped so that this order leaves the engine in the state applying the requests in the order they were made would, with two exceptions this decision accepts:

- A `stop` empties both tempo slots. Nothing asked before it survives it, and anything asked after it applies to the stopped engine it leaves behind. A `start` and a `stop` made before the task looks are therefore no run at all.
- `start` does nothing to a live run and `retune` does nothing to a stopped engine, so each writes the slot for the state it changes. The task applies whichever slot matches the state it finds once the stop is applied. A `start` replaces a pending `start`, so the newest period is the one a run begins at. A `retune` made while a `start` is pending also sets that start's period, because it retunes the run that start begins.
- Beginning, retuning and stopping a run do not choose where the run is delivered, so the destination change commutes with both of them.

The exceptions:

- A `start` that replaces a pending `start` sets the period the run begins at, where in order it would have found the run live and done nothing. Stopped, `start(1s)`, `retune(2s)`, `start(3s)` begins a run at 3s rather than 2s. The newest period is the one the caller asked for last.
- A `disconnect` that replaces a pending installation leaves the published destination on the device installed before it, where in order the replaced one would have been installed and published first. Nothing is delivered either way; only the selection shown differs.

The safety actions sent along the way can also differ:

- A `disconnect` then a `stop` while playing silences the device twice rather than once.
- An installation then a `stop` sends the stop's silence to the outgoing device rather than the new one.

Both are harmless. No Tick runs between the parts of one backlog, and every device that was delivered to is silenced.

**`stop` shuts the gate before it leaves its flag, under the mailbox lock.** It raises a request on the gate only when no flag is pending. A `stop` that finds a flag pending is covered by the request already standing, and the task answers every `stop` in that backlog when it takes the flag. A full mailbox cannot drop a stop or strand its request, because the stop's slot is a flag that is always free to set. A Tick already admitted still runs to completion.

**A destination change replaces the one pending and releases what it replaces.** Disconnecting and installing a connection answer the same question, so the newest is the answer. The request it replaces is dropped on the caller's thread as soon as the lock is released, which closes the port of a superseded connection. A connection offered after the task has gone is refused and dropped in the same way. When the task ends it drops any connection still pending, rather than leaving it until the last handle is dropped.

**A run's first Tick follows exactly the backlog that began it.** The backlog containing the `start` marks the boundary: it was taken whole and applied whole, so every request pending when the run began has been applied before the first Tick. That includes a `disconnect` or destination change, as [playback-actor/07](../../.scratch/playback-actor/issues/07-pin-the-first-ticks-order-against-a-queued-message.md) requires. The first Tick is due on arrival and executes without taking another backlog, so requests made after that backlog was taken wait for the Tick after it. A caller that keeps making requests cannot defer the first Tick. The one thing the task checks first is whether any handle is left: an engine nobody holds executes no Tick, and applies what was left before shutting down.

**Sleeping deadlines keep a budget of 64 backlogs.** Once a live run has applied `BACKLOGS_BEFORE_A_DEADLINE` backlogs since its last Tick, a reached deadline is taken ahead of a pending backlog. This guarantees progress and says nothing about wall-clock rate, because the callers set the pace at which backlogs arrive. Inverting the bias does not weaken `stop`: a deadline that overtakes a pending stop still meets the gate that `stop` shut. No other budget is claimed to improve the rate, because no measurement supports one.

## Caller consequences

No call waits on the task or on a Tick. The mailbox lock is held only for a constant-time update on either side, never across an `await` or a Tick. On native targets a caller can meet it briefly held; on the browser's single thread it is never contended. A caller whose destination change replaces a pending connection drops that connection on its own thread, and closing its port may wait on the device. No request is refused for lack of room.

- `PlaybackEngine::start` and `retune` keep their `Result`. They answer `EngineUnavailable` only when the task has gone, and `start` also reports that refusal as a `StartFailure` diagnostic.
- `PlaybackEngine::stop` and `disconnect` return nothing. Against a task that has gone there is nothing left to stop or disconnect.
- `MidiSelectionHandle::install` answers "running Orcvs is no longer available" when the task or every engine handle has gone, and drops the connection. A request later superseded by a newer selection was accepted, so it is not reported. The published destination shows which selection was installed.

Dropping the last engine handle still shuts the engine down. The task applies whatever was pending, sends the safety action and exits.

## Rejected alternatives

**A bounded channel.** It bounds memory but makes a full queue a new failure for every caller. `stop` would have to either wait, which is impossible on the browser's main thread, or be dropped, which strands its gate request. A bounded channel also still holds every superseded connection open until it reaches the front.

**A bounded queue with coalescing at its tail.** It collapses only adjacent duplicates. An alternating `start`/`stop` or `install`/`disconnect` pattern still grows it without limit.

**Letting the first Tick drain the queue.** This keeps every message queued before the first Tick ahead of it, including messages that arrive after the run began. A producer that never stops can therefore defer the first Tick indefinitely while the engine publishes `Playing`.
