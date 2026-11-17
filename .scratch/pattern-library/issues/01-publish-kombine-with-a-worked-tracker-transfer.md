# 01 — Publish Kombine with a worked Tracker transfer

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:** A musician opens Kombine from Help → Examples, hears three
pitches against four accents, changes the accent period, then copies its accent
Region into Tracker's closed hi-hat using the guide. This complete entry
establishes the library's first release.

- [x] Ship a playable Source File and sibling guide, register Kombine under
      Help → Examples, and provide an index beside the examples linking the
      Source and guide.
- [x] Cycle C4, E4 and G4 against four velocity steps with a distinct loud
      accent. Specify cadence, channel and Note lifetime. Verify the delivered
      twelve-Tick pitch/velocity cycle, including its wrap.
- [x] Document and verify shortening the accent cycle to three steps by editing
      both Clock period and Track count. The loud accent then returns to C.
- [x] Place an Accent Comment above the copyable Clock, Track and velocity
      data. A coordinate diagram gives the Region's corners, runtime write
      space and output pair.
- [x] Give a worked transfer into Tracker with the destination coordinates,
      address changes and connection to the closed hi-hat's velocity operand.
      Set the accent Clock division to two so successive hat hits sample all
      four velocities.
- [x] Verify the actual transferred Source, all four velocities and unaffected
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
  accent steps, where the loud `70` lands on every C4. Each expected delivery
  is written out Tick by Tick. No Tick diagnoses anything, the Language Map
  diagnoses no Function at load, and the guide draws every row the tests run.
- `orcvs/src/playback/tests/shipped_trackers.rs` builds the transferred
  Tracker from both shipped files and the guide's edits, and checks that the
  guide draws each changed row. Over two loops the hat plays `70 30 50 40` in
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
  Delete and retype the Play. Ticks run against the console's Source and
  show the same accents.
- `three_kombine_accent_steps_typed_in_the_console_keep_the_loud_accent_on_c`
  types the three-step edit.
- Following the guide in the harness found one discrepancy, which the guide
  now fixes. Space starts Playback rather than typing an empty Cell, so the
  hat cannot be moved by typing a row that holds spaces. The guide now uses
  a cut and paste for the Every, and Delete then arrow keys for the Play.

Offline render (not listening): a scratch program sent the delivered Notes
through `synth::Synth`, the Rust Built-in Synth, to WAV. On the hat alone, the
transferred Tracker's onset peaks are 0.196, 0.085, 0.141 and 0.113 for `70`,
`30`, `50` and `40`, in proportion to velocity. The shipped Tracker's hat is
0.209 on every hit. The program and the WAV files are not part of the change.

Remaining for a person:

- Open Kombine in the console and play it on the Built-in Synth. Make the
  three-step edit, follow the transfer into the Tracker and listen to the hat.
- Record what you hear, and fix the guide or Source wherever the result
  differs.
- In the full mix the kick dominates each step's peak. The loudest accent,
  `70`, is also slightly below the shipped hat's `77`, so listen for whether
  the accents stand out enough.
