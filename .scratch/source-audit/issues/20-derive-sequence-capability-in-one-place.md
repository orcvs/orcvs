# 20 — Make the Language Map's Sequence capability linear and declare the scalar width once

**What to build:** The Language Map's Sequence-capability derivation is linear in positioned entries and allocates nothing per Expression, and the scalar Output Portal width is declared once. Today `root_may_answer_a_sequence` collects a `Vec` per Expression, and `sequence_capable` scans every entry for children inside its loop, which is quadratic in the positioned entries of one Expression; a single nested Expression holds many entries. The width is declared three times: `SCALAR_WIDTH` (`source/tick.rs`), `OUTPUT_PORTAL_SCALAR_WIDTH` (`source/language_map.rs`) and `PAIR_WIDTH` (`source/portal.rs`). Sharing one derivation between the Language Map and Tick scheduling is syntax-highlighting/11; this ticket follows it and does the rest.

**Blocked by:** syntax-highlighting/11 — Share one Output Portal derivation with the scheduler.

**Status:** ready-for-agent

Related: 06 changes how rows are carried across revisions, but Sequence capability is computed on demand in `output_portal_reservations`, not stored in rows.

- [ ] The derivation syntax-highlighting/11 leaves visits each positioned entry a bounded number of times, so its cost is linear in positioned entries, not in Expressions.
- [ ] It performs no heap allocation per Expression or per entry; any scratch storage is allocated at most once per derivation call, or is inline, and the choice is recorded here.
- [ ] Evidence: an allocation test pins the per-call allocation count, and a benchmark over a Source with deeply nested Expressions shows the derivation's cost growing linearly with entry count. The comparison runs in CI, not locally.
- [ ] The scalar width is declared once.
- [ ] Language Map and Tick tests, including property tests, pass unchanged.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The "one derivation" half duplicated syntax-highlighting/11 (ready-for-agent), which now owns it; the comment at `language_map.rs` on the width constants already points at it. Width declarations corrected from two to three. 06 weakened from a blocker to a relation.

**2026-09-25 — performance contract specified.** "Linear in the number of Expressions" measured the wrong quantity: the quadratic scan is over the entries inside one Expression. "Allocation-free" is now stated as a per-Expression and per-entry bound, and the ticket requires allocation and benchmark evidence.
