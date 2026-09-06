# Source Functions plan spatial effects

Timing, ordering, and Bang-lifetime clauses below are superseded where they conflict with [ADR 0032](0032-schedule-tick-execution-by-dependency.md). Other decisions remain in force.

Orcvs adapts Orca's spatial read, write, and generation capabilities as Source Functions rather than reducing every Function to its explicit operands. A Source Function reads working Source at its turn under [ADR 0032](0032-schedule-tick-execution-by-dependency.md) and may resolve any Cell as a Portal. A Source-writing Function validates its complete effect bundle before contributing any Portal write to the Tick Plan; one invalid destination diagnoses and rejects the bundle. A bundle may contain multiple Portals and explicit clears, represented as spaces written to Source rather than Empty values.

Writes retain the Function's producer anchor and emission order under ADR 0020. This makes overlapping writes deterministic: a Self-Banging Function clears its current Span before writing its spelling at the shifted destination, so the later write wins at a shared horizontal Cell. An admitted change is visible to later turns in the same Tick; an earlier root is never revisited. Source-reading Functions may be nested; Source-writing and external-output Functions are root-only and produce effects rather than values.
