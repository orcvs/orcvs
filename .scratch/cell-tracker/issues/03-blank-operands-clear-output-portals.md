# 03 — Blank operands clear Output Portals and propagate blank

Status: ready-for-agent
Blocked by: 02

**What to build:**

Implement ADR 0062 through root and nested execution. An empty inline operand
makes a value Function answer a Blank Answer without a diagnostic. It writes
two spaces at its Output Portal and returns those blank Cells when nested.

## Acceptance criteria

- [ ] A value Function with a completely blank operand answers blank, writes two spaces through its Output Portal and raises no diagnostic. A nested blank Return makes its parent do the same.
- [ ] An activated Terminal Output Function with a blank operand emits no Play Command and raises no diagnostic.
- [ ] Nested Addition with a blank operand writes spaces at both Output Portals, returns blank to the parent and raises no diagnostic.
- [ ] The nested test asserts both south destinations are cleared to spaces and the nested Function spelling remains intact. A blank operand to Increment or Interpolation clears the shared feedback/output Cell; the next valid evaluation reads the cleared Cell as initial `00`.
- [ ] A value root with a blank operand whose Output Portal feeds Timed Play's note slot clears that slot, and the Bang-activated Timed Play emits no Play Command rather than replaying the previous Note. Test the same consumer fed by a nested blank Return.
- [ ] A partially empty operand remains malformed and diagnoses under the ordinary literal rules.
- [ ] Cover Number and Note operand contexts, Bang-activated roots and Terminal Output operands through Source/Tick tests.
- [ ] Record Blank Answer separately from the Absence Marker in ADR 0062 and the domain glossary. This explicit encoding applies only when an inline operand is blank; ordinary absence from unequal Equality or a non-firing pulse remains a no-write result.
- [ ] The Function reference agrees with blank-operand behavior; a failed computation remains distinguishable from deliberate absence.
