# 01 — Publish Kombine with a worked Tracker transfer

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:** A musician opens Kombine from Help → Examples, hears three
pitches against four accents, changes the accent period, then copies its accent
Region into Tracker's closed hi-hat using the guide. This complete entry
establishes the library's first release.

- [ ] Ship a playable Source File and sibling guide, register Kombine under
      Help → Examples, and provide an index beside the examples linking the
      Source and guide.
- [ ] Cycle C4, E4 and G4 against four velocity steps with a distinct loud
      accent. Specify cadence, channel and Note lifetime. Verify the delivered
      twelve-Tick pitch/velocity cycle, including its wrap.
- [ ] Document and verify shortening the accent cycle to three steps by editing
      both Clock period and Track count. The loud accent then returns to C.
- [ ] Place an Accent Comment above the copyable Clock, Track and velocity
      data. A coordinate diagram gives the Region's corners, runtime write
      space and output pair.
- [ ] Give a worked transfer into Tracker with the destination coordinates,
      address changes and connection to the closed hi-hat's velocity operand.
      Set the accent Clock division to two so successive hat hits sample all
      four velocities.
- [ ] Verify the actual transferred Source, all four velocities and unaffected
      neighbouring voices. Delete a hat hit and verify that later accents keep
      their absolute-Tick phase.
- [ ] Follow the published instructions in the console and listen on the
      Built-in Synth. Record the observed result and fix discrepancies in the
      guide or Source before calling the entry complete.
- [ ] Exercise the shipped Source and advertised edits through existing
      playback testing conventions, checking delivered output and diagnostics.
      Complete the repository gates applicable to the changes and record
      automated, console and listening evidence distinctly.
