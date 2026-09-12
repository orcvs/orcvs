# Say which Turn a computation took

**Status:** ready-for-agent

## Goal

A Tick computes two facts about itself — which computations may take a Turn, and in what order — uses them, and keeps neither. Everything downstream infers them from Source bytes. Record the Turn a computation took, so a test about ordering can assert ordering.

## What is actually missing

`ComputationState` already carries `activated`. The two facts it does not carry are **which Turn** a computation took and **why** it was activated, and they are not the same size of change.

`Schedule.order` is a `Vec<usize>` that leaves `order_turns`, but `Schedule` is private and `execution::execute` consumes the field by value in its loop, so nothing survives the call. `active_roots` answers a `Vec<bool>` that is a local inside `order_turns` and is never returned at all.

## Why this is not a test-only seam

The repository contract forbids a shipped function reaching a branch that only a test populates. This is the adjacent shape and it is already settled here: `test-only-seams/10` — the effort whose purpose was removing test-only seams — added `ComputationState::interpreted` and `::interpretations` in exactly this form. The field is written unconditionally by production code, on a value `Source::execute` already returns; only the accessor is unused in the library build. `interpreted`'s doc comment states the reasoning, including why the suppression is `allow` rather than `expect`: the method is dead in the library build and live in the test build, so an expectation would go unfulfilled in the second and fail the gate that compiles both.

Follow that shape exactly. It also states the future this has beyond tests — "what a console or a diagnostic view will ask for".

## Why a Turn ordinal is `Option`

`execution::execute` can die partway through its own loop, and `reject` keeps the states deliberately. Its comment is this effort's argument, written down by whoever wrote it:

> An ordering defect discards all writes, including Bang cleanup, and independent Play Commands, but keeps ordered diagnostics. The states survive it: what ran is still what ran, and a rejected Tick is the one case an empty plan cannot be told apart from a quiet one.

`None` is a Turn the Tick never reached. A sentinel, or recording the ordinal at schedule time, would conflate "ordered third and ran" with "ordered third and the Tick died first" — the exact case a rejection test wants to assert on.

## Decisions

- Two tickets. The ordinal is `01` and lands without touching `active_roots`. The activation reason is `02` and changes a fixpoint; it does not hold `01` hostage.
- `turn: Option<usize>` on `ComputationState`, written in `execute`'s loop, read through a `pub(in crate::source)` accessor carrying `#[allow(dead_code, reason = "an output the shipped callers discard")]`.
- The Turn order itself is unchanged. This effort records what already happens and changes nothing about what happens.
- No bulk rewrite of the ~80 byte-offset assertions. `01` adds the capability and demonstrates it on a handful of cases; converting the rest is opportunistic work for whoever next touches each test.
- No new ADR. Nothing about the language, the schedule or the Tick Plan changes; this publishes a fact the Tick already computes.

## One question `02` must settle first

When two producers both activate one root, does the record keep the first or all of them? Today `active[index] = true` cannot hold a second, so "first writer wins" is an accident of the `bool` rather than a decision anyone made. Naming the fact forces the choice, and it is a language question rather than an implementation one. `02` is filed but must not be built until it is answered.

## Required behavior

No Tick plans differently, orders differently, or diagnoses differently. The existing suite is the evidence: a ticket that needs a behavioural test rewritten to pass has changed something it should not have.

## Out of scope

- Publishing `ComputationState` beyond `crate::source`.
- The console or diagnostic view that would eventually read these facts.
- Splitting `tick.rs`, which is a separate candidate this effort makes worth more.
