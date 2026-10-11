# 05 — A Copy with partial input clears its destination

Status: wontfix

Dropped by ticket 07 D14. The clear contradicts ADR 0069 and the CONTEXT.md Copy entry ("Partial or invalid input diagnoses and writes nothing"). A Copy with partial input writes nothing, and the `**` it carried last Tick fires once, as for any producer that writes nothing (D4); ticket 02 owes the test.
