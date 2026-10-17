# 03: Remove the Absence Marker

**What to build:** Every value Function answers a value. The Absence Marker
leaves the language, along with each guard that keeps it out of operands, the
parser and rendering, its `_` display, and the tests that state it as an
answer. CONTEXT.md drops its entry and rewords Equality, Delay, Euclidean and
Return around the Bang Output. A new ADR records the decision, as described in
`../spec.md`, Decisions.

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [ ] Every value the language holds has a Source encoding.
- [ ] Rendering an answer cannot yield "nothing".
- [ ] CONTEXT.md and the new ADR describe a Bang Output and no Absence
      Marker.
- [ ] Gates pass with `PROPTEST_CASES=32`, including rustdoc with
      `-D warnings`.
