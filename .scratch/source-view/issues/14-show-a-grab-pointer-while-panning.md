# 14: Show a grab pointer while Panning

**What to build:** The pointer announces a drag Pan: a grab hand while Alt (Option) is held over the console, and a grabbing hand while an Alt-held primary drag or a middle-drag is Panning. See ADR 0047.

**Blocked by:** 06

**Status:** resolved

- [x] Holding Alt over the console shows a grab hand.
- [x] An Alt-held primary drag and a middle-drag show a grabbing hand while they Pan.
- [x] The ordinary pointer returns once neither is under way.
