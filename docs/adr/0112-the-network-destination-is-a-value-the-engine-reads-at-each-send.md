# The network destination is a value the engine reads at each send

Status: proposed on 2026-11-15. Drafted for Toby from the built engine and console; it decides
nothing the code does not already do. Amends ADR 0085's "saved with the console's other settings":
the console stores the destination itself, in eframe storage, natively and only with the
`persistence` feature, and never in the settings file. Records the one Playback handle request that
travels outside the mailbox slots of [ADR 0056](0056-playback-requests-wait-in-slots-not-a-queue.md),
whose slots and order are otherwise unchanged.

[ADR 0085](0085-udp-and-osc-send-the-cells-up-to-their-closer.md) makes the UDP and OSC destination
"a console setting, never Source. One host and two ports, saved with the console's other settings".
ADR 0056 says every request a Playback handle makes has a slot in the engine's mailbox, and lists
the slots: a stop flag, two tempo periods and one MIDI destination change. The network destination
is set through the same handle and is in neither list. Since ADR 0053's 2026-10-10 amendment the
console's other settings live in `~/.orcvs/config.toml`, which the console only reads (CONS-130), so
"saved with" them cannot be met as written. `docs/requirements/playback.md` lists the path as built,
awaiting a record
([requirements-snapshot/06](../../.scratch/requirements-snapshot/issues/06-decide-the-network-destination-path.md)).

## Decision

**The destination reaches the engine on its own newest-wins channel.**
`PlaybackEngine::set_network_destination` in `orcvs/src/playback.rs` replaces the value in a
`tokio::sync::watch` channel. A later call overwrites an earlier one, so at most one destination is
ever pending. The call never fails, never waits and never wakes the engine's task, including after
the task has gone.

**The engine reads it at each Tick that makes a Network Send.** `Outputs::deliver` in
`orcvs/src/playback/outputs.rs` reads the newest value once per Tick, after the Tick's MIDI and
before its Network Sends, only when the Tick Plan holds a Network Send. Every send in that Tick goes
to that one destination. A new destination applies from the next send, as PLAY-350 states, and
clears the network failure latch so its first failure is reported (ADR 0090).

**It is not ordered against the mailbox.** The stop, tempo and MIDI destination slots are applied in
ADR 0056's fixed order when the task takes a backlog; the network destination is read at delivery
instead. It commutes with all three: beginning, retuning or stopping a run never chooses where a
send goes, and a destination is a value with no port to release, so nothing is held for a
superseded one.

**The console sets it on every engine it builds, and on every edit.** `environment` in
`console/src/console.rs` sets the console's destination on each new Orcvs, so opening another
Source keeps it. The Panel's Output fields set it again after each valid edit, and a value the
fields refuse changes nothing (`NetworkSetting::commit` in `console/src/network.rs`).

**The console stores the destination under the eframe Storage key `orcvs_network_destination`.**
`console/src/network.rs` reads it at startup (`stored`) and writes it on each eframe save (`store`).
Only the native build with the `persistence` feature writes it; without the feature the console
always starts on the default, `127.0.0.1` with UDP port `49161` and OSC port `49162`. The settings
file holds no destination, and the console never writes that file.

## Considered options

- **A fourth slot in ADR 0056's mailbox.** It is the alternative
  [requirements-snapshot/06](../../.scratch/requirements-snapshot/issues/06-decide-the-network-destination-path.md)
  names. A slot would order the destination against stop, tempo and MIDI changes and wake the task
  for it. No source records a reason to prefer either path; this record keeps the built one.
- **The settings file, as ADR 0085 wrote.** The console never writes `config.toml` (CONS-130), so
  the Output list could not save an edit there.

## Consequences

- ADR 0056's statement that each kind of handle request has a mailbox slot holds for every request
  but this one.
- A destination set while no Tick sends anything waits, unread, for the next Tick that does.
- `docs/requirements/playback.md` can state the path as a rule once this is accepted, replacing its
  "Built, awaiting a record" item.
- Tests: `console/src/network.rs` `a_stored_destination_is_read_back`,
  `an_invalid_value_is_reported_and_changes_nothing`; `orcvs/src/network.rs`
  `the_default_destination_is_orcas`.

## Open questions

- Whether the storage clause should amend ADR 0085 as written here, or the code should change to
  meet ADR 0085, is Toby's to choose; the code is recorded.
- The web build reads `orcvs_network_destination` at startup when `persistence` is on, but nothing
  on the web writes it and the web Panel shows no network fields, since a browser cannot send UDP or
  OSC (PLAY-360). The read is inert there.
- `.scratch/udp-osc-output/spec.md` and its ticket 04 still say the destination is saved "with the
  other settings"; ticket 04's answer records the storage key the code uses.
