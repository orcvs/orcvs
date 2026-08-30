# Implement explicit Tick and feedback Functions

**Status:** ready-for-agent

## Goal

Implement ADRs 0012 and 0013 by making absolute Tick, Function Position, and visible Portal feedback
explicit interpretation inputs. Deterministic Source Snapshot plus Tick must produce a deterministic
Tick Plan.

## Delivery order

1. `issues/01-thread-tick-and-position-into-interpretation.md`
2. `issues/02-add-clock-delay-and-euclidean.md`
3. `issues/03-add-visible-increment-and-interpolation.md`
4. `issues/04-add-deterministic-random.md`

## Required behavior

- First Playback Tick is absolute Tick `0`; later Ticks increment without hidden language state.
- Clock, Delay, and Euclidean use the formulas and invalid-domain diagnostics from ADR 0012.
- Increment and Interpolation read exactly one previous visible Number through their Portal.
- Random is deterministic from seed, Tick, Position, and Sequence index using ADR 0013's seed layout.

## Out of scope

- Tempo units other than Playback Engine's existing Tick period.
- Hidden per-Function history or activation-count randomness.
- Sparse/infinite Position implementation.
