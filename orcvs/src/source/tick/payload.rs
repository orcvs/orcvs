//! The Payload of UDP `!u` and OSC `!o`: the Cells after the spelling through
//! the next `!` on the row, which the Function reads at its Turn.
//!
//! Equality at (0, 0) Bangs the root at (2, 1) on every Tick.

use lang::{NetworkSend, OscArgument, OscMessage, Tick};

use super::observed::{diagnostic, observed, rows_of, source_of};
use crate::grid::Grid;
use crate::source::Source;
use crate::source::language_map::LanguageUnitKind;

/// The Grid every test here draws, 24 Cells wide, so row 1 begins at Cell 24
/// and the Payload of a root at (2, 1) at Cell 28.
fn grid() -> Grid {
    Grid::with_shape(24, 4)
}

/// A Source with the Equality that Bangs row 1's root at (2, 1) on row 0,
/// beside `beside`, `root` on row 1, and `below` on the rows after it.
fn banged_over(beside: &str, root: &str, below: &[&str]) -> Source {
    let rows = [format!(".=0101{beside}"), format!("  {root}")];
    let rows: Vec<&str> = rows
        .iter()
        .map(String::as_str)
        .chain(below.iter().copied())
        .collect();
    source_of(grid(), &rows)
}

fn banged(beside: &str, root: &str) -> Source {
    banged_over(beside, root, &[])
}

fn udp(bytes: &[u8]) -> NetworkSend {
    NetworkSend::Udp(bytes.into())
}

/// Every Payload's Span and text in `source`'s current revision.
fn payloads(source: &Source) -> Vec<(std::ops::Range<usize>, std::ops::Range<usize>)> {
    source
        .language_map()
        .units()
        .filter_map(|unit| match unit.kind() {
            LanguageUnitKind::Payload(text) => Some((unit.span().range(), text.range())),
            _ => None,
        })
        .collect()
}

#[test]
fn a_banged_udp_function_sends_its_payload() {
    for (root, sent) in [
        ("!uhello!", b"hello".as_slice()),
        ("!uhello", b"hello               ".as_slice()),
        ("!u a  b!", b" a  b".as_slice()),
    ] {
        let mut source = banged("", root);
        let plan = source.execute(Tick::ZERO);
        assert_eq!(plan.network_sends, [udp(sent)], "{root:?}");
        assert!(
            plan.diagnostics.is_empty(),
            "{root:?}: {:?}",
            plan.diagnostics
        );
    }
}

#[test]
fn an_empty_payload_sends_nothing_and_reports_nothing() {
    let mut source = banged("", "!u!");
    let plan = source.execute(Tick::ZERO);
    assert!(plan.network_sends.is_empty());
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

#[test]
fn an_unbanged_payload_function_sends_nothing() {
    let mut source = source_of(grid(), &["", "  !uhello!"]);
    assert!(source.execute(Tick::ZERO).network_sends.is_empty());
}

#[test]
fn a_banged_osc_function_sends_its_message_or_diagnoses() {
    let mut source = banged("", "!o/synth/freq 40 7F!");
    assert_eq!(
        source.execute(Tick::ZERO).network_sends,
        [NetworkSend::Osc(OscMessage {
            address: "/synth/freq".to_owned(),
            arguments: vec![OscArgument::Int(64), OscArgument::Int(127)],
        })]
    );

    let mut source = banged("", "!osynth!");
    let plan = source.execute(Tick::ZERO);
    assert!(plan.network_sends.is_empty());
    let tick = observed(&source, &plan);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(
            2,
            1,
            "OSC address \"synth\" does not start with \"/\""
        )]
    );
}

#[test]
fn a_write_into_a_payload_lands_in_the_same_tick() {
    // The absolute Write is a dynamic writer, and the Addition writes its
    // result south into the Payload at a reservation: both take their Turn
    // before the root that reads those Cells.
    for (beside, sent) in [("@$0601C4", b"heC4o"), (".+0102", b"he03o")] {
        let mut source = banged(beside, "!uhello!");
        let plan = source.execute(Tick::ZERO);
        assert_eq!(plan.network_sends, [udp(sent)], "{beside:?}");
        assert!(
            plan.diagnostics.is_empty(),
            "{beside:?}: {:?}",
            plan.diagnostics
        );
    }
}

#[test]
fn a_closer_a_write_puts_in_a_payload_is_sent_and_closes_it_from_the_next_parse() {
    // The Copy at (6, 2) writes the `!>` south of it north, into the Payload.
    let mut source = banged_over("", "!uhello!", &["      =^", "      !>"]);
    assert_eq!(payloads(&source), [(28..34, 28..33)]);

    let plan = source.execute(Tick::ZERO);
    assert_eq!(plan.network_sends, [udp(b"he!>o")]);
    assert_eq!(rows_of(&source)[1].trim_end(), "**!uhe!>o!");
    // Committed and parsed again, the written `!` closes the Payload, and
    // what follows it is Source.
    assert_eq!(payloads(&source), [(28..31, 28..30)]);
}

#[test]
fn a_write_over_the_closer_moves_it_from_the_next_parse() {
    let mut source = banged("@$0901C4", "!uhello!");
    let plan = source.execute(Tick::ZERO);
    assert_eq!(plan.network_sends, [udp(b"hello")]);
    assert_eq!(payloads(&source), [(28..48, 28..48)]);
}

#[test]
fn the_cells_after_an_early_closer_are_source_again() {
    let source = banged("", "!uhi!c000140");
    let map = source.language_map();
    let diagnosed: Vec<usize> = map
        .lexical_diagnostics()
        .map(|diagnostic| diagnostic.start())
        .collect();
    assert_eq!(payloads(&source), [(28..31, 28..30)]);
    assert_eq!(diagnosed.first(), Some(&31), "{diagnosed:?}");
}

#[test]
fn a_payload_function_is_invalid_where_a_value_is_required() {
    let mut source = banged("", ".+!uab!03");
    let plan = source.execute(Tick::ZERO);
    assert!(plan.network_sends.is_empty());
    assert!(
        source
            .language_map()
            .diagnostics()
            .any(|diagnostic| diagnostic.message
                == "a Function that answers an effect is valid only at the root of an Expression"),
    );
}
