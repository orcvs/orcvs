//! `docs/functions.md` is held to the language: every Function in the table
//! has an entry, and every example ticks to the Source the entry shows.
//!
//! An example is a fenced block whose info string is `orcvs`, the Source
//! before any Tick, followed by one or more blocks whose info string is
//! `orcvs tick N`, the Source after Ticks `0` through `N` run on it. A `tick`
//! block may add `plays K`, the Play Commands and Network Sends Tick `N` sends, and
//! `diagnoses`, when Tick `N` diagnoses; without it, every Tick must run
//! without a diagnostic.

use lang::Function;
use orcvs::source::{Tick, file};

const REFERENCE: &str = include_str!("../../docs/functions.md");

/// One fenced `orcvs` block: its info string, its text, and the line its
/// fence opens on.
struct Block {
    info: String,
    text: String,
    line: usize,
}

/// The marker a line's fence is drawn with and its info string, when the line
/// is a fence: three or more backticks or tildes, indented or not.
fn fence(line: &str) -> Option<(char, &str)> {
    let line = line.trim_start();
    let marker = line.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let info = line.trim_start_matches(marker);
    (line.len() - info.len() >= 3).then_some((marker, info.trim()))
}

fn blocks() -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut open: Option<(char, Block)> = None;
    for (index, line) in REFERENCE.lines().enumerate() {
        match (&mut open, fence(line)) {
            (None, Some((marker, info))) => {
                open = Some((
                    marker,
                    Block {
                        info: info.to_owned(),
                        text: String::new(),
                        line: index + 1,
                    },
                ))
            }
            (Some((opened, _)), Some((marker, ""))) if *opened == marker => {
                let (_, block) = open.take().expect("an open block");
                if block.info.split_whitespace().next() == Some("orcvs") {
                    blocks.push(block);
                }
            }
            (Some((_, block)), _) => {
                block.text.push_str(line.trim_end());
                block.text.push('\n');
            }
            (None, None) => {}
        }
    }
    assert!(
        open.is_none(),
        "docs/functions.md ends inside a fenced block"
    );
    blocks
}

/// What a `tick` block's info string claims about the last Tick it runs.
struct Claim {
    tick: u64,
    plays: usize,
    diagnoses: bool,
}

fn claim(info: &str, line: usize) -> Option<Claim> {
    let words: Vec<&str> = info.split_whitespace().collect();
    let number = |word: Option<&&str>| {
        word.and_then(|word| word.parse().ok())
            .unwrap_or_else(|| panic!("line {line}: `{info}` needs a number after each keyword"))
    };
    let at = |keyword: &str| words.iter().position(|word| *word == keyword);
    let tick = at("tick")?;
    Some(Claim {
        tick: number(words.get(tick + 1)),
        plays: at("plays").map_or(0, |plays| number(words.get(plays + 1)) as usize),
        diagnoses: at("diagnoses").is_some(),
    })
}

#[test]
fn every_function_has_an_entry() {
    for function in Function::ALL {
        let heading = format!("### `{}`", function.cells());
        assert!(
            REFERENCE.lines().any(|line| line.starts_with(&heading)),
            "docs/functions.md has no entry headed {heading}"
        );
    }
}

/// Every `### ` heading in `text` outside a fenced block, with the line it
/// is on, counting from 1.
fn headings(text: &str) -> Vec<(usize, &str)> {
    let mut fenced = None;
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            match (fenced, fence(line)) {
                (None, Some((marker, _))) => fenced = Some(marker),
                (Some(opened), Some((marker, ""))) if opened == marker => fenced = None,
                _ => {}
            }
            fenced.is_none()
        })
        .filter_map(|(index, line)| line.strip_prefix("### ").map(|rest| (index + 1, rest)))
        .collect()
}

/// A fence may be indented, as it is inside a list item, or spelled with
/// tildes; a `### ` line inside either is the block's text, not a heading,
/// and the fence's close ends the block.
#[test]
fn a_heading_scan_skips_indented_and_tilde_fences() {
    let text = "\
- an item

  ```orcvs
### inside an indented fence
  ```

~~~
### inside a tilde fence
~~~

### `.+` Add
";
    assert_eq!(headings(text), vec![(11, "`.+` Add")]);
}

/// Every `### ` heading outside a fenced block is ``### `<spelling>` <name>``,
/// the name is the one its Function's row in the Function table carries, and
/// each Function has exactly one, so the two cannot drift.
#[test]
fn every_heading_names_its_function_as_the_table_does() {
    let mut headed = Vec::new();
    for (line, heading) in headings(REFERENCE) {
        let (spelling, name) = heading
            .strip_prefix('`')
            .and_then(|rest| rest.split_once("` "))
            .unwrap_or_else(|| {
                panic!("line {line}: `### {heading}` is not ### `<spelling>` <name>")
            });
        let function = Function::ALL
            .iter()
            .find(|function| function.cells().to_string() == spelling)
            .unwrap_or_else(|| panic!("line {line}: `{spelling}` is no Function in the table"));
        assert!(
            !headed.contains(function),
            "line {line}: `{spelling}` is headed twice"
        );
        headed.push(*function);
        assert_eq!(
            function.name(),
            name,
            "line {line}: the table names `{spelling}`"
        );
    }
    for function in Function::ALL {
        assert!(
            headed.contains(function),
            "docs/functions.md has no heading for `{}`",
            function.cells()
        );
    }
}

/// An entry that spells a signature names the operands its Function's Roles
/// name, in order.
#[test]
fn every_signature_names_its_functions_roles() {
    let lines: Vec<&str> = REFERENCE.lines().collect();
    let entries = headings(REFERENCE);
    let mut signatures = 0;
    for (index, (line, heading)) in entries.iter().enumerate() {
        let spelling = heading
            .strip_prefix('`')
            .and_then(|rest| rest.split_once('`'))
            .unwrap_or_else(|| panic!("line {line}: `### {heading}` names no spelling"))
            .0;
        let function = Function::ALL
            .iter()
            .find(|function| function.cells().to_string() == spelling)
            .unwrap_or_else(|| panic!("line {line}: `{spelling}` is no Function in the table"));
        let end = entries
            .get(index + 1)
            .map_or(lines.len(), |(next, _)| next - 1);
        let signature = lines[*line..end]
            .iter()
            .flat_map(|text| text.split('`').skip(1).step_by(2))
            .find_map(|span| span.strip_prefix(spelling)?.strip_prefix(' '));
        if let Some(signature) = signature {
            signatures += 1;
            let roles: Vec<&str> = function.roles().iter().map(|role| role.name).collect();
            assert_eq!(
                signature.split(' ').collect::<Vec<_>>(),
                roles,
                "line {line}: the table's Roles for `{spelling}`"
            );
        }
    }
    assert!(signatures > 0, "no entry spells a signature");
}

/// Each Function's summary in the table is one short line.
#[test]
fn every_summary_is_one_short_line() {
    for function in Function::ALL {
        let summary = function.summary();
        assert!(
            !summary.is_empty()
                && summary.len() <= 60
                && !summary.contains('\n')
                && summary.trim() == summary,
            "the summary for `{}` is not one short line: {summary:?}",
            function.cells()
        );
    }
}

#[test]
fn every_example_ticks_to_the_source_it_shows() {
    let mut before: Option<&Block> = None;
    let blocks = blocks();
    let mut checked = 0;
    for block in &blocks {
        let Some(claim) = claim(&block.info, block.line) else {
            before = Some(block);
            continue;
        };
        let start = before.unwrap_or_else(|| {
            panic!(
                "line {}: a `tick` block with no example before it",
                block.line
            )
        });
        let mut source = file::read(start.text.as_bytes())
            .unwrap_or_else(|refusal| panic!("line {}: {refusal}", start.line));
        for tick in 0..=claim.tick {
            let plan = source.execute(Tick::new(tick));
            let last = tick == claim.tick;
            if !(last && claim.diagnoses) {
                assert!(
                    plan.diagnostics.is_empty(),
                    "line {}: Tick {tick} diagnosed {:?}",
                    block.line,
                    plan.diagnostics
                );
            }
            if last {
                assert_eq!(
                    plan.play_commands.len() + plan.network_sends.len(),
                    claim.plays,
                    "line {}: Tick {tick} sent {:?} and {:?}",
                    block.line,
                    plan.play_commands,
                    plan.network_sends
                );
                assert_eq!(
                    claim.diagnoses,
                    !plan.diagnostics.is_empty(),
                    "line {}: the block says whether Tick {tick} diagnoses",
                    block.line
                );
            }
        }
        assert_eq!(
            file::write(&source),
            block.text,
            "line {}: the Source after Tick {}",
            block.line,
            claim.tick
        );
        checked += 1;
    }
    assert!(checked > 0, "docs/functions.md holds no checked example");
}
