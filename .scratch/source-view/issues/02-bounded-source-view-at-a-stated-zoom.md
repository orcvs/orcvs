# 02: Bounded Source View at a stated Zoom

**What to build:** The console stops fitting the Source to the window. The Source View holds a Zoom, opening at 1.0, and a Pan anchored at the console's top-left. A resize shows more or less of the Source and never changes the Cell size. The viewer Pans by wheel or two-finger scroll and by middle-drag, as far as the Grid's edges and no further. See ADR 0045.

**Blocked by:** 01

**Status:** done

- [x] The console opens at Zoom 1.0 whatever the window size.
- [x] Where the Source is smaller than the console on an axis, it sits at the console's top-left on that axis and does not Pan.
- [x] Where the Source is larger, wheel, two-finger scroll and middle-drag Pan it as far as its own edges and no further.
- [x] A resize that would open a gap past an edge settles the Source View back inside the Grid.
- [x] Pinch and command-wheel no longer Zoom.
- [x] The double click on the letterboxing that returned to the fit is gone, along with the fit.
- [x] Painting and clicking still go through one presented geometry: a click after a Pan selects the Cell under the pointer.
