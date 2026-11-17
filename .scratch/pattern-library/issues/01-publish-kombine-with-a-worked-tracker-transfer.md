# 01 — Publish Kombine with a worked Tracker transfer

Status: ready-for-human
Blocked by: None — can start immediately

**What to build:** A musician opens Kombine from Help → Examples, hears three
pitches against four accents, changes the accent period, then copies its accent
Region into Tracker's closed hi-hat using the guide. This complete entry
establishes the library's first release.

- [x] Ship a playable Source File and sibling guide, register Kombine under
      Help → Examples, and provide an index beside the examples linking the
      Source and guide. Registration is two edits today: `EXAMPLES` in
      `console/src/examples.rs` and the positional `EXAMPLE_CONFIRMATIONS` in
      `console/src/console/files.rs`. `example-discard/01` removes the second
      list and has no ordering against this ticket, so whichever lands second
      adapts. A registered example also gains a web link name, as every
      registered example does.
- [x] Cycle C4, E4 and G4 against four velocity steps with a distinct loud
      accent. Specify cadence, channel and Note lifetime. Verify the delivered
      twelve-Tick pitch/velocity cycle, including its wrap.
- [x] Ship four distinct velocities so each step can be told apart, replacing
      the study's `70 40 30 40`, which repeats `40`. Document and verify
      shortening the accent cycle to three steps by editing both Clock `steps`
      and Track `count`. The loud accent then returns to C.
- [x] Place an Accent Comment above the copyable Clock, Track and velocity
      data. A diagram gives the Region's corners, runtime write space, output pair
      and its destination in Tracker. Q10 confirms it for this Region.
- [x] Give a worked transfer into Tracker with the destination Position,
      address changes, connection to the closed hi-hat's velocity operand and
      the expected accents after transfer.
      Set the accent Clock division to two so successive hat hits sample all
      four velocities. Replacing the hat Play's two-Cell velocity literal with
      a Read widens that operand and moves the Read east of it, so the guide
      names every Cell that moves to keep the hat's Read under its Add and in
      its pattern column, or Writes into the literal's Cells instead.
- [x] Verify the actual transferred Source, the four distinct velocities, observed in
      order on the hat's four successive scheduled steps, and unaffected
      neighbouring voices. Delete a hat hit and verify that later accents keep
      their absolute-Tick phase.
- [ ] Follow the published instructions in the console and listen on the
      Built-in Synth. Record the observed result and fix discrepancies in the
      guide or Source before calling the entry complete.
- [ ] Exercise the shipped Source and advertised edits through existing
      playback testing conventions, checking delivered output and diagnostics.
      Complete the repository gates applicable to the changes and record
      automated, console and listening evidence distinctly.

## Comments

**17 November 2026 — implementation on `claude/project-thread-4dv2du`.** Six
of the eight boxes are done. The console box and the evidence box stay open
until a person follows the guide in the console and listens on the Built-in
Synth.

Automated evidence:

- `orcvs/src/playback/tests/shipped_kombine.rs` runs the shipped
  `examples/kombine.orcvs` through `HandDrivenRun`. It checks the twelve-Tick
  pitch and velocity cycle and its wrap at Ticks 12 and 13, and the three
  accent steps, where the loud `70` lands on every C4, and each of the two
  single-operand edits the guide warns about. Each expected delivery is
  written out Tick by Tick. No Tick diagnoses anything. At load the Language
  Map diagnoses no Function, and diagnoses each Cell of the Tracks' data,
  and nothing else, as an unknown Function and as a character that starts no
  Language Unit. The guide draws every row the tests run.
- `orcvs/src/playback/tests/shipped_trackers.rs` builds the transferred
  Tracker from both shipped files and the guide's edits, and checks that the
  guide draws each changed row: rows 6, 7, 32 and 33, and row 34 empty, as
  it stands before the first Tick. Over two loops the hat plays `70 30 50 40` in
  order, every other voice keeps `77`, and nothing is diagnosed. With the hat
  hit on step 2 cleared, step 4 still plays `50` and step 8 still plays `70`.
- In the transferred Tracker, the hat's Play waits on the accent Track below
  the pattern. Playback therefore receives its Note after the other voices'
  Notes on the same Tick. Both arrive on one Tick, so this order should not
  be audible.

Console evidence (automated in a headless harness, not by a person):

- `the_kombine_accent_region_pasted_into_the_tracker_accents_the_closed_hat`
  is a kittest in `console/src/console/kittest_tests/live_editing.rs`. It
  follows the transfer with the console's menus, keys and clipboard events:
  Help > Examples > Kombine, Shift-arrow Region, Copy, Help > Examples >
  Tracker, Paste at (2, 32), type `2`, cut and paste the hat's Every, then
  Delete and retype the Play, pasting what the Copy and the Cut put on the
  clipboard. Ticks run against the console's Source and show the same
  accents, and every other voice plays the shipped Tracker's notes on the
  same Ticks.
- `three_kombine_accent_steps_typed_in_the_console_keep_the_loud_accent_on_c`
  types the three-step edit.
- Following the guide in the harness found one discrepancy, which the guide
  now fixes. Space starts Playback rather than typing an empty Cell, so the
  hat cannot be moved by typing a row that holds spaces. The guide now uses
  a cut and paste for the Every, and Delete then arrow keys for the Play.

Offline render (not listening evidence, and not reproducible from the
repository): a scratch program, not committed, sent the delivered Notes
through `synth::Synth`, the Rust Built-in Synth, to WAV. On the hat alone, the
transferred Tracker's onset peaks are 0.196, 0.085, 0.141 and 0.113 for `70`,
`30`, `50` and `40`, in proportion to velocity. The shipped Tracker's hat is
0.209 on every hit. Neither the program nor the WAV files are in the
repository, so these figures are a note, not evidence a reviewer can rerun.

Remaining for a person:

- Open Kombine in the console and play it on the Built-in Synth. Make the
  three-step edit, follow the transfer into the Tracker and listen to the hat.
- Record what you hear, and fix the guide or Source wherever the result
  differs.
- Listen for whether the accents stand out in the full mix, beside the kick
  on the same steps. The loudest accent, `70`, is slightly below the shipped
  hat's `77`.
