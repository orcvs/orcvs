# 03 — Name and describe every Function in the Function table

Status: ready-for-agent
Type: task

**What to build:** each row of the Function table in `lang/src/atom.rs` carries the Function's
display name and a one-line summary of what it answers or does ([spec](../spec.md), ADR 0100).

- [ ] The name is the words of the Function's heading in `docs/functions.md` (`.|` is
      `Absolute Difference`).
- [ ] The summary is one short line in the reference's voice, such as `answers left + right`.
- [ ] A test reads every heading in `docs/functions.md` and fails when a heading's spelling and name
      do not match a row, or a row has no heading, alongside the existing
      `orcvs/tests/function_reference_doc.rs`.
- [ ] Operand role names stay where the table already declares them; the Cursor Line reads them
      from there.
