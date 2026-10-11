# Handoff: implement the Bang Function on PR #213

## Where things stand

- Branch `feat/bang-glyph`, PR #213 on `orcvs/orcvs` (https://github.com/orcvs/orcvs/pull/213). Push to `origin`, never `fork`. The branch is level with `main` plus five commits; head `1dce710a` (or later) carries the settled design and no implementation of it.
- The PR's earlier commits implement ADR 0071 with a Bang display provenance record. This work replaces that record with the Bang Function.
- The design is settled. Do not reopen it: implement it. Read, in order:
  1. `CLAUDE.md` (repository contract, gates, completion evidence).
  2. `docs/adr/0072-a-standalone-bang-is-a-function.md` (Proposed; the language decision).
  3. `.scratch/bang-function/issues/07-settle-clearing-and-halt-precedence.md` — decisions D1–D17, the audit summary, and worked Grids E1–E9. Where anything disagrees, ticket 07 wins.
  4. `.scratch/bang-function/spec.md` and `map.md`.
  5. `.scratch/placement-semantics/spec.md`, its issue 03, and `docs/adr/0060-value-inputs-wait-placement-tests-occupancy.md`.
- `lang/bang-function-prototype.html` is a historical walkthrough and not authoritative.

## The design constraint

The language owner's constraint: the Bang Function is a new Function that follows the rules every Function follows, and every other outcome emerges from existing rules applied to it. No Bang-specific ordering, clearing or placement rule. Only two things are new: a Function whose Turn activates its aligned roots and vacates its own Span, and the parser naming a root `**` as that Function.

If a test shows an E-example or a D17 consequence was derived wrongly (E1–E9 are hand-traced, not executed), do not add an exception to make it pass. Work out which existing rule produces the actual outcome, and if that changes a language outcome, record it under `## Comments` in ticket 07 and stop to ask the owner before continuing.

## Order of work

### 0. Close placement-semantics/02

`.scratch/placement-semantics/issues/02-let-movers-enter-vacated-cells.md` shipped in PR #195 (ADR 0060, commit `0c1e7bb2`), and its own comment says implemented, but its status line still says `claimed`. Confirm its acceptance criteria against `main`, then set `Status: resolved`.

### 1. placement-semantics/03 — Turn-local occupancy for emissions

Implement `.scratch/placement-semantics/issues/03-apply-turn-local-occupancy-to-emissions.md` as written, on this branch, as its own commit(s), before any Bang Function code. It removes the emitter → occupant edge and the mutual emitter/mover exception (`orcvs/src/source/tick.rs`, `producer_edges`, the `emits_without_vacating` arm). Without it, `*v` over a `**` becomes a same-Tick cycle once `**` is a Function. Mark it resolved when done. Placement issue 04 is an independent investigation; leave it.

### 2. Bang ticket 01 — the Bang Function (test-first)

`.scratch/bang-function/issues/01-a-standalone-bang-is-a-function.md`. Use the `rust-change` and `mattpocock-skills:tdd` skills. Write E1–E9 and the ticket's criteria as failing Source Tick tests first, at the production seam (`Source` edits and Ticks; assert Play Commands, diagnostics and committed Grids). No test asserts on scheduler internals.

Code map (line numbers approximate, from `c2f0eb7e`):

- **Function table:** `lang/src/atom.rs` `define_functions!` (~672). Add the `**` row: no operands, `Intrinsic`, can emit Bang. `from_spelling` is generated with `#[deny(unreachable_patterns)]` (~504), so keep `**` out of it with a per-row fact or a row the parser names directly; a hand-written arm will not compile. Amend the `ActivationSource` doc (~383-401: Self-Banging Functions are no longer "the one exception") and `takes_no_operand`'s list (~598-606).
- **Parser:** `lang/src/parser.rs` root-only arm `Some("**") => (Token::Bang, Atom::Bang)` (~234) is reached only at an Expression start; name the Bang Function there. `**` in an untyped operand must stay the Bang value (test ~983); in a typed operand it stays a type error. Sweep tests over every Function: ~638-646, ~1745-1779.
- **Declaration and Turn:** no existing `Interpretation` (`lang/src/interpreter.rs` ~10-29) or `SourceBundle` both activates and clears. Add one whose write site is only its own Span, modelled on the Advance's own-anchor site (`orcvs/src/source/portal.rs` ~460-462). Its Turn activates `bang_roots` over its Span (`tick.rs` ~421-446) and writes blanks with `WriteKind::Vacate` (`execution/working.rs` ~27-39), as the Advance's vacate does (`execution.rs` ~806-814). Its content is today's start-of-Tick fire (`execution.rs` ~254-266), which is deleted.
- **Self-edge exemption:** `producer_edges` (`tick.rs` ~919-936) exempts `advances()`. Add a predicate for "declares it clears its own Span"; do not widen `advances()`, which also drives contacted-root activation (~679-691), the contact skip (~972-979) and `reserves_over` (~953-955).
- **Delete provenance:** `Source.bang_display`, `shared_bang_display`, `forget_bang_display` and the commit update (`orcvs/src/source/model.rs` ~132-153, 250, 303, 343, 425-540); `TickPlan.bang_display`; Bang bookkeeping in `resolve` (`tick.rs` ~1040-1110); the typed-Bang loop in `Execution::new` (`execution.rs` ~200-270); `PlanningSnapshot.bang_display` (`planning.rs` ~35-57); the extra argument on `plan`, `plan_carrying`, `plan_first`, `execute`. Test helpers must plan exactly as production does.
- **Delete contact activation:** `contacted_roots` (`tick.rs` ~382-387) and its use in `activate` (~688-691); the contact arm in execution (`execution.rs` ~845-864). A blocked mover still writes `**` over its Span.
- **Missed Bang:** drop the Selected-only condition (`execution.rs` ~655-664, `misses_activation` ~893-903).
- **Add no ordering edge.** Overwrite, reader, activation, lock, `lock_covers` and writers-first edges apply unchanged; restate `lock_covers`'s doc comment (`tick.rs` ~762-765).
- **Console:** `console/src/function_reference.rs` — `every_function_in_the_table_has_a_worked_example` (~972-1000) and the settled-area test (~760-800); the asset's `*v`, `*<`, `*>` first-step-block examples now alternate (E9). Check Language Map paint (`render_frame.rs` ~54-56, `language_map.rs` ~577-610, 856-862): a root `**` must not highlight an Output Portal. Use the `egui` skill if presentation changes.

Regression tests from the two provenance bugs (rebuild them; the earlier agent worktree is not available): an offset `***` beside a written `**` under an active covering writer plays nothing and the typed `*` survives; a `**` written into an operand fires exactly once when freed, with zero and with several quiet Ticks before the freeing edit.

Tests to flip at their sites rather than delete: `a_source_read_back_fires_the_bang_display_it_was_saved_with`, `a_bang_typed_over_bang_display_fires_once`, `a_block_edit_over_bang_display_fires_the_bang_it_leaves` and the `assert_only_bang_display` helper (`model.rs`); `a_planned_tick_fires_a_typed_bang_once_and_never_the_display_a_tick_wrote` (`orcvs/src/source/mod.rs` ~380); the mover contact tests (`tick.rs` ~2572, ~2589); the Directional Bang emission-block tests (`tick.rs` ~1703-1723); the Halt-over-`**` diagnostic (`tick.rs` ~2870-2895); `a_result_written_where_stale_bang_display_stood_joins_nothing` (`tick.rs` ~4808, signature only).

### 3. Bang tickets 02, 03, 04

`issues/02-producers-that-take-no-turn-leave-their-bang-to-fire.md`, `03-a-mover-meets-a-bang-in-source-order.md`, `04-a-halt-above-a-typed-bang-misses-it.md`. These mostly pin outcomes ticket 01's code already produces; write each test, and treat a failure as a design question per the constraint above. Ticket 05 is dropped.

### 4. Bang ticket 06 and documentation

`issues/06-close-the-remaining-pr-213-review-findings.md`. Then:

- Set ADR 0072 to accepted and point the superseded clauses' status lines at it: ADR 0071 (third decision and its consequences), ADR 0032 (Bang is not a standalone Function), ADR 0006 (contact activation; Halt over an occupied non-root as it applies to `**`), ADR 0067 (missed Bang restricted to dynamic writes).
- CONTEXT.md: split **Bang** (the pulse value) from **Bang Function** (standalone `**`: root-only, intrinsically active, no operands; its Turn activates aligned roots and clears its Span); add **Missed Bang**; retire Bang display to an _Avoid_ entry; amend Source Snapshot, Producer, Self-Banging Function, Halt Function and Copy as the implemented behaviour requires. The glossary carries no implementation detail.
- Mark tickets resolved as each lands; mark `lang/bang-function-prototype.html` non-authoritative or remove it.
- Retitle PR #213 and rewrite its body for the Bang Function, listing the D17 behaviour changes.

## Verification

Per `CLAUDE.md`, for every change, with `PROPTEST_CASES=32` exported:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
cargo nextest run --package <crate> --locked
```

`lang` means `lang` and `orcvs`; `orcvs` means `orcvs` and `console`. Before updating the PR, once:

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
cargo nextest run --workspace --tests --no-default-features --locked
```

For `.scratch/` changes: `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`. ADR files: `bash scripts/check-tooling-contract.sh`. `lang`'s `Interpretation` and `SourceBundle` are `pub`, so a new variant is a public-API change: run the rustdoc gates and note it. Deferred to CI: `mise run check`, `check_merge`, `test_wasm`, `bench`, full proptest. Run `rust-review` on the complete diff before pushing the final state, and report in the `CLAUDE.md` completion-evidence format.

## Commits

One commit per ticket or coherent step, in the repository's style: an imperative subject, then a body stating the behaviour, the tests flipped or added, and the docs changed. The rationale for removing provenance belongs in the commit message and ADR 0072, not in code comments (`docs/agents/comments.md`). Push each commit to `origin feat/bang-glyph`.
