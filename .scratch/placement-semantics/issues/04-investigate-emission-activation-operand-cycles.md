# 04 — Investigate emission activation cycles through operand reservations

**What to build:** A reproducible account and bounded language-design assessment of a Tick that
is rejected because an emission waits for Bang from a value producer while its destination is
reserved over that producer's operand. Make the current result and the decision needed to change
it explicit, so placement work neither silently fixes nor silently excludes the case.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Type:** research

Related to the [placement semantics spec](../spec.md). This investigation is independent of its
approved mover and emission slices and is not a prerequisite for either. It does not authorize
changing the accepted value-readiness or cycle-admission rules.

- [ ] Reproduce the minimal program through `Source::execute`: on an 8-by-3 test Grid, Equality
      with two literal 01 operands starts at column 0 of row 0; an eastbound mover starts at
      column 0 of row 1, immediately followed by a north-emitting Directional Bang Function at
      column 2; the remaining Cells are empty. Record the unchanged Source and the exact
      same-Tick dependency-cycle diagnostic. Distinguish confirmed branch evidence from any
      separately verified main baseline, citing the revisions used.
- [ ] Explain both edges: Equality's Bang can activate the emitter beside its output, while the
      emitter's northward destination reserves Equality's first literal operand. Potential writer
      readiness orders the emitter before Equality, closing the activation loop. Distinguish
      these operand/activation edges from occupancy-only Function-contact edges removed by
      placement work.
- [ ] Show why the separate eastbound mover and ordinary occupied-destination refusal do not
      erase that pre-execution dependency cycle. Inspect an adjacent control arrangement that
      breaks the feedback edge, and record its actual outcome through the same production seam.
- [ ] Determine whether the existing reservation contract intentionally requires rejection or
      whether an admissible narrower analysis can prove the emission unable to supply that
      operand. Assess the case where earlier writes can clear or change those Cells; an
      initially occupied operand alone must not be assumed permanently unwritable.
- [ ] Document the implications of retaining rejection versus pruning provably impossible
      suppliers, including determinism, activation, failed/no-write suppliers, and genuine cycle
      atomicity. Recommend a bounded next step. Any changed admission policy requires an
      explicit language decision and a separate implementation ticket, rather than incidental
      removal of literal-consumer edges.
- [ ] Link the findings and exact reproduction commands here. State that the placement property
      uses collision activation only and does not establish that arbitrary Bang-producing value
      graphs are free of cycles. Resolve this investigation when the evidence and recommendation
      are recorded, independently of whether a later semantics change is approved.

## Comments

### 2026-10-02 — Raised during ADR review

The concrete case was executed against the current branch and returned the dependency-cycle
diagnostic without committing effects. The same cycle was reported on main by the earlier design
investigation; that provenance needs a pinned reproduction before being treated as independently
verified. This ticket records the question without absorbing it into placement delivery.

### 2026-10-13 — Audit against origin/main cf01e3c

The reproduction is pinned on main by `an_emission_reserving_its_activators_operand_is_a_cycle`
(`orcvs/src/source/tick.rs:1424`, PR #213). It asserts the unchanged Grid and the same-Tick cycle
diagnostic; the first box still owes its write-up. Read the framing above against current ADRs:

- A cycle no longer rejects the Tick or withholds its committed effects. Under ADR 0065 and ADR
  0068 it stops only the Expressions it reaches, and Turns already completed keep their effects, so
  "rejected", "without committing effects" and "genuine cycle atomicity" mean the Expressions the
  cycle stops.
- Issue 03 already removed the occupancy-only contact edges that the second box asks to be told
  apart.
- Under ADR 0072 a collision activates nothing directly: a blocked mover writes `**`, which fires
  on the next Tick. "Collision activation" in the last box means activation from that `**`.
