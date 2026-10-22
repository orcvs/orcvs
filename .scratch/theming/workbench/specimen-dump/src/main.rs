//! Prints every Cell of a Source File, as the console would paint it after a
//! few Ticks, as JSON rows: [glyph, paint, roles, diagnostic].
use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::playback::InMemoryOutputAdapter;
use orcvs::render_frame::PortalRole;
use orcvs::source::{OperandState, SourcePaint, Token, file};

fn main() {
    let path = std::env::args().nth(1).expect("a Source File path");
    let (cols, rows): (usize, usize) = (
        std::env::args().nth(2).map_or(72, |s| s.parse().unwrap()),
        std::env::args().nth(3).map_or(32, |s| s.parse().unwrap()),
    );
    let source = file::read(&std::fs::read(path).unwrap()).expect("a Source File");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _enter = runtime.enter();
    let mut orcvs = Orcvs::with_source_and_output_adapter(source, InMemoryOutputAdapter::default()).unwrap();
    if std::env::var_os("STOPPED").is_some() {
        println!("{}", dump(&orcvs, cols, rows));
        return;
    }
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    // Sample the running frames and keep the one showing the most kinds of paint fact.
    let mut best = (0, String::new());
    for _ in 0..80 {
        std::thread::sleep(std::time::Duration::from_millis(40));
        let dump = dump(&orcvs, cols, rows);
        let kinds: std::collections::HashSet<&str> = dump.split("],[").filter_map(|c| c.split(',').nth(1).zip(c.split(',').nth(2))).map(|(a, _)| a).collect();
        let roles = ["\"O\"", "\"I\"", "\"L\"", "\"b\"", "!"].iter().filter(|r| dump.contains(*r)).count();
        let score = kinds.len() + roles * 3;
        if score > best.0 { best = (score, dump); }
    }
    println!("{}", best.1);
}

fn dump(orcvs: &Orcvs<()>, cols: usize, rows: usize) -> String {
    let frame = orcvs.render_frame();
    let grid = frame.grid();
    let mut out = Vec::new();
    for y in 0..rows {
        let mut row = Vec::new();
        for x in 0..cols {
            let p = grid.position(x, y).unwrap();
            let cell = frame.at(p);
            let ch = cell.content().map(|c| c.to_string()).unwrap_or_default();
            let paint = match cell.source_paint() {
                SourcePaint::Unclaimed => "o",
                SourcePaint::Function => "f",
                SourcePaint::Bang => "b",
                SourcePaint::Comment => "c",
                SourcePaint::Operand { token: Token::Number, state: OperandState::Invalid } => "n!",
                SourcePaint::Operand { token: Token::Note, state: OperandState::Invalid } => "N!",
                SourcePaint::Operand { token: Token::Number, state: OperandState::Pending } => "n?",
                SourcePaint::Operand { token: Token::Note, state: OperandState::Pending } => "N?",
                SourcePaint::Operand { token: Token::Number, .. } => "n",
                SourcePaint::Operand { token: Token::Note, .. } => "N",
                SourcePaint::Operand { .. } => "o",
            };
            let roles = cell.portal_roles();
            let r: String = [(PortalRole::Output, 'O'), (PortalRole::Input, 'I'), (PortalRole::Locked, 'L')]
                .iter().filter(|(role, _)| roles.contains(*role)).map(|(_, c)| *c).collect();
            let diag = frame.diagnostic_covers(p);
            row.push(format!("[{:?},{:?},{:?},{}]", ch, paint, r, diag));
        }
        out.push(format!("[{}]", row.join(",")));
    }
    format!("[{}]", out.join(",\n"))
}
