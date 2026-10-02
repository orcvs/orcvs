# 08 — Refresh the inventory's Current state against the code

**What to build:** The "Current state" column of `01`'s shipped-language inventory states what the code does today, so `v1-release/03` can read it as the inventory of record. The definition of done requires this refresh before the inventory is used as authority.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Every inventory row's Current state is checked against the Function definitions in `lang` and the behaviour tests, and corrected. The column last refreshed on 2026-09-09 still calls the Self-Banging movement, the Sequence family, the Tick family and the Jump Functions missing; all of them are now defined.
- [x] Every residual owner the column names is checked, and resolved owners are removed or replaced by what now owns the gap, if anything does.
- [x] Each row links the tests that prove it, so `v1-release/03`'s inventory-to-evidence mapping can start from this table.
- [x] Any member whose Current state is still short of the release is named with its owning `release/v1` issue, or recorded as an accepted deferral.
- [x] The refresh date and the commit it was checked against are recorded.

## Answer

`01`'s Answer now carries the refreshed column, dated 2026-10-01 and checked against `98bd5d0d`
(`origin/main`). Every member was read against its row in `define_functions!` and its behaviour
tests, and every residual owner the old column named is resolved. The four families
that were prose are now tables, one row per Function or behaviour, so each row has an Evidence cell.
A script read every `module::test` that `01` cites and found each one in the file named beside it:
186 citations of 182 distinct tests.

Members still short of the release are listed in `01`'s
[Members still short of the release](01-name-the-shipped-language-inventory.md#members-still-short-of-the-release).
No deferral was added.

Three things were found here and not acted on. They are outside this ticket's scope, so they are
listed for triage:

- `midi-port-ownership/11` is a browser port-reselection gap. It was untagged when found, and was
  triaged into `release/v1` on 2026-10-02; `01` names it as the Browser Web MIDI row's owner.
- `v1-release/definition-of-done.md` names only the native physical MIDI smoke. The browser manual
  checks in `midi-port-ownership/06`, `09` and `10` are tagged `release/v1`, but the definition of
  done has no line for them.
- `spatial-tick-planning/09` is the shipped Terminal Output Portal refusal. It is `needs-triage`
  and untagged. It is not an inventory member's state, because Source cannot yet assign an Output
  Portal to a Terminal Output Function.

## Comments

**2026-09-24 — opened by the release-membership audit.** `01` stays resolved: its decision stands and only its state column is stale. Blocks `v1-release/03`.

### Issue audit against d3fd1b27 — 2026-10-01

The inventory column is still stale. Its named residual owners (sequence-values/06,
spatial-tick-planning/02, midi-output-family/06) are resolved. The MIDI row should also record that
the browser has a Web MIDI backend (#185, ADR 0059).
