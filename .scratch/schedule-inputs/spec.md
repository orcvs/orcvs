# Give schedule identity and construction one borrowed projection

Status: ready-for-agent

## Problem

Schedule reuse relies on a comparison of Parser facts maintained separately from
the reads that construct computations and classify locks. A new scheduling read
can leave equality unchanged and admit a stale schedule. The agreement is a
caller obligation rather than something the schedule module owns.

## Confirmed design

The schedule module owns one borrowed projection of Parser-derived scheduling
facts. Equality and schedule construction consume that projection, including
lock occupancy. Construction no longer receives unrestricted Language Map
access. Merely relocating the comparison does not meet this requirement.

Expression facts include scheduled eligibility, Span, leading Function, each
positioned entry's Cell range and parent, and whether the entry is unparsed,
a parsed non-Function, or a particular Function. Language Unit facts retain
anchor, Span and kind. Retain every distinction current equality makes,
including Language Unit kind, even where construction currently needs less.

The projection borrows the existing row derivation. It stores no second copy of
Parser structure and adds no projection allocation or memoization. Literal
values, Source characters, Tokens and Parser diagnostic text are not schedule
inputs. Layout and cycle diagnostics remain derived during construction.

## Constraints

- Preserve Tick Plans, diagnostics, activation and Turn ordering.
- Preserve exactly which edits invalidate a schedule; broadening reuse is
  separate work.
- Preserve reuse across literal-only edits that keep scheduling facts unchanged.
- Preserve shared unchanged rows, comparison of dirty rows only, and lazy
  Portal resolution, dependency derivation and ordering.
- Keep execution's access to live Source and Parser structure; the narrowed
  interface concerns schedule construction.
- Preserve ADRs 0032, 0034 and 0068. No language decision is reopened.

## Delivery and verification

Two independently green slices: first Expression facts, then Language Unit facts
and lock occupancy. The second depends on the first. Each ticket includes its
own verification; there is no separate testing ticket.

Existing fresh-versus-reused comparisons remain the main evidence: Tick Plans,
diagnostics, Turn order, stale-schedule counterexamples and edit properties.
Add focused behavioural regressions only for gaps exposed by the change.
Representation-mirroring tests do not establish the contract.

Run the repository gates for the edited crate and its dependants with
PROPTEST_CASES=32, and the required workspace gates before opening an
implementation PR. No performance improvement is claimed. Add a reproducible
benchmark if implementation changes a cost the PR claims to move; comparisons
are deferred to CI under the repository contract.

## Out of scope

- Sharing Output Portal Reservation derivation between Paint and scheduling.
- Changing cache invalidation, language semantics or Function declarations.
- Storing an eager per-row scheduling projection or adding a projection OnceLock.

## Decision record

Confirmed with the user on 2026-10-14. An owned per-row projection was considered
and rejected in favour of a borrowed projection to avoid duplicate structural
storage and parsing-time allocation. The user approved the two-ticket breakdown.
