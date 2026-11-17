# 01 — Publish Kombine with a worked Tracker transfer

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:** A musician opens Kombine from Help → Examples, hears three
pitches against four accents, changes the accent period, then copies its accent
Region into Tracker's closed hi-hat using the guide. This complete entry
establishes the library's first release.

- [ ] Ship a playable Source File and sibling guide, register Kombine under
      Help → Examples, and provide an index beside the examples linking the
      Source and guide. Registration is two edits today: `EXAMPLES` in
      `console/src/examples.rs` and the positional `EXAMPLE_CONFIRMATIONS` in
      `console/src/console/files.rs`. `example-discard/01` removes the second
      list and has no ordering against this ticket, so whichever lands second
      adapts. A registered example also gains a web link name, as every
      registered example does.
- [ ] Cycle C4, E4 and G4 against four velocity steps with a distinct loud
      accent. Specify cadence, channel and Note lifetime. Verify the delivered
      twelve-Tick pitch/velocity cycle, including its wrap.
- [ ] Ship four distinct velocities so each step can be told apart, replacing
      the study's `70 40 30 40`, which repeats `40`. Document and verify
      shortening the accent cycle to three steps by editing both Clock `steps`
      and Track `count`. The loud accent then returns to C.
- [ ] Place an Accent Comment above the copyable Clock, Track and velocity
      data. A diagram gives the Region's corners, runtime write space, output pair
      and its destination in Tracker. Q10 confirms it for this Region.
- [ ] Give a worked transfer into Tracker with the destination Position,
      address changes, connection to the closed hi-hat's velocity operand and
      the expected accents after transfer.
      Set the accent Clock division to two so successive hat hits sample all
      four velocities. Replacing the hat Play's two-Cell velocity literal with
      a Read widens that operand and moves the Read east of it, so the guide
      names every Cell that moves to keep the hat's Read under its Add and in
      its pattern column, or Writes into the literal's Cells instead.
- [ ] Verify the actual transferred Source, the four distinct velocities, observed in
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
