# 08 — Rename Jump to Copy

Status: resolved
Blocked by: 07 — Accept the Read, Write and Copy families

**What to build:**

The four Jump Functions `&^ &v &< &>` become the Copy Functions `=^ =v =< =>` (ADR 0070). Behaviour is unchanged: each writes the pair on its arrow's side, reads the pair on the opposite side, and has static Portals. This frees the `&` arrows for the directional Reads (ticket 02).

## Acceptance criteria

- [x] `=^`, `=v`, `=<` and `=>` are in the Function table with the Jumps' declarations unchanged except for their spelling; `&^ &v &< &>` no longer parse as Jumps.
- [x] Code names follow the glossary: Jump becomes Copy in identifiers such as `Function::Jump*`, in diagnostics such as `JumpInput` and their messages, and in test module names such as `nested_jump`. Doc comments and rustdoc links follow.
- [x] Every existing Jump test passes with only spellings and names changed, including the reused-versus-fresh schedule comparison and the proptest regressions file.
- [x] Source Paint, the console's syntax colouring and any console test that spells a Jump use the Copy spellings.
- [x] The Function reference (`console/assets/function_reference.orcvs` and `console/src/function_reference.rs`) and every shipped Source File spell the Copies, with their columns realigned where a respelling moves a Language Unit.
