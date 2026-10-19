# 03: Light the Cells a Tick wrote

**What to build:** Every delivery at an Output Portal site is recorded, admitted
or refused, and admitted writes paint as Output Portal, so a selected write
(Write, Push, `=$`) lights where it landed this Tick. See `../spec.md`
sections 1-3 and 6.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] A write is recorded for each site delivered to, Declared or Selected,
      with an Output Span when admitted or a Write Refusal that covers every
      refusal: geometric (with the attempted Position where one exists),
      Function Replacement, and ordering.
- [ ] Output Spans can be made only from an admitted write.
- [ ] A destination that resolved for a Turn whose interpretation failed is
      not recorded.
- [ ] Admitted writes paint as Output Portal; refused writes do not paint.
- [ ] Tests: each refusal keeps its reason and paints nothing; a Cell both read
      and written paints as Output Portal; an empty read Cell shows the Input
      Portal background; order and owners hold when the Tick rewrites the Source
      its owners stood in.
