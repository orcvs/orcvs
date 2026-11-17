# Requirements snapshot audit (base c8cc197, snapshot commit 9bf2428)

Read-only audit. No files edited, no cargo run.

## Findings, ranked by severity

### F1 (High) ADR 0088's Sound Module behaviour is in no requirement

- Scenario: an agent changes `site/src/console/sound-module.ts` and asks what the Sound Module must do: channel lights, the chosen channel, typing values. No requirement answers.
- Affected: playback.md:5-8 (marker says "0088's Sound Module [is] stated in site.md"); site.md has no Sound Module rule. SITE-031 (site.md:147-154) only places the row. console.md:600 (CONS-079) also says "the Sound Module itself: site.md".
- Source: ADR 0088 l.22 "takes the Function role's colours, and goes dim again 150 ms later. A channel shows no level"; l.55-58 "The reader changes a value by typing... A digit that would take the value past `7F` is refused"; l.139-144 (2026-11-14 amendment) "A click on a channel's digit... The chosen channel is outlined in the magnitude colour. Channel `00` is chosen at load, and until the reader chooses one the chosen channel follows the channel of [each effect CC]... the reader's typing sends each change as a [CC]". The trace (`.scratch/requirements-snapshot/trace/playback.md:174,176,182`) marks these "redirected — SITE". `trace/site.md:82` redirects the behaviour back to PLAY ("behaviour redirected PLAY (0088)"). Each side redirects to the other, so the clauses are dropped. The code exists (`site/src/console/sound-module.ts`).
- Consequence: still-binding obligations of an accepted ADR, recently amended, have no rule. The CONTEXT.md "Sound Module" entry is now the only restatement, and that puts operational policy in the glossary.
- Fix: add a "Sound Module" section to site.md (for example SITE-035..038) covering the drawing as Cells, the 150 ms channel lights, the chosen channel (load `00`, follows the CC stream, click/keys choose, outline), value typing (refuse past `7F`, send CC on the chosen channel), and the `CU`/`DC` columns on `08`. Add 0088 to site.md's Source ADRs marker and to the ADR README Site row. Correct trace/site.md:82 and trace/playback.md:174-182.

### F2 (High) CONS-028 states a retired concept as binding, and its evidence does not exist

- Affected: console.md:151-167.
- Rule text: "A Sequence-capable root highlights: 1. Up to four Cells... 2. Past those, the run of written Cells, ending at the first blank Cell... a one-Cell gutter ends the highlight". It also says "The Language Map holds no fitted highlight".
- Source: ADR 0063/0067 remove the Sequence value, so no root is "Sequence-capable" (the docs/adr/README.md words table says so itself). The code computes the highlight in the Language Map from Output Portal Reservations alone: `orcvs/src/source/language_map.rs:628` `output_portal_highlight` ("Whether each Cell ... lies in an Output Portal Reservation"). `SourceRevision::output_portal_highlight` (orcvs/src/source/mod.rs:190) only delegates to it. The cited tests `a_blank_gutter_cell_stops_the_highlight_whatever_follows_it` and `a_one_cell_gutter_stops_the_left_columns_sequence_tint` exist nowhere in the tree. These were the only 2 of 472 Evidence identifiers that do not resolve. The "ruled 2026-09-19, no ADR" citation is also wrong: `.scratch/syntax-highlighting/issues/12-*.md:42` records "Found from a screenshot on 2026-09-19". That is a bug report, not a maintainer ruling.
- Consequence: the requirement states a rule for a value the language no longer has, quotes "Sequence" vocabulary as current, and cites evidence that does not exist. The code's location (Language Map) also contradicts ADR 0052 l.7 ("does not keep ... a fitted Output Portal highlight"), but that clause concerned fitting, which is now moot. The real current rule (every value answer highlights its reserved pair) is unstated.
- Fix: restate CONS-028 as "each value answer's Output Portal Reservation, root or nested, is highlighted; the highlight is not part of a Claim". Record the ADR 0052 location/fit clauses as superseded in effect by 0063, or as an open decision. Cite the tests in `orcvs/src/source/mod.rs` `mod output_portal_highlight` (l.738) and `language_map.rs:1707`. Drop the "ruled" label.

### F3 (Medium) CONS-076 states proposed ADR 0079's content as binding against accepted ADR 0053

- Affected: console.md:614-620, "No Theme property controls motion, and nothing a Theme styles is animated."
- Source: accepted ADR 0053 l.29: "A Theme decides how things look, never how much they move." The clause "so nothing a Theme styles moves" is in the parenthetical "Amended by ADR 0079", which is proposed. ADR 0053 l.35 (accepted, unamended): "Animated strokes vary around the nominal width ... retain the existing animated fragment variation of 0.45–1.25 times that width".
- Consequence: a proposed decision reads as binding, which D6 forbids, and an accepted clause disappeared without a gap entry.
- Fix: keep "no Theme property controls motion" as the binding rule. Move "nothing a Theme styles is animated" into "Proposed, built" (CONS-121 already covers the still outline). List the 0053 animated-stroke clause under "built differently" (removed in code under proposed 0079).

### F4 (Medium) The same rules have two owners: PLAY vs SITE on the synth

- PLAY-540 (playback.md:762-777) and SITE-043 (site.md:200-208) both state "fixed delay, 50 ms target, a message later than the delay sounds at once". The D9 gap (synth-host drops messages >500 ms late during start-up) is listed only under SITE-043, so PLAY-540 reads as fully built.
- PLAY-520 (playback.md:747-749) also states "A synth the page announces as unavailable ... answers the page's reason as a connect failure" and "A console no page frames offers no Built-in Synth". These duplicate SITE-046 and SITE-048.
- Source: ADR 0087; D2 assigns the timing to SITE ("SITE owns ... the page message protocol and timing").
- Fix: PLAY-540 keeps "each message carries how late its Tick is" and links to SITE-043 for the delay. Strip the unavailable/no-page clauses from PLAY-520 and link SITE-046 and SITE-048.

### F5 (Medium) A CONTEXT.md entry still carries policy that conflicts with PLAY-260

- CONTEXT.md:290 (Playback Engine): "a refused submission leaves it standing and the stop is delivered again at the next executed Tick".
- PLAY-260 (playback.md:447-449) binds ADR 0016: teardown after a refused delivery "end[s] all Timed and Mono ownership". The retry is listed as a gap at playback.md gaps section.
- Consequence: the glossary states, as current behaviour, the reverse of the binding rule. D8 trimmed other entries but missed this one.
- Fix: drop the retry sentence from the CONTEXT entry, or point it to PLAY-260 and its gap.

### F6 (Medium) Dated rulings that are not at the cited path

- console.md:685 (CONS-130), :704 (CONS-131) cite "ruled 2026-10-28, no ADR (`.scratch/menu-structure/spec.md`)". The spec has no date and no maintainer ruling (grep for 2026-10-28 or Toby finds nothing).
- console.md:723 (CONS-133) cites "ruled 2026-10-28 (`.scratch/site-as-console/spec.md`, `orcvs-site/13`)". site-as-console/spec.md has no 2026-10-28 entry, and orcvs-site/13 l.5 says "Toby decided this on 2026-10-26".
- console.md:166 (CONS-028): see F2.
- Verified correct: evaluation.md:85 (bang-function/08 l.36, "2026-10-13 — Decided by Toby"); site.md:153,164 (site-as-console spec l.8,15, Toby 2026-10-26); language.md:229,495 (chords-and-pitch spec l.5, "Settled in design on 2026-11-10", l.106 "The Pitch family is decided here"; acceptable, though it names no maintainer).
- Fix: CONS-133 becomes "ruled 2026-10-26 (orcvs-site/13)". Under D3, CONS-130/131's per-key fallback and document model should move to Open decisions (undated spec decision), as site.md did for SITE-022/023, or cite a real dated ruling.

### F7 (Low) The ADR-to-topic table in docs/adr/README.md disagrees with the markers

- The table lists each ADR once, splitting only 0087 and 0103. The markers claim more:
  - evaluation.md:5 lists 0067, 0071, 0072, 0073 and 0076; the table lists them only under Language/Playback.
  - site.md:5 lists 0054, 0089 and 0102; the table has them only under Language/Console.
  - console.md:5-8 lists partial clauses of 0034, 0054, 0055, 0059, 0061, 0069, 0076, 0077, 0085, 0088, 0091 and 0105; the table omits all of them.
  - With F1, 0088 belongs under Site.
- Fix: annotate the splits in the table as for 0087/0103, or say the table names the primary owner and the markers list the rest.

### F8 (Low) The "Words the older ADRs use" table is missing two renames

- "Delay `~*`" (ADRs 0012, 0039, 0071, 0076) is now the Every Function (CONTEXT.md:201-203; `.scratch/musical-terminology/audit.md:12`).
- "Jump Function `&^ &v &< &>`" (ADRs 0008, 0014, 0067, 0071) is now Read (CONTEXT.md:145 `_Avoid_: Jump`; ADR 0070).
- The existing six rows are accurate.
- Fix: add both rows. For Every, name the renaming change, or "no ADR (musical-terminology)" if none.

### F9 (Low) Broken anchors and phantom IDs

- console.md:58 `#cons-040-the-console-has-one-zoom-egui-s-whole-ui-zoom` and console.md:556,1018 `#cons-029-a-cell-s-tick-portal-roles-...`: GitHub drops the apostrophe, so the slugs are `eguis` and `cells`. The links are dead.
- site.md Open decisions uses IDs SITE-023 and SITE-082, which are not rules. Mark them clearly as non-rule labels or drop the IDs.

### F10 (Low) Process wording differs across entry points

- docs/agents/domain.md allows "or marks that topic's snapshot as behind", and so does docs/requirements/README.md.
- docs/adr/README.md and AGENTS.md say only "in the same change" (D1).
- Fix: harmonise. Either all mention the "behind" escape, or none does.

## Checked and found correct

- Check 5: `git diff --stat c8cc197..HEAD -- docs/adr` shows only README.md (+32).
- No ADR is missing from the topic table. Every file 0001–0109 appears; 0027, 0064 and 0106 do not exist.
- Amendment chains:
  - 0062→0066→0069→0108(+0073/0075/0104/0107, 0109): LANG-280..330. Correct outcomes; Pending stated as no state; defaulted `legato` included.
  - 0045→0047→0084(+0054/0058/0102): CONS-030..035. Margin dropped; 1024×640 and grab hand kept as 0084 says.
  - 0080→0081→0096(+0098): CONS-090..095 match 0096's corner, one pixel per Cell and hiding.
  - 0093→0101: SITE-070..081 match 0101's decisions. The D9 concurrency gap is real (`console-web.yml:93-95`).
  - 0076/0089: PLAY-150..170, CONS-099/100.
  - 0021/0067 standalone literals: LANG-130 plus open decision D4. Ruling confirmed in source-diagnostics/01 l.21.
  - 0016 teardown: PLAY-260 binds the ADR, retry listed as a gap.
  - 0055: PLAY-120 four classes, NetworkFailure gap listed.
  - 0105: 39 names, amendment honoured.
  - 0006/0032: EVAL-050 plus D5 open decision is defensible.
- Proposed 0078/0079 are kept in "Proposed, built" (CONS-120..123), except F3.
- Evidence: all 472 test/symbol identifiers resolve except F2's two. The following were opened and assert what their rules claim:
  - `a_tick_conflicting_on_every_optimistic_attempt_commits_once_under_the_lock` (`OPTIMISTIC_TICK_ATTEMPTS`=3)
  - `all_notes_off_releases_tones_and_leaves_drums_ringing`
  - `an_operand_literal_outside_a_slot_is_not_an_expression`, and the model.rs assertion string
  - `a_nested_clear_under_a_bang_driven_output_binds_absent_in_a_note_and_as_00_in_legato`
  - `an_inert_directional_bang_function_emits_nothing`
  - `the_minimap_hides_where_one_pixel_per_texel_does_not_fit_in_the_source`
  - `the_default_window_is_a_fixed_size_in_points`
  - `a_pending_operand_keeps_its_declared_colour_rather_than_diagnostic`
  - config keys at `config.rs:168-173` and their tests
  - `AUDIO_WAIT`=0.5 s and `DELAY_MS`=50
  - synth.spec.ts offset test (planned+50 ms within 2 ms; it does not cover the "late sounds at once" path)
  - smoke.spec.ts tests for SITE-030/031
  - help-menu kittest
- Cross-document rule references (PLAY-180/190 from evaluation.md) point to the right rules. No duplicate rule IDs.
- CONTEXT.md diff:
  - The Operand Literal change matches D4.
  - The 0029→0061 citation fix is correct.
  - The Clear entry now includes 0109.
  - Pan adds the finger (0102).
  - The Built-in Synth entry is trimmed to a pointer.
- AGENTS.md and domain.md routing is correct, and neither lists manifest-owned commands or config.
