# 03 — Name and describe every Function in the Function table

Status: resolved
Type: task

**What to build:** each row of the Function table in `lang/src/atom.rs` carries the Function's
display name and a one-line summary of what it answers or does ([spec](../spec.md), ADR 0100).

- [x] The name is the words of the Function's heading in `docs/functions.md` (`.|` is
      `Absolute Difference`).
- [x] The summary is one short line in the reference's voice, such as `answers left + right`.
- [x] A test beside `orcvs/tests/function_reference_doc.rs` reads every `### ` heading in
      `docs/functions.md` as ``### `<spelling>` <name>`` and fails when a heading does not have that
      shape, matches no row, or names a row differently, when a row is headed twice, and when a row
      has no heading. Each row having exactly one heading makes the heading count equal the table's
      row count (51 today), so a format change cannot let it pass by matching nothing.
- [x] Each Function's Roles are declared on its row of the Function table, a Position's two Numbers
      as one Role; the Cursor Line reads them from there.

## Answer

Each row of `define_functions!` in `lang/src/atom.rs` now carries the name and summary after its
spelling, and `Function::name`, `Function::summary` and `Function::roles` answer them; `roles` reads
the Roles the row declares. `every_heading_names_its_function_as_the_table_does` in
`orcvs/tests/function_reference_doc.rs` holds every heading to its row and each row to exactly one
heading, which makes the heading count equal `Function::ALL`'s 51 and fails a scan that matches
nothing.
