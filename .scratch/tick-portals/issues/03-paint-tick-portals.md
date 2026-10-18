# 03: Paint Tick Portals in the Source Grid

**What to build:** Fold the record into `RenderCell` role bits in
`RenderFrame::derive`, add the Input Portal theme role, and paint it with the
precedence in spec section 6.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `RenderCell` carries role bits in place of `output_portal: bool`; no new
      allocation per frame, pinned by an allocation assertion.
- [ ] Input Portal in both built-in themes and the contrast checks.
- [ ] `Retained` records paint only entries whose owner still stands.
- [ ] Tests: overlapping read and write roles, empty read Cells, existing
      precedence (Portal over Diagnostic, Function and Bang exceptions, Cursor
      and Region over Portal), refused writes unpainted.
- [ ] A benchmark of the derive with a record present.
- [ ] A kittest showing the tracker's `&v` reads lit as it steps.
