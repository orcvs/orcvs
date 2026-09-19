# Function reference

A complete Function reference the console can load from its File menu, so the language can be read and experimented with in place.

- Every Function in the Function table appears once, with a worked example: the Expression, and on the row south the result a Tick writes.

      .+0101
      02

      .-1001
      09

- Examples are organised in columns aligned to the Sector Seams and grouped logically, each group headed by a Comment.
- The Grid is as large as the reference needs; its dimensions come from the reference, not from the blank default.
- The reference is loaded on demand, replacing the current Source. A console with no stored Source still opens the blank default Grid (`source-view/03`, ADR 0045).
- The reference is its own test: ticking it writes what it says it writes, and every Function is present.

Out of scope: how written results paint. Until `syntax-highlighting/05`–`07` land, a written result such as `02` parses as refused Function spellings and paints as a diagnostic.
