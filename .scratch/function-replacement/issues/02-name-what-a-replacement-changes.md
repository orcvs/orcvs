# 02 — Name what a replacement changes

**What to build:** The five remaining terms of the replacement guard each answer the same diagnostic string, so a refusal says "activation requirements, output kind, or result width" whichever fact actually differed. Replace the chain with a comparison that answers which one, and let the diagnostic name it.

**In `lang`:** a `ReplacementChange` enum with variants `AnswerKind`, `Activation`, `BangEmission` and `Write`, and a `Function::replacing(self, target) -> Option<ReplacementChange>` that walks a `const` table of `(ReplacementChange, fn(Function, Function) -> bool)` in that order and answers the first difference. The table is the point: an `if` chain leaves a duplicated comparison re-creatable as a dead arm Rust does not warn on, while a table lets a test assert each variant appears exactly once. `domains()` at `lang/src/atom.rs:842-846` is the nearest existing shape — fn pointers in a `const` slice, though per-variant inside a match arm rather than as a flat table. A slice of tuples is new here; adopt it rather than copying a worse shape for want of precedent, and do not generalise it into a library for its one caller.

**In `orcvs`:** a `Lookup::replacement_change(index, replacement)` that asks `lang` first and, finding no difference, compares `would_reserve(index, replacement)` against `reserved(index)` and answers `ReplacementChange::Width`. Today's term order already puts width last, so appending it preserves the order exactly and every reachable pair reports the fact it reports today. The `Width` variant lives on the `lang` enum even though only `orcvs` can produce it, so that one type names all five facts and `ReplacementChange::ALL` is a complete list.

**The diagnostic** names the change. Four assertions match the current string, all in `orcvs/src/source/tick.rs` — at `:2452`, `:2475`, `:2504` and `:2573` — and nothing outside that file does. Those four become assertions on the specific fact. Prose paraphrases in ADRs 0003, 0032, 0034 and 0036 and in several `.scratch` files are not assertions and do not change.

**The rationale** currently interleaved through the guard becomes doc comments on the variants: why the Portal offset is its own fact, why ADR 0036's width defect is stated about direction as well as extent, why the whole `SourceEffect` is compared rather than its fields.

**Carry forward unchanged:** the guard reads `self.states[contact.index].function`, the Function the computation is running, not the one the Parser found, and the comment recording that no test pins that choice stays as it is. This ticket does not move it.

**Two tests over the new shape.** First, every variant of `ReplacementChange::ALL` appears exactly once across the table plus the appended width term — this is the test that would have failed on `01`'s duplicate. Second, for each variant, that it is the **first** difference for some pair drawn from `Function::ALL`, with `Width` stated as expected-none and that expectation carrying its reason from the spec's reachability table. Record the weaker sole-difference set beside it: only `BangEmission` and `Write` are the *sole* difference for any pair, and "Activation fires but never alone" is exactly the fact that made an existing test's comment wrong.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] `Function::replacing` answers the first of four declared differences from a `const` table walked in order.
- [ ] `Lookup::replacement_change` composes that with the width comparison, appended last.
- [ ] The guard reads one answer and emits a diagnostic naming the fact that differed.
- [ ] The four existing message assertions name a specific fact instead of the shared string.
- [ ] Each variant's rationale is a doc comment on the variant.
- [ ] A test proves each variant appears exactly once across the table and the appended term.
- [ ] A test proves each variant is the first difference for some pair, and that `Width` is the first difference for none, under a comment stating why.
- [ ] No replacement admitted before this ticket is refused after it, and none refused is admitted.

## Verification

Both crates are edited, so: `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for `lang`, `orcvs` and `console`.

No benchmark is owed. The guard is unreachable in production — see the spec — so this change cannot move the cost of any path a Source reaches, and the ticket claims no performance effect.
