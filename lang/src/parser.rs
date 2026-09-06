use crate::Atom;
use crate::Atoms;
use crate::Error;
use crate::Expression;
use crate::Function;
use crate::SyntaxError;
use crate::Token;
use crate::Tokens;
use crate::atom::to_atom_char;
use crate::to_atom_note;
use crate::to_atom_num;

#[derive(Debug)]
pub enum SourceAnalysis {
    Complete(Expression),
    Incomplete {
        expression: Expression,
        error: Error,
    },
    Invalid {
        expression: Expression,
        error: Error,
    },
}

impl SourceAnalysis {
    pub fn expression(&self) -> &Expression {
        match self {
            Self::Complete(expression)
            | Self::Incomplete { expression, .. }
            | Self::Invalid { expression, .. } => expression,
        }
    }

    pub fn into_expression(self) -> Expression {
        match self {
            Self::Complete(expression)
            | Self::Incomplete { expression, .. }
            | Self::Invalid { expression, .. } => expression,
        }
    }

    pub fn error(&self) -> Option<&Error> {
        match self {
            Self::Complete(_) => None,
            Self::Incomplete { error, .. } | Self::Invalid { error, .. } => Some(error),
        }
    }
}

#[derive(Debug)]
enum ParseStatus {
    Complete,
    Incomplete(Error),
    Invalid(Error),
}

impl ParseStatus {
    fn merge(self, next: Self) -> Self {
        match (self, next) {
            (invalid @ Self::Invalid(_), _) | (_, invalid @ Self::Invalid(_)) => invalid,
            (incomplete @ Self::Incomplete(_), _) | (_, incomplete @ Self::Incomplete(_)) => {
                incomplete
            }
            (Self::Complete, Self::Complete) => Self::Complete,
        }
    }
}

pub struct Parser<'a> {
    expression: Expression,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn from(source: &'a mut str) -> Self {
        Self {
            expression: Expression::new(),
            source,
        }
    }

    /// Strictly parses one complete Expression.
    #[inline]
    pub fn try_parse(mut self) -> Result<Atoms, Error> {
        match self.take_language_unit()? {
            ParseStatus::Complete if self.source.is_empty() => Ok(self
                .expression
                .take_atoms()
                .expect("strict parsing contains only values")),
            ParseStatus::Complete => {
                Err(SyntaxError::UnexpectedTrailingContent(self.source.to_string()).into())
            }
            ParseStatus::Incomplete(error) | ParseStatus::Invalid(error) => Err(error),
        }
    }

    /// Permissively analyzes Source while preserving every complete entry.
    #[inline]
    pub fn analyze(mut self) -> Result<SourceAnalysis, Error> {
        let mut status = self.take_language_unit()?;
        if matches!(status, ParseStatus::Complete) && !self.source.is_empty() {
            let trailing = self.source.to_string();
            status = ParseStatus::Invalid(SyntaxError::UnexpectedTrailingContent(trailing).into());
        }

        let expression = self.expression;
        let analysis = match status {
            ParseStatus::Complete => SourceAnalysis::Complete(expression),
            ParseStatus::Incomplete(error) => SourceAnalysis::Incomplete { expression, error },
            ParseStatus::Invalid(error) => SourceAnalysis::Invalid { expression, error },
        };
        Ok(analysis)
    }

    ///
    /// A Language Unit may be a Function or a standalone Atom.
    #[inline(always)]
    fn take_language_unit(&mut self) -> Result<ParseStatus, Error> {
        match self.next_token(2) {
            Some(t) => {
                match t {
                    "**" => {
                        self.add(Token::Bang, Atom::Bang)?;
                        return Ok(ParseStatus::Complete);
                    }
                    _ if let Ok(activation) = crate::Activation::try_from(t) => {
                        self.add(Token::Activation, Atom::Activation(activation))?;
                        return Ok(ParseStatus::Complete);
                    }
                    _ => {}
                }
                let result = Function::try_from(t);

                let tokens = match result {
                    Ok(f) => {
                        self.add(Token::Function, Atom::from(f))?;
                        Tokens::from(&f)
                    }
                    Err(error) => {
                        self.expression.add_invalid(Token::Function)?;
                        return Ok(ParseStatus::Invalid(error));
                    }
                };

                let mut status = ParseStatus::Complete;
                for t in tokens {
                    if self.is_function_next() {
                        status = status.merge(self.take_language_unit()?);
                    } else {
                        match self.take_token(&t) {
                            Ok(Some(atom)) => self.add(t, atom)?,
                            Ok(None) => {
                                status = status.merge(ParseStatus::Incomplete(
                                    SyntaxError::ExpectedToken.into(),
                                ));
                            }
                            Err(error) => {
                                self.expression.add_invalid(t)?;
                                return Ok(ParseStatus::Invalid(error));
                            }
                        }
                    }
                }
                Ok(status)
            }
            None => {
                self.expression.add_incomplete(Token::Char)?;
                Ok(ParseStatus::Incomplete(
                    SyntaxError::ExpectedFunction.into(),
                ))
            }
        }
    }

    #[inline(always)]
    fn take_token(&mut self, token: &Token) -> Result<Option<Atom>, Error> {
        let t = self.next_token(token.len());
        let atom = match t {
            Some(s) => match token {
                Token::Note => to_atom_note(s)?,
                Token::Number => to_atom_num(s)?,
                Token::Char => to_atom_char(s)?,
                Token::Activation | Token::Bang | Token::Function => unreachable!(),
            },
            None => {
                self.expression.add_incomplete(*token)?;
                return Ok(None);
            }
        };

        Ok(Some(atom))
    }

    #[inline(always)]
    fn next_token(&mut self, count: usize) -> Option<&'a str> {
        match self.source.split_at_checked(count) {
            Some((next_token, rest)) => {
                self.source = rest;
                Some(next_token)
            }
            None => None,
        }
    }

    #[inline(always)]
    fn is_function_next(&self) -> bool {
        let peek = self.peek_next();
        is_function(peek)
    }

    #[inline(always)]
    fn peek_next(&self) -> Option<&'a str> {
        // `split_at_checked` rather than `split_at`, matching `next_token`. Every
        // Cell the Source layer admits is single-byte, so byte 2 is a character
        // boundary for any Source that reaches here through a Grid; a `&mut str`
        // handed straight to `Parser::from` carries no such guarantee, and the
        // unchecked split panicked on it rather than declining to peek.
        match self.source.split_at_checked(2) {
            Some((next_token, _)) => Some(next_token),
            None => None,
        }
    }

    #[inline(always)]
    fn add(&mut self, t: Token, a: Atom) -> Result<(), Error> {
        self.expression.add(t, a)?;
        Ok(())
    }
}

#[inline(always)]
fn is_function(s: Option<&str>) -> bool {
    // s.map_or(false, |t| Function::try_from(t).is_ok())
    // s.filter(|t| Function::try_from(*t).is_ok()).is_some()
    if let Some(t) = s {
        Function::try_from(t).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod test {

    use crate::{
        Atom, Atoms, Error, Function, SourceAnalysis, SyntaxError, Token, TypeError,
        parser::Parser, trace,
    };
    use arrayvec::ArrayVec;

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    #[test]
    fn source_analysis_represents_complete_incomplete_and_invalid_source() {
        assert!(matches!(
            Parser::from(&mut ".+0102".to_owned()).analyze().unwrap(),
            SourceAnalysis::Complete(_)
        ));
        assert!(matches!(
            Parser::from(&mut ".+01".to_owned()).analyze().unwrap(),
            SourceAnalysis::Incomplete { .. }
        ));
        assert!(matches!(
            Parser::from(&mut ".+01XY".to_owned()).analyze().unwrap(),
            SourceAnalysis::Invalid { .. }
        ));
    }

    #[test]
    fn source_analysis_marks_trailing_content_invalid_without_losing_complete_entries() {
        let analysis = Parser::from(&mut ".+0102Z".to_owned()).analyze().unwrap();

        match analysis {
            SourceAnalysis::Invalid { expression, error } => {
                assert_eq!(
                    expression.atoms().unwrap().as_slice(),
                    &[
                        Atom::Function(Function::Add),
                        Atom::Number(1),
                        Atom::Number(2),
                    ]
                );
                assert!(matches!(
                    error,
                    Error::Syntax(SyntaxError::UnexpectedTrailingContent(ref trailing))
                        if trailing == "Z"
                ));
            }
            other => panic!("expected invalid analysis, found {other:?}"),
        }
    }

    #[test]
    fn every_real_function_parses_and_renders_from_its_canonical_spelling() {
        for function in Function::ALL {
            let spelling = function.spelling();

            assert_eq!(Function::try_from(spelling).unwrap(), *function);
            assert_eq!(function.to_string(), spelling);
        }
    }

    fn parse(exp: &mut str) -> Result<Vec<Atom>, Error> {
        let parser = Parser::from(exp);
        Ok(parser
            .analyze()?
            .into_expression()
            .take_atoms()
            .unwrap_or_default()
            .into_iter()
            .collect())
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let mut s = String::from(".+");
        let parsed = parse(&mut s).unwrap();

        assert!(parsed.is_empty());

        let mut s = String::from("+");
        let parsed = parse(&mut s).unwrap();

        let stack = vec![];
        assert_eq!(parsed, stack);

        let mut s = String::from("..");
        let parsed = parse(&mut s).unwrap();
        assert!(parsed.is_empty());

        let mut s = String::from("ABC");
        let parsed = parse(&mut s).unwrap();
        assert!(parsed.is_empty());

        let mut s = String::from("A           ");
        let parsed = parse(&mut s).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn permissive_parse_keeps_non_values_out_of_runtime_atoms() {
        let incomplete = Parser::from(".+01".to_owned().as_mut_str())
            .analyze()
            .unwrap()
            .into_expression();
        assert_eq!(
            incomplete.tokens().collect::<Vec<_>>(),
            vec![Token::Function, Token::Number, Token::Number]
        );
        assert_eq!(
            incomplete.entries().collect::<Vec<_>>(),
            vec![
                (Token::Function, Atom::Function(Function::Add)),
                (Token::Number, Atom::Number(1))
            ]
        );
        assert!(incomplete.atoms().is_none());

        let invalid = Parser::from(".+01XY".to_owned().as_mut_str())
            .analyze()
            .unwrap()
            .into_expression();
        assert_eq!(
            invalid.tokens().collect::<Vec<_>>(),
            vec![Token::Function, Token::Number, Token::Number]
        );
        assert_eq!(
            invalid.entries().collect::<Vec<_>>(),
            vec![
                (Token::Function, Atom::Function(Function::Add)),
                (Token::Number, Atom::Number(1))
            ]
        );
        assert!(invalid.atoms().is_none());
    }

    #[test]
    fn source_analysis_preserves_invalid_nested_operand_span() {
        let source = ".+.-01XY02";
        let expression = Parser::from(&mut source.to_owned())
            .analyze()
            .unwrap()
            .into_expression();

        assert_eq!(
            expression.tokens().map(|token| token.len()).sum::<usize>(),
            source.len()
        );
    }

    #[test]
    fn test_try_parse_with_invalid() {
        trace();

        let mut s = String::from("+");
        let result = try_parse(&mut s);

        let error = result.unwrap_err();
        assert!(matches!(
            error,
            Error::Syntax(SyntaxError::ExpectedFunction)
        ));
    }

    #[test]
    fn test_with_bad_syntax() {
        trace();

        let mut s = String::from(".+01XY");
        let result = try_parse(&mut s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    #[test]
    fn test_parse_nested_arithmetic_expression() {
        trace();

        // Add(Add(Multiply(02, 03), 04), 05) — three levels of prefix nesting,
        // replacing the identity-wrapped cases retired by ADR 0015.
        let mut s = String::from(".+.+.x02030405");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![
            Atom::Function(Function::Add),
            Atom::Function(Function::Add),
            Atom::Function(Function::Multiply),
            Atom::Number(2),
            Atom::Number(3),
            Atom::Number(4),
            Atom::Number(5),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_function_in_either_operand_slot() {
        trace();

        // A nested Function is valid in the right operand slot as well as the
        // left, so the recursive descent must not assume left-only nesting.
        let mut s = String::from(".-0A./0402");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![
            Atom::Function(Function::Subtract),
            Atom::Number(10),
            Atom::Function(Function::Divide),
            Atom::Number(4),
            Atom::Number(2),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }

    #[test]
    fn retired_arithmetic_spellings_do_not_parse_as_functions() {
        for spelling in ["++", "--", "//"] {
            let error = try_parse(&mut spelling.to_owned()).unwrap_err();
            assert!(
                matches!(error, Error::Syntax(SyntaxError::UnknownFunction(ref found)) if found == spelling),
                "{spelling} produced {error:?}"
            );
        }
    }

    #[test]
    fn numeric_conversion_spellings_parse_without_language_unit_collisions() {
        assert_eq!(
            try_parse(&mut ".vC4".to_owned()).unwrap().as_slice(),
            &[
                Atom::Function(Function::ConvertToNumber),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ]
        );
        assert_eq!(
            try_parse(&mut ".^3C".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(60)]
        );
    }

    #[test]
    fn comparison_spellings_parse_without_activation_collisions() {
        // `.<` and `.>` share their second Cell with the Self-Banging Functions
        // `<<` and `>>`, and the parser tests an Activation before a Function.
        // These pin that the shared Cell alone never wins.
        for (source, function) in [
            (".|0A05", Function::AbsoluteDifference),
            (".%0A05", Function::Modulo),
            (".<0A05", Function::Minimum),
            (".>0A05", Function::Maximum),
            (".=0A05", Function::Equality),
        ] {
            assert_eq!(
                try_parse(&mut source.to_owned()).unwrap().as_slice(),
                &[
                    Atom::Function(function),
                    Atom::Number(0x0A),
                    Atom::Number(0x05),
                ],
                "failed to parse {source:?}",
            );
        }
    }

    #[test]
    fn every_note_source_encoding_parses_in_context() {
        for value in 0x00..=0x7F {
            let note_value = crate::Note::try_from(value).unwrap();
            let note = Atom::Note(note_value).to_string();
            let mut source = format!(".v{note}");
            assert_eq!(
                try_parse(&mut source).unwrap().as_slice(),
                &[
                    Atom::Function(Function::ConvertToNumber),
                    Atom::Note(note_value),
                ],
                "failed to parse Note({value}) from {note:?}",
            );
        }
    }

    #[test]
    fn conversion_literal_operands_are_monomorphic() {
        assert!(matches!(
            try_parse(&mut ".v3C".to_owned()),
            Err(Error::Type(TypeError::Note(_)))
        ));
        assert!(matches!(
            try_parse(&mut ".^G9".to_owned()),
            Err(Error::Type(TypeError::Number(_)))
        ));

        // An overlapping spelling receives the type fixed by the Function's
        // literal operand slot, rather than choosing a type from its spelling.
        assert_eq!(
            try_parse(&mut ".^C4".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(0xC4)]
        );
    }

    #[test]
    fn conversion_operands_without_a_note_spelling_stay_numbers() {
        // The whole `00`-`7F` domain `.^` accepts starts with a hexadecimal
        // digit rather than a pitch letter, so no in-range operand is ambiguous
        // and the `80`-`FF` diagnosis stays reachable from Source.
        assert_eq!(
            try_parse(&mut ".^7F".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(0x7F)]
        );
        assert_eq!(
            try_parse(&mut ".^80".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(0x80)]
        );
        assert_eq!(
            try_parse(&mut ".^FA".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(0xFA)]
        );
    }

    #[test]
    fn bang_and_activations_parse_as_complete_language_units() {
        assert_eq!(
            try_parse(&mut "**".to_owned()).unwrap().as_slice(),
            &[Atom::Bang]
        );
        for (source, direction) in [
            ("^^", crate::Activation::North),
            ("vv", crate::Activation::South),
            ("<<", crate::Activation::West),
            (">>", crate::Activation::East),
        ] {
            assert_eq!(
                try_parse(&mut source.to_owned()).unwrap().as_slice(),
                &[Atom::Activation(direction)]
            );
        }
    }

    #[test]
    fn test_parse_play_function() {
        trace();

        let mut s = String::from("!>010AC4");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![
            Atom::Function(Function::RawPlay),
            Atom::Number(1),
            Atom::Number(10),
            Atom::Note(crate::Note::try_from(60).unwrap()),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }

    /// Every Atom of the domain a `Token` names, in Source order.
    ///
    /// Both domains are small enough to enumerate — the 256 Numbers, and the
    /// 128 MIDI Notes `C/` through `G9` — so the round trip below sweeps them
    /// rather than sampling them.
    fn every_atom_of(token: Token) -> Vec<Atom> {
        match token {
            Token::Number => (0..=u8::MAX).map(Atom::Number).collect(),
            Token::Note => (0x00..=0x7F)
                .map(|value| Atom::Note(crate::Note::try_from(value).expect("a MIDI Note")))
                .collect(),
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    /// Source text for the lowest value of the domain a `Token` names: what the
    /// operand positions that are not being swept are held at.
    fn baseline(token: Token) -> String {
        every_atom_of(token)[0].to_string()
    }

    #[test]
    fn every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it() {
        // A standalone Language Unit is a whole Expression, so it renders and
        // parses back with no Function to type it.
        for atom in [
            Atom::Bang,
            Atom::Activation(crate::Activation::North),
            Atom::Activation(crate::Activation::South),
            Atom::Activation(crate::Activation::West),
            Atom::Activation(crate::Activation::East),
        ] {
            let mut source = atom.to_string();
            assert_eq!(try_parse(&mut source).unwrap().as_slice(), &[atom]);
        }

        // Every other Atom the parser yields is an Operand Literal, and ADR
        // 0021 gives it the type of the slot that consumes it rather than a
        // type of its own. The round trip is therefore contextual: `C4` is the
        // Note 60 in `.v`'s Note slot and the Number `C4` in `.^`'s Number
        // slot, and the sweep covers both readings because it covers every
        // value of every declared operand domain in every slot that declares
        // it. `Atom::Char` and `Atom::Empty` are absent because no signature
        // declares them, so no Source spells one.
        for function in Function::ALL.iter().copied() {
            let signature = function.signature();
            for (slot, token) in signature.iter().copied().enumerate() {
                for atom in every_atom_of(token) {
                    let operands: String = signature
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(position, declared)| {
                            if position == slot {
                                atom.to_string()
                            } else {
                                baseline(declared)
                            }
                        })
                        .collect();
                    let source = format!("{function}{operands}");
                    let mut spelled = source.clone();
                    let parsed = try_parse(&mut spelled)
                        .unwrap_or_else(|error| panic!("{source:?} did not parse: {error}"));

                    assert_eq!(parsed[0], Atom::Function(function));
                    assert_eq!(parsed[slot + 1], atom, "{source:?}");
                    // Rendering the whole Expression back reproduces the very
                    // Cells it was parsed from, which is the round trip and the
                    // absence of trailing content in one statement.
                    assert_eq!(
                        parsed.iter().map(Atom::to_string).collect::<String>(),
                        source
                    );
                }
            }
        }
    }
}

///
/// Parser totality over printable ASCII.
///
/// `AGENTS.md` obliges a change at the parser boundary to bring "boundary or
/// property tests", and the parser is the widest input surface in the
/// workspace because every keystroke reaches it. Strict parsing and permissive
/// analysis both have to answer rather than panic for anything a Cell can
/// hold, and they have to keep their contracts apart while doing it: strict
/// parsing yields only a whole Expression of complete evaluable entries, and
/// analysis yields the complete entries it recognized plus an explicit report
/// of what it could not.
///
/// The generators produce raw Source text rather than valid Expressions. One
/// that only spelled Expressions the parser accepts would test itself and
/// slowly become a second implementation of the grammar, so the space that
/// ends a run, the incomplete `#`, and the `##` Comment introducer are drawn as
/// text like everything else — and
/// `generated_source_covers_the_space_the_incomplete_hash_and_the_comment_introducer`
/// pins that they are actually reached rather than merely reachable.
///
/// The `Atom` round trip is `mod test`'s
/// `every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it`
/// rather than a property: the two operand domains hold 384 values between
/// them, which is small enough to enumerate and too small to be worth
/// sampling.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {

    use crate::{
        Activation, Atom, EXP_LEN, Error, Function, SourceAnalysis, SyntaxError, Token,
        midi_number_to_note, parser::Parser,
    };
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    use std::cell::Cell;

    /// How many fragments a generated Source is assembled from at most. A
    /// fragment is one or two Cells, so a long case runs well past the Cells
    /// one Language Unit occupies and the parser has a run to abandon rather
    /// than a single unit to read.
    ///
    /// This ceiling is deliberately not set from `EXP_LEN`. Capacity bounds
    /// records rather than Cells, and analysis stops recording at the first
    /// Token it cannot read, so raw text answers one invalid record and ends
    /// however long it runs. Only a chain of whole Functions accumulates
    /// records at all, and drawing thirty-two of those in a row has no
    /// meaningful probability. The capacity bound is therefore reached by the
    /// Expression `an_expression_that_outruns_the_parser_capacity_...` spells
    /// by hand, not from here.
    const FRAGMENTS: usize = 24;

    /// The Atoms that are a whole Language Unit on their own: the Bang and the
    /// four Activations.
    const STANDALONE: &[Atom] = &[
        Atom::Bang,
        Atom::Activation(Activation::North),
        Atom::Activation(Activation::South),
        Atom::Activation(Activation::West),
        Atom::Activation(Activation::East),
    ];

    /// Source text for one Operand Literal of the type its position declares.
    ///
    /// ADR 0021 makes an Operand Literal's type the consuming Function's rather
    /// than the Source's, so a literal is spelled against the `Token` the slot
    /// declares. The same two Cells spell a Number in one slot and a Note in
    /// another.
    fn literal_source(token: Token) -> BoxedStrategy<String> {
        match token {
            Token::Number => any::<u8>()
                .prop_map(|number| format!("{number:02X}"))
                .boxed(),
            Token::Note => (0x00u8..=0x7F)
                .prop_map(|note| midi_number_to_note(note).expect("a MIDI Note"))
                .boxed(),
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    /// One Function spelled with a literal in each operand position its
    /// signature declares: the shape strict parsing accepts whole.
    fn complete_expression() -> BoxedStrategy<String> {
        select(Function::ALL)
            .prop_flat_map(|function| {
                let operands: Vec<BoxedStrategy<String>> = function
                    .signature()
                    .iter()
                    .map(|token| literal_source(*token))
                    .collect();
                (Just(function), operands)
            })
            .prop_map(|(function, operands)| format!("{function}{}", operands.concat()))
            .boxed()
    }

    /// One piece of generated Source text.
    ///
    /// Most of the weight is one arbitrary printable character, which is what
    /// keeps the whole range in reach. The rest are the pieces the language
    /// gives meaning to, so that a case is more often a near miss than noise:
    /// the space and the `##` Comment introducer that end a run, the `#` that
    /// is incomplete Source rather than a Comment, a Function spelling with no
    /// operands after it, a standalone Atom, and an Operand Literal outside any
    /// slot. They are concatenated in whatever order they are drawn, so the
    /// result is raw text rather than a grammar.
    fn fragment() -> BoxedStrategy<String> {
        prop_oneof![
            8 => proptest::char::range(' ', '~').prop_map(String::from),
            2 => Just(" ".to_owned()),
            2 => Just("#".to_owned()),
            2 => Just("##".to_owned()),
            2 => select(Function::ALL).prop_map(|function| function.to_string()),
            1 => select(STANDALONE).prop_map(|atom| atom.to_string()),
            2 => prop_oneof![literal_source(Token::Number), literal_source(Token::Note)],
        ]
        .boxed()
    }

    /// Source text for one Expression's worth of Cells.
    ///
    /// The unbiased branch is the printable range itself, drawn straight from
    /// the character class. The fragment branch is the same range with the
    /// language's own pieces mixed in, biased short so that the whole of a
    /// short case is one Language Unit's worth of Cells rather than a run the
    /// parser abandons in its first two.
    ///
    /// The third branch is the minority one, and it is here because half of
    /// every property below is conditional on a parse succeeding. Raw text
    /// almost never spells a whole Expression, so without a branch that does,
    /// the accepting arms would pass by never running and the suite would
    /// state only that nothing panics. It stays a minority: a generator made
    /// of valid Expressions tests itself rather than the parser, which is why
    /// five parts in six here are raw.
    fn generated_source() -> BoxedStrategy<String> {
        prop_oneof![
            2 => "[ -~]{0,48}",
            3 => prop_oneof![3 => 1usize..6, 1 => 6..FRAGMENTS]
                .prop_flat_map(|count| prop::collection::vec(fragment(), count))
                .prop_map(|fragments| fragments.concat()),
            1 => complete_expression(),
        ]
        .boxed()
    }

    /// Every Atom of an Expression, rendered back to Source text.
    fn rendered(atoms: impl IntoIterator<Item = Atom>) -> String {
        atoms.into_iter().map(|atom| atom.to_string()).collect()
    }

    /// Whether an entry's Atom is the kind its Token names.
    fn entry_agrees(token: Token, atom: Atom) -> bool {
        matches!(
            (token, atom),
            (Token::Function, Atom::Function(_))
                | (Token::Number, Atom::Number(_))
                | (Token::Note, Atom::Note(_))
                | (Token::Bang, Atom::Bang)
                | (Token::Activation, Atom::Activation(_))
                | (Token::Char, Atom::Char(_))
        )
    }

    proptest! {
        ///
        /// Strict parsing is total over printable ASCII: it answers with a
        /// whole Expression of complete evaluable entries, or with one of the
        /// crate's typed errors. A panic inside `try_parse` fails the case,
        /// which is the first half of the property; the arms state the second.
        ///
        /// Success is checked by rendering the Atoms back. Every Atom strict
        /// parsing yields occupies exactly the Cells it was read from, so a
        /// rendering equal to the Source is the whole of "no trailing content
        /// and nothing truncated" — a parse that stopped early or dropped an
        /// Atom produces a shorter string, and one that invented an Atom
        /// produces a longer one.
        ///
        #[test]
        fn strict_parsing_of_printable_ascii_yields_a_whole_expression_or_a_typed_error(
            source in generated_source(),
        ) {
            let spelled = source.clone();
            let mut source = source;

            match Parser::from(&mut source).try_parse() {
                Ok(atoms) => {
                    prop_assert!(atoms.len() <= EXP_LEN);
                    prop_assert!(
                        !atoms.iter().any(|atom| matches!(atom, Atom::Empty | Atom::Char(_))),
                        "{spelled:?} parsed to a value no signature declares: {atoms:?}",
                    );
                    prop_assert_eq!(rendered(atoms), spelled.as_str());
                }
                // Reading two Cells is the only thing the parser does, so the
                // families it can diagnose are the shape of those Cells and the
                // type the slot consuming them declares. An error from any
                // other family would be one raised on a value's behalf, and
                // strict parsing never holds a value.
                Err(error) => prop_assert!(
                    matches!(error, Error::Syntax(_) | Error::Type(_)),
                    "{spelled:?} raised {error:?}",
                ),
            }
        }

        ///
        /// Permissive analysis is total over the same input and keeps the other
        /// contract: it preserves every complete entry it recognized, reports
        /// incomplete or invalid Source as an explicit error, and hands no
        /// runtime Atoms to a caller when it has not read a whole Expression.
        /// A placeholder standing in for a Cell that was never written is what
        /// the last of those rules out.
        ///
        #[test]
        fn permissive_analysis_of_printable_ascii_reports_what_it_could_not_read(
            source in generated_source(),
        ) {
            let spelled = source.clone();
            let mut source = source;

            let analysis = match Parser::from(&mut source).analyze() {
                Ok(analysis) => analysis,
                // Analysis diagnoses the Source it was handed and returns the
                // Expression it built from it, so the one thing it can fail at
                // is having nowhere left to record what it read.
                Err(error) => {
                    prop_assert!(
                        matches!(
                            error,
                            Error::Syntax(SyntaxError::ExpressionTooLong { capacity }) if capacity == EXP_LEN
                        ),
                        "{spelled:?} failed analysis with {error:?}",
                    );
                    return Ok(());
                }
            };

            let expression = analysis.expression();
            let entries: Vec<(Token, Atom)> = expression.entries().collect();
            for (token, atom) in entries.iter().copied() {
                prop_assert!(
                    entry_agrees(token, atom),
                    "{spelled:?} paired {token:?} with {atom:?}",
                );
                prop_assert_eq!(atom.to_string().len(), token.len());
            }

            // No value ever stands in for a Token the parser could not read.
            // An Expression answers with runtime Atoms only when every record
            // it holds is a complete entry, so an incomplete or invalid Token
            // withholds the whole Expression rather than contributing a
            // placeholder to it.
            prop_assert_eq!(
                expression.atoms().is_some(),
                entries.len() == expression.len(),
                "{:?}",
                spelled,
            );

            match &analysis {
                SourceAnalysis::Complete(_) => {
                    prop_assert!(analysis.error().is_none());
                    let atoms = expression
                        .atoms()
                        .expect("a complete analysis holds only complete entries");
                    prop_assert_eq!(rendered(atoms), spelled.as_str());
                }
                SourceAnalysis::Incomplete { .. } | SourceAnalysis::Invalid { .. } => {
                    prop_assert!(analysis.error().is_some());
                    // Trailing content is the one shape that is not complete
                    // and still holds nothing but complete entries: the Cells
                    // before the trailing run were read as a whole Expression,
                    // and the run after them is reported rather than parsed.
                    // Every other way of failing records the Token it could
                    // not read, which is what withholds the Atoms above.
                    prop_assert!(
                        expression.atoms().is_none()
                            || matches!(
                                analysis.error(),
                                Some(Error::Syntax(SyntaxError::UnexpectedTrailingContent(_)))
                            ),
                        "{spelled:?} produced runtime Atoms for {:?}",
                        analysis.error(),
                    );
                }
            }

            // Trailing content is the one invalid shape whose complete entries
            // account for the whole Source: everything before the trailing run
            // was read as a whole Expression, so preserving it means the
            // rendering and the trailing text spell the Cells that were handed
            // in.
            if let Some(Error::Syntax(SyntaxError::UnexpectedTrailingContent(trailing))) =
                analysis.error()
            {
                let preserved = rendered(entries.iter().map(|(_, atom)| *atom));
                prop_assert_eq!(format!("{preserved}{trailing}"), spelled.as_str());
            }
        }

        ///
        /// The two contracts agree about exactly one thing: strict parsing
        /// accepts the Source analysis calls complete, and no other. Analysis
        /// is the permissive path, so what separates them is that it also
        /// answers for the rest — not that it reads a different language.
        ///
        #[test]
        fn strict_parsing_accepts_exactly_the_source_analysis_calls_complete(
            source in generated_source(),
        ) {
            let mut strict = source.clone();
            let mut permissive = source.clone();

            let parsed = Parser::from(&mut strict).try_parse();
            let analysis = Parser::from(&mut permissive).analyze();

            let complete = matches!(analysis, Ok(SourceAnalysis::Complete(_)));
            prop_assert_eq!(parsed.is_ok(), complete, "{:?}", source);

            if let (Ok(atoms), Ok(analysis)) = (parsed, analysis) {
                prop_assert_eq!(
                    Some(atoms),
                    analysis.into_expression().take_atoms(),
                    "{:?}",
                    source,
                );
            }
        }

        ///
        /// An Expression that spells more Atoms than one can hold is refused
        /// with `ExpressionTooLong`, and one that fits is parsed whole.
        ///
        /// What the capacity bounds is Atoms rather than Cells: a chain of
        /// `depth` Additions over `depth + 1` Numbers spells `2 * depth + 1`
        /// Atoms across `4 * depth + 2` Cells, so from depth 8 the Source is
        /// already longer than `EXP_LEN` Cells and still parses. Both halves
        /// are asserted here because the failure the bound exists to prevent is
        /// a truncation, which would look exactly like the accepted half with
        /// fewer Atoms in it.
        ///
        /// The chain starts at one Function rather than none: a standalone
        /// Operand Literal has no Function to type it and is invalid Source,
        /// so depth zero would be a case about ADR 0021 rather than about the
        /// capacity.
        ///
        #[test]
        fn an_expression_that_outruns_the_parser_capacity_is_refused_rather_than_truncated(
            depth in 1usize..24,
        ) {
            let spelled = ".+".repeat(depth) + &"00".repeat(depth + 1);
            let atoms_spelled = 2 * depth + 1;
            let mut source = spelled.clone();

            let parsed = Parser::from(&mut source).try_parse();

            if atoms_spelled <= EXP_LEN {
                let atoms = parsed.map_err(|error| {
                    TestCaseError::fail(format!("{spelled:?} was refused with {error:?}"))
                })?;
                prop_assert_eq!(atoms.len(), atoms_spelled);
                prop_assert_eq!(rendered(atoms), spelled.as_str());
            } else {
                prop_assert!(
                    matches!(
                        parsed,
                        Err(Error::Syntax(SyntaxError::ExpressionTooLong { capacity }))
                            if capacity == EXP_LEN
                    ),
                    "{spelled:?} answered {parsed:?} rather than refusing {atoms_spelled} Atoms",
                );

                // The permissive reading is bounded by the same capacity, and
                // this is the only place that says so. Analysis stops
                // recording at the first Token it cannot read, so raw Source
                // answers one invalid record however long it runs and the
                // generated properties cannot reach this bound at all. A
                // hand-spelled chain is what reaches it.
                let mut source = spelled.clone();
                let analysis = Parser::from(&mut source).analyze();
                prop_assert!(
                    matches!(
                        analysis,
                        Err(Error::Syntax(SyntaxError::ExpressionTooLong { capacity }))
                            if capacity == EXP_LEN
                    ),
                    "{spelled:?} was analysed as {analysis:?} rather than refused",
                );
            }
        }
    }

    ///
    /// Source that is not ASCII declines to parse rather than panicking.
    ///
    /// Every Cell a Grid admits is a single byte, so the Source layer never
    /// hands this text to the parser. `Parser::from` takes any `&mut str`
    /// though, and the totality this suite states is a claim about the
    /// parser rather than about its callers, so the one input class the
    /// generator cannot draw is pinned here by hand: a multi-byte character
    /// straddling the two-Cell peek used to split a `char` down the middle.
    ///
    #[test]
    fn source_that_is_not_ascii_is_refused_rather_than_panicking() {
        for spelled in [".+aé", "é", "aé", "é.+", "..éé"] {
            let mut source = String::from(spelled);
            let parsed = Parser::from(&mut source).try_parse();
            assert!(parsed.is_err(), "{spelled:?} parsed as {parsed:?}");

            let mut source = String::from(spelled);
            // Analysis is the permissive reading and answers rather than
            // failing, so the claim here is only that it returns at all.
            let _ = Parser::from(&mut source).analyze();
        }
    }

    ///
    /// The generator reaches the three pieces of Source the language treats
    /// specially — the space that ends a run, the `#` that is incomplete
    /// Source rather than a Comment, and the `##` Comment introducer — and it
    /// reaches Source strict parsing accepts.
    ///
    /// A property is only as good as what its generator produces, and none of
    /// the properties above can tell an input it never saw from one it saw and
    /// handled. Driving the runner directly is what lets the draws be counted
    /// across cases; the count is asserted afterwards, where `proptest!` would
    /// have had nowhere to put it.
    ///
    /// The case count is pinned rather than taken from `PROPTEST_CASES`,
    /// because this claim is about the generator rather than about the parser:
    /// it draws Source and reads it as text, so it costs nothing at either
    /// verification tier and has no reason to weaken at the cheaper one.
    ///
    #[test]
    fn generated_source_covers_the_space_the_incomplete_hash_and_the_comment_introducer() {
        let config = Config {
            cases: 256,
            source_file: Some(file!()),
            ..Config::default()
        };
        let space = Cell::new(0usize);
        let incomplete = Cell::new(0usize);
        let comment = Cell::new(0usize);
        let complete = Cell::new(0usize);

        TestRunner::new(config)
            .run(&generated_source(), |source| {
                if source.contains(' ') {
                    space.set(space.get() + 1);
                }
                if source.contains("##") {
                    comment.set(comment.get() + 1);
                }
                // A `#` with no `#` beside it: incomplete Source rather than
                // the introducer, which is the distinction CONTEXT.md draws.
                let bytes = source.as_bytes();
                if bytes.iter().enumerate().any(|(index, byte)| {
                    *byte == b'#'
                        && bytes.get(index + 1) != Some(&b'#')
                        && (index == 0 || bytes[index - 1] != b'#')
                }) {
                    incomplete.set(incomplete.get() + 1);
                }
                let mut spelled = source.clone();
                if Parser::from(&mut spelled).try_parse().is_ok() {
                    complete.set(complete.get() + 1);
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{error}"));

        assert!(space.get() > 0, "no generated Source held a space");
        assert!(
            incomplete.get() > 0,
            "no generated Source held an incomplete `#`",
        );
        assert!(
            comment.get() > 0,
            "no generated Source held the `##` Comment introducer",
        );
        // And the accepting half of every property above has to be reached by
        // something, or those properties pass by never running.
        assert!(
            complete.get() > 0,
            "no generated Source spelled an Expression strict parsing accepts",
        );
    }
}
