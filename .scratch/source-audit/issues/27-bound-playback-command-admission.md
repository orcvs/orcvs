# 27 — Bound Playback command admission and define overload behavior

**What to build:** Playback has a bounded command backlog and a stated overload policy. Today `PlaybackEngine::spawn` in `orcvs/src/playback.rs` creates an unbounded channel. The 64-message fairness budget limits scheduling preference, not queued memory. `MidiSelectionHandle::request` in `orcvs/src/midi/selection.rs` sends already-open connections through the same channel, so queued installation requests retain device resources as well as memory. This is an embedding and sustained-load risk; the audit has not demonstrated overload from ordinary console interaction.

**Blocked by:** source-audit/02 (PR #144) should merge first; the policy below relies on its combined stop gate.

**Status:** resolved

Related: source-audit/02 repairs the stop-admission protocol; source-audit/21 bounds outgoing diagnostics rather than incoming commands. playback-actor/11 owns scheduling fairness, including first-Tick starvation. Design their ordering and overload guarantees together without duplicating their implementations.

- [x] Record a bound on queued commands and retained connection resources, including any side channels introduced by the solution.
- [x] Define admission, rejection or coalescing behavior for Start, Retune, Stop, Disconnect and output installation. Preserve meaningful ordering, especially destination changes before the first Tick; a bounded channel alone is not a complete policy.
- [x] Stop closes Tick admission synchronously and remains effective when the queue is full. It cannot silently lose its lifecycle command or leave an unanswerable outstanding-stop count. Preserve the rule that an already-admitted Tick may finish.
- [x] Rejected or superseded connection requests release their resources, and dropping the last engine handle still permits shutdown.
- [x] Deterministic tests stall the consumer, fill the backlog, exercise each overload policy, issue Stop, and observe resource release and shutdown. Tests use test-only synchronization rather than a seam in shipped code.
- [x] Record consequences for synchronous caller APIs: which calls can fail under overload, how callers observe that failure, and how the policy avoids blocking the browser main thread.

## Comments

**2026-09-25 — source audit against `origin/main` `199c3331`.** Filed separately from diagnostics retention because it bounds the other direction of traffic. The first-Tick fairness decision remains in playback-actor/11; bounded memory does not itself guarantee a deadline gets a turn.

**2026-09-25 — triage: admission policy decided.** Implement with playback-actor/11 in one PR (epic PR 4).

- Commands stop sharing one unbounded channel. Latest-wins slots replace queue entries where only the newest request matters, and whatever queue remains has a fixed, recorded capacity. The PR records the resulting bound on queued messages and on held connections, including every slot.
- Start and Retune coalesce: only the newest requested period is kept. A coalesced request is never refused for lack of space.
- Stop closes Tick admission synchronously through the combined gate from source-audit/02 before it enqueues anything. Its lifecycle effect lands in a slot that cannot be full, so a full queue can neither drop it nor leave an outstanding-stop count that is never answered. Coalescing several Stops answers every one of them. An already-admitted Tick may still finish.
- Disconnect and output installation share one destination slot, latest wins. A superseded or refused connection is dropped as soon as it is replaced, which releases its device. A destination change pending when a run begins is applied before that run's first Tick, preserving playback-actor/07.
- No caller waits on the Playback task, a Tick, or queue capacity, so the browser main thread is safe. The mailbox lock is held only for a constant-time update and never across an `await` or a Tick; on native targets a caller can briefly contend for it, and a caller whose destination change replaces a pending connection drops that connection itself, which may wait on the device closing its port. On the browser's single thread the lock is never contended. `start` and `retune` keep their existing `Result` and do not gain an overload failure, because coalescing is their overload behaviour. Any request that can still be refused reports it through its existing error or diagnostic path, and the PR names each such call.
- Dropping the last engine handle still shuts the engine down, and the tests cover it.

**2026-09-25 — implementation.** Implemented in orcvs/orcvs#148 (epic PR 4, stacked on #144): latest-wins mailbox replaces the unbounded command channel; policy in ADR 0056. Resolve on merge.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#148 (`67d28248`) directly onto `main` after #144 merged; every criterion verified against the PR.

**2026-09-25 — guarantee wording corrected.** "Caller APIs never block" overstated the policy: the mailbox is a mutex, and replacing a pending connection destroys it on the caller's thread. The design note now matches `orcvs/src/playback/mailbox.rs` and ADR 0056's caller consequences.
