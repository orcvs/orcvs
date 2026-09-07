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

///
/// What was read from one Source, and how much of it that took.
///
/// An Expression the Parser reads has a fixed width, set by its root
/// Function's arity, so the Source beyond `consumed` is the next Expression's
/// to read and not evidence that this one is wrong. Reporting the boundary
/// rather than a verdict about the whole Source is what lets a caller resume
/// after it, which is the partition ADR 0018 describes.
///
/// `complete` takes its width from a caller that assembled the Expression
/// without the Parser, so that width answers to the caller's partition rather
/// than to arity and can reach past where a root Function would have ended.
/// The arity claim scopes to the Parser, not to every `SourceAnalysis`.
///
#[derive(Debug)]
pub struct SourceAnalysis {
    expression: Expression,
    status: AnalysisStatus,
    consumed: usize,
}

impl SourceAnalysis {
    ///
    /// A complete Expression assembled without the Parser, spanning `consumed`
    /// bytes of Source.
    ///
    pub fn complete(expression: Expression, consumed: usize) -> Self {
        // The caller owes the width, because it assembled the Expression
        // without reading the Source. Only the lower bound is checkable here,
        // and it is the one a caller resuming from `consumed` depends on — so
        // it is checked in release too, exactly as `analyze` checks it. A zero
        // admitted here hangs the same loop.
        assert!(
            consumed > 0,
            "a complete Expression spans at least one Cell"
        );
        Self {
            expression,
            status: AnalysisStatus::Complete,
            consumed,
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn into_expression(self) -> Expression {
        self.expression
    }

    pub fn error(&self) -> Option<&Error> {
        match &self.status {
            AnalysisStatus::Complete => None,
            AnalysisStatus::Incomplete(error) | AnalysisStatus::Invalid(error) => Some(error),
        }
    }

    ///
    /// How many bytes of the Source this analysis read.
    ///
    /// At least one for any non-empty Source, so a caller advancing by it
    /// always makes progress. It is what was read rather than what the layout
    /// claims: an Expression cut short reports the Cells it reached.
    ///
    pub fn consumed(&self) -> usize {
        self.consumed
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, AnalysisStatus::Complete)
    }
}

#[derive(Debug)]
enum AnalysisStatus {
    Complete,
    Incomplete(Error),
    Invalid(Error),
}

impl AnalysisStatus {
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
    /// The Source as it was handed in, so how much was read is a fact about
    /// that rather than about whatever is left of it.
    len: usize,
}

impl<'a> Parser<'a> {
    pub fn from(source: &'a mut str) -> Self {
        Self {
            expression: Expression::new(),
            len: source.len(),
            source,
        }
    }

    ///
    /// How much of the Source has been read.
    ///
    /// Derived from the remainder rather than counted beside it. Two records
    /// of one position drift — the refused-Function arm below rewinds, and a
    /// count kept separately would have had to be rewound in step with the
    /// Source or silently disagree with it. There is one record, so there is
    /// nothing to keep in step.
    ///
    #[inline(always)]
    fn consumed(&self) -> usize {
        self.len - self.source.len()
    }

    /// Strictly parses one complete Expression.
    #[inline]
    pub fn try_parse(mut self) -> Result<Atoms, Error> {
        match self.take_language_unit()? {
            AnalysisStatus::Complete if self.source.is_empty() => Ok(self
                .expression
                .take_atoms()
                .expect("strict parsing contains only values")),
            AnalysisStatus::Complete => {
                Err(SyntaxError::UnexpectedTrailingContent(self.source.to_string()).into())
            }
            AnalysisStatus::Incomplete(error) | AnalysisStatus::Invalid(error) => Err(error),
        }
    }

    ///
    /// Permissively analyzes one Expression while preserving every complete
    /// entry, and reports where that Expression ends.
    ///
    /// Source past the Expression is left for the caller rather than reported
    /// against this one: `.+0102Z` is a complete Addition of six Cells with a
    /// `Z` still to read.
    ///
    #[inline]
    pub fn analyze(mut self) -> Result<SourceAnalysis, Error> {
        let status = self.take_language_unit()?;
        let consumed = self.consumed();
        // Forward progress belongs to the Parser, because the Parser is what
        // decides how far one Expression reaches. A caller looping over a row
        // advances by `consumed`, so a zero here would park it on the same
        // Cell forever. A release assert rather than a debug one: what it
        // prevents is a running Tick hanging, and it costs one branch per
        // Expression rather than per Token.
        assert!(
            consumed > 0 || self.len == 0,
            "a non-empty Source consumes at least one byte"
        );

        Ok(SourceAnalysis {
            expression: self.expression,
            status,
            consumed,
        })
    }

    ///
    /// A Language Unit may be a Function or a standalone Atom.
    #[inline(always)]
    fn take_language_unit(&mut self) -> Result<AnalysisStatus, Error> {
        // Where this Language Unit starts, so the refused-Function arm can put
        // back the Cell it read past rather than leave the Source and the
        // consumed length describing different positions.
        let start = self.source;
        match self.next_token(2) {
            Some(t) => {
                match t {
                    "**" => {
                        self.add(Token::Bang, Atom::Bang)?;
                        return Ok(AnalysisStatus::Complete);
                    }
                    _ if let Ok(activation) = crate::Activation::try_from(t) => {
                        self.add(Token::Activation, Atom::Activation(activation))?;
                        return Ok(AnalysisStatus::Complete);
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
                        // ADR 0018 resumes after one invalid character, not
                        // after the pair `next_token` had to read to try the
                        // spelling. `Z.+0304` is one stray `Z` and then an
                        // Addition; skipping two would eat the `.` and lose
                        // the Function that is really there. The invalid
                        // record says what was attempted and `consumed` says
                        // what to skip, and they disagree deliberately.
                        //
                        // One character, not one byte: `Parser::from` takes
                        // any `&mut str`, as `peek_next` says, and giving back
                        // half of an `é` would land a resuming caller inside a
                        // character and panic the slice.
                        //
                        // The Source is rewound rather than a length beside
                        // it, so the one Cell this arm declines is still there
                        // to be read. `try_parse` decides completeness from
                        // the Source, and this arm returning early is not what
                        // ought to make that safe.
                        //
                        // The rule is stated here alone, so the
                        // `is_function_next` recursion below inherits it.
                        let invalid = t
                            .chars()
                            .next()
                            .expect("a token of two bytes holds a character")
                            .len_utf8();
                        self.source = &start[invalid..];
                        self.expression.add_invalid(Token::Function)?;
                        return Ok(AnalysisStatus::Invalid(error));
                    }
                };

                let mut status = AnalysisStatus::Complete;
                for t in tokens {
                    if self.is_function_next() {
                        status = status.merge(self.take_language_unit()?);
                    } else {
                        match self.take_token(&t) {
                            Ok(Some(atom)) => self.add(t, atom)?,
                            Ok(None) => {
                                status = status.merge(AnalysisStatus::Incomplete(
                                    SyntaxError::ExpectedToken.into(),
                                ));
                            }
                            Err(error) => {
                                self.expression.add_invalid(t)?;
                                status = status.merge(AnalysisStatus::Invalid(error));
                            }
                        }
                    }
                }
                Ok(status)
            }
            None => {
                // No two-Cell spelling starts here — either fewer than two
                // Cells are left, or the second is only part of a character.
                // One character is read and reported either way, which is the
                // same skip a refused Function spelling takes. Reading nothing
                // would report the last `*` of `***` as costing nothing and a
                // caller resuming there would never move; reading the whole
                // tail would step over the `.+0304` after a `€`.
                if let Some(next) = self.source.chars().next() {
                    self.source = &self.source[next.len_utf8()..];
                }
                self.expression.add_incomplete(Token::Char)?;
                Ok(AnalysisStatus::Incomplete(
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
        Atom, Atoms, EXP_LEN, Error, Function, SyntaxError, Token, TypeError,
        parser::{AnalysisStatus, Parser},
        trace,
    };
    use arrayvec::ArrayVec;

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    #[test]
    fn source_analysis_represents_complete_incomplete_and_invalid_source() {
        assert!(matches!(
            Parser::from(&mut ".+0102".to_owned())
                .analyze()
                .unwrap()
                .status,
            AnalysisStatus::Complete
        ));
        assert!(matches!(
            Parser::from(&mut ".+01".to_owned())
                .analyze()
                .unwrap()
                .status,
            AnalysisStatus::Incomplete(_)
        ));
        assert!(matches!(
            Parser::from(&mut ".+01XY".to_owned())
                .analyze()
                .unwrap()
                .status,
            AnalysisStatus::Invalid(_)
        ));
    }

    ///
    /// An Expression is as wide as its root Function's arity, and analysis
    /// says so instead of judging the Source that follows it. `.+0102Z` is a
    /// complete Addition with a `Z` nobody has read yet.
    ///
    #[test]
    fn source_analysis_reports_a_complete_expression_and_leaves_the_source_after_it() {
        let analysis = Parser::from(&mut ".+0102Z".to_owned()).analyze().unwrap();

        assert!(analysis.is_complete());
        assert!(analysis.error().is_none());
        assert_eq!(analysis.consumed(), 6);
        assert_eq!(
            analysis.expression().atoms().unwrap().as_slice(),
            &[
                Atom::Function(Function::Add),
                Atom::Number(1),
                Atom::Number(2),
            ]
        );
    }

    ///
    /// ADR 0018 resumes after one invalid character. The Parser has to read two
    /// to try a Function spelling, and reports the one, so the `.+` in
    /// `Z.+0304` is still there to be read on the next pass.
    ///
    /// The nested case takes the same path: `is_function_next` admits the
    /// recursion only for a spelling that parses, so the only Function failure
    /// there is this one, and it inherits the rule rather than restating it.
    ///
    #[test]
    fn an_unrecognized_function_consumes_one_cell_and_records_one_invalid_slot() {
        let analysis = Parser::from(&mut "Z.+0304".to_owned()).analyze().unwrap();

        assert_eq!(analysis.consumed(), 1);
        assert!(matches!(analysis.status, AnalysisStatus::Invalid(_)));
        assert_eq!(
            analysis.expression().layout().collect::<Vec<_>>(),
            vec![(0, Token::Function, None)]
        );

        // Resuming where it says to reaches the Addition that is really there.
        let resumed = Parser::from(&mut ".+0304".to_owned()).analyze().unwrap();
        assert!(resumed.is_complete());
        assert_eq!(resumed.consumed(), 6);
    }

    ///
    /// An Expression cut short reports the Cells it read, not the ones its
    /// layout claims: `.+01` lays out six and reached four.
    ///
    #[test]
    fn an_incomplete_expression_reports_what_it_read_rather_than_what_it_claims() {
        let analysis = Parser::from(&mut ".+01".to_owned()).analyze().unwrap();

        assert!(matches!(analysis.status, AnalysisStatus::Incomplete(_)));
        assert_eq!(analysis.consumed(), 4);
        assert_eq!(
            analysis
                .expression()
                .tokens()
                .map(|token| token.len())
                .sum::<usize>(),
            6
        );
    }

    ///
    /// `***` is ADR 0018's worked example and the case that would hang a
    /// caller looping on `consumed`: a Bang, then a lone `*` that starts no
    /// Language Unit. The tail is read and reported rather than left behind,
    /// so the loop advances.
    ///
    #[test]
    fn a_source_too_short_for_a_language_unit_still_consumes_its_tail() {
        let bang = Parser::from(&mut "***".to_owned()).analyze().unwrap();
        assert!(bang.is_complete());
        assert_eq!(bang.consumed(), 2);

        let tail = Parser::from(&mut "*".to_owned()).analyze().unwrap();
        assert!(matches!(tail.status, AnalysisStatus::Incomplete(_)));
        assert_eq!(tail.consumed(), 1);

        // Nothing to read consumes nothing, which is the one case the
        // invariant exempts.
        let empty = Parser::from(&mut String::new()).analyze().unwrap();
        assert_eq!(empty.consumed(), 0);
    }

    ///
    /// Strict parsing is unchanged: it still refuses Source it did not consume
    /// whole, which is now the difference between the two readings rather than
    /// something both agree on.
    ///
    #[test]
    fn strict_parsing_still_refuses_source_left_over_after_the_expression() {
        let error = try_parse(&mut ".+0102Z".to_owned()).unwrap_err();

        assert!(matches!(
            error,
            Error::Syntax(SyntaxError::UnexpectedTrailingContent(ref trailing))
                if trailing == "Z"
        ));
    }

    #[test]
    fn layout_preserves_invalid_and_missing_slots_and_later_nested_operands() {
        let analysis = Parser::from(&mut "!>**7F.^3C".to_owned())
            .analyze()
            .unwrap();
        assert!(matches!(analysis.status, AnalysisStatus::Invalid(_)));
        assert_eq!(
            analysis.expression().layout().collect::<Vec<_>>(),
            vec![
                (0, Token::Function, Some(Atom::Function(Function::RawPlay))),
                (2, Token::Number, None),
                (4, Token::Number, Some(Atom::Number(127))),
                (
                    6,
                    Token::Function,
                    Some(Atom::Function(Function::ConvertToNote))
                ),
                (8, Token::Number, Some(Atom::Number(60))),
            ]
        );
        let incomplete = Parser::from(&mut "!>00".to_owned()).analyze().unwrap();
        assert_eq!(
            incomplete.expression().layout().collect::<Vec<_>>(),
            vec![
                (0, Token::Function, Some(Atom::Function(Function::RawPlay))),
                (2, Token::Number, Some(Atom::Number(0))),
                (4, Token::Number, None),
                (6, Token::Note, None),
            ]
        );
    }

    #[test]
    fn binding_layout_repairs_operands_without_reparsing_functions_or_types() {
        let analysis = Parser::from(&mut "!>**7F.v".to_owned()).analyze().unwrap();
        let expression = analysis.expression();
        assert_eq!(
            expression.bind_source("!>007F.vC4").unwrap().as_slice(),
            &[
                Atom::Function(Function::RawPlay),
                Atom::Number(0),
                Atom::Number(127),
                Atom::Function(Function::ConvertToNumber),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ]
        );
        assert!(expression.bind_source("!>007F.^3C").is_err());
        assert!(expression.bind_source("!>.v7F.vC4").is_err());
        assert!(expression.bind_source("!>007F.v").is_err());
        assert!(expression.bind_source("!>007F.v**").is_err());
        // The same two Cells remain a Number or a Note according to the
        // original operand slot, including after an earlier slot was invalid.
        let numeric = Parser::from(&mut ".+XY01".to_owned()).analyze().unwrap();
        assert_eq!(
            numeric.expression().bind_source(".+C401").unwrap()[1],
            Atom::Number(196)
        );
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
    ///
    /// `mod property`'s `literal_source` draws its Source text from here, so
    /// the two operand domains are declared once: a domain narrowed in this
    /// match narrows the generator with it, rather than leaving one of the two
    /// sweeping values the other no longer admits.
    pub(super) fn every_atom_of(token: Token) -> Vec<Atom> {
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
    fn an_operand_literal_outside_a_slot_is_not_an_expression() {
        // ADR 0021 gives an Operand Literal the type of the slot that consumes
        // it, so a literal standing alone has nothing to type it and is
        // invalid Source rather than a one-Atom Expression. The sweep below
        // only ever spells literals inside a slot, and the capacity property
        // starts its chain at one Function to stay off this case, so this is
        // the one place that says a fallback to `to_atom_num` or
        // `to_atom_note` in `take_language_unit` would be wrong.
        for spelled in ["01", "FF", "C4", "3C", "G9"] {
            let mut source = String::from(spelled);
            assert!(
                matches!(
                    try_parse(&mut source),
                    Err(Error::Syntax(SyntaxError::UnknownFunction(_)))
                ),
                "{spelled:?} parsed as an Expression on its own",
            );
        }
    }

    /// Every Atom of an Expression, rendered back to Source text.
    pub(super) fn rendered(atoms: impl IntoIterator<Item = Atom>) -> String {
        atoms.into_iter().map(|atom| atom.to_string()).collect()
    }

    /// A chain of `depth` Additions over `depth + 1` Numbers, wrapped in
    /// `wrappers` unary `.^`s, with the number of Atoms it spells.
    ///
    /// Addition takes two operands, so a chain of them alone spells an odd
    /// `2 * depth + 1` Atoms and can never equal an even `EXP_LEN`. `.^` takes
    /// one, so each wrapper shifts the parity and the two together reach every
    /// count.
    pub(super) fn addition_chain(wrappers: usize, depth: usize) -> (String, usize) {
        let spelled = ".^".repeat(wrappers) + &".+".repeat(depth) + &"00".repeat(depth + 1);
        (spelled, wrappers + 2 * depth + 1)
    }

    ///
    /// The capacity bound falls between an Expression of `EXP_LEN` Atoms and
    /// one of `EXP_LEN + 1`: the first is parsed whole, the second is refused.
    ///
    /// `mod property`'s
    /// `an_expression_that_outruns_the_parser_capacity_is_refused_rather_than_truncated`
    /// sweeps a range that contains both counts, but which counts a run draws
    /// is up to the runner and the pull-request tier draws only 32 cases. The
    /// bound is the one number the criterion is about, so it is spelled here
    /// rather than sampled: a bound off by one in either direction fails on
    /// one of these two Expressions every run, on every tier.
    ///
    /// It lives in `mod test` for the same reason the non-ASCII case below
    /// does: it draws nothing, so the `cfg` that keeps proptest out of a WASM
    /// build has no claim on it. `addition_chain` is shared with the property
    /// from here rather than the other way round, so the two always spell the
    /// same chain.
    ///
    #[test]
    fn the_capacity_bound_falls_between_exp_len_atoms_and_one_more() {
        // `.^` shifts the parity a chain of Additions cannot reach on its own,
        // so these are the two consecutive Atom counts either side of the
        // bound rather than the nearest odd ones.
        let (fits, atoms_spelled) = addition_chain(1, (EXP_LEN - 2) / 2);
        assert_eq!(atoms_spelled, EXP_LEN);
        let (overruns, atoms_spelled) = addition_chain(0, EXP_LEN / 2);
        assert_eq!(atoms_spelled, EXP_LEN + 1);

        let mut source = fits.clone();
        let parsed = Parser::from(&mut source)
            .try_parse()
            .unwrap_or_else(|error| {
                panic!("{fits:?} spells {EXP_LEN} Atoms and was refused: {error:?}")
            });
        assert_eq!(parsed.len(), EXP_LEN);
        assert_eq!(rendered(parsed), fits);

        let mut source = overruns.clone();
        let parsed = Parser::from(&mut source).try_parse();
        assert!(
            matches!(
                parsed,
                Err(Error::Syntax(SyntaxError::ExpressionTooLong { capacity })) if capacity == EXP_LEN
            ),
            "{overruns:?} spells {} Atoms and answered {parsed:?}",
            EXP_LEN + 1,
        );
    }

    ///
    /// Source that is not ASCII declines to parse rather than panicking.
    ///
    /// Every Cell a Grid admits is a single byte, so the Source layer never
    /// hands this text to the parser. `Parser::from` takes any `&mut str`
    /// though, and the totality the property suite states is a claim about the
    /// parser rather than about its callers, so the one input class an ASCII
    /// generator cannot draw is pinned here by hand: a multi-byte character
    /// straddling the two-Cell peek used to split a `char` down the middle.
    ///
    /// This is a plain test rather than a property, and it belongs here rather
    /// than in `mod property`: it draws nothing, so the `cfg` that keeps
    /// proptest out of a WASM build has no claim on it. `check_wasm`'s
    /// `--all-targets` clippy now type-checks it, which is as far as any
    /// `lang` test reaches on that target — `test_wasm` runs the `shell`
    /// crate's browser suite and no unit test here — so what running it proves
    /// is proven natively.
    ///
    #[test]
    fn source_that_is_not_ascii_is_refused_rather_than_panicking() {
        // `".+aé"` and `".+00aé"` are the cases that reach the fix: each
        // consumes whole Language Units and leaves `"aé"`, so byte two of the
        // remaining Source falls inside the `é` that `peek_next` is then asked
        // about, which is what `split_at(2)` panicked on. The other four
        // decline before any peek, so they widen the input class without
        // covering the fix — keep them, but do not mistake them for coverage
        // of it. The offset is what matters rather than the `.+`: `".+0aé"`
        // leaves an odd byte count and lands the split off the character
        // boundary, so it declines like the rest.
        for spelled in [".+aé", ".+00aé", "é", "aé", "é.+", "..éé"] {
            let mut source = String::from(spelled);
            let parsed = Parser::from(&mut source).try_parse();
            assert!(parsed.is_err(), "{spelled:?} parsed as {parsed:?}");

            let mut source = String::from(spelled);
            // Analysis is the permissive reading and answers rather than
            // failing, so it returns at all — and what it returns is a byte
            // count a caller can resume from. `"é!"` is the case that would
            // not be: `é` is refused as a Function spelling, and reporting one
            // byte rather than one character would hand back an offset inside
            // it.
            let analysis = Parser::from(&mut source).analyze().unwrap();
            assert!(
                spelled.is_char_boundary(analysis.consumed()),
                "{spelled:?} consumed {} bytes, which is not a character boundary",
                analysis.consumed(),
            );
        }

        // A Function spelling the Parser read whole and refused costs its
        // first character.
        let analysis = Parser::from(&mut String::from("é!")).analyze().unwrap();
        assert_eq!(analysis.consumed(), "é".len());

        // A character too wide to read a spelling across costs the same one
        // character. Draining the rest instead would step over the Addition
        // that follows and lose the row to a single mistyped Cell, which is
        // the whole point of skipping one.
        let analysis = Parser::from(&mut String::from("€.+0304"))
            .analyze()
            .unwrap();
        assert_eq!(analysis.consumed(), "€".len());
        let resumed = Parser::from(&mut String::from(".+0304")).analyze().unwrap();
        assert!(resumed.is_complete());
        assert_eq!(resumed.consumed(), 6);
    }

    #[test]
    fn every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it() {
        // A standalone Language Unit is a whole Expression, so it renders and
        // parses back with no Function to type it.
        for atom in std::iter::once(Atom::Bang)
            .chain(crate::Activation::ALL.iter().copied().map(Atom::Activation))
        {
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
/// sampling. `mod test` also holds the two claims here that need no generator
/// — `source_that_is_not_ascii_is_refused_rather_than_panicking` and
/// `the_capacity_bound_falls_between_exp_len_atoms_and_one_more` — along with
/// the `every_atom_of`, `addition_chain` and `rendered` helpers this module
/// draws from, so that nothing a WASM build could run is gated off with the
/// generators.
///
/// `orcvs::source::language_map`'s `mod property` has a fragment generator of
/// the same shape, and the two are deliberately separate: `orcvs` depends on
/// `lang`, so sharing one would mean a test-support module here compiled into
/// a dependency for the sake of a test. The duplication is recorded rather
/// than left to be discovered — a new run boundary or a second Comment form
/// has to be taught to both, and neither coverage guard notices if only one
/// learns it.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {

    use super::test::{addition_chain, every_atom_of, rendered};
    use crate::{Atom, EXP_LEN, Error, Function, SyntaxError, Token, parser::Parser};
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    // Aliased because `Cell` is a domain noun everywhere else in this file and
    // in CONTEXT.md. What `std::cell::Cell` holds here is a draw count.
    use std::cell::Cell as Counter;

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

    /// The Atoms that are a whole Language Unit on their own: the Bang and
    /// every Activation, read from `Activation::ALL` so a fifth one is drawn
    /// the day it is declared.
    fn standalone() -> Vec<Atom> {
        std::iter::once(Atom::Bang)
            .chain(crate::Activation::ALL.iter().copied().map(Atom::Activation))
            .collect()
    }

    /// Source text for one Operand Literal of the type its position declares.
    ///
    /// ADR 0021 makes an Operand Literal's type the consuming Function's rather
    /// than the Source's, so a literal is spelled against the `Token` the slot
    /// declares. The same two Cells spell a Number in one slot and a Note in
    /// another.
    ///
    /// The domain comes from `mod test`'s `every_atom_of` rather than from a
    /// second range written here, so the enumerated round trip and this
    /// generator can never disagree about what a slot admits.
    fn literal_source(token: Token) -> BoxedStrategy<String> {
        select(
            every_atom_of(token)
                .iter()
                .map(Atom::to_string)
                .collect::<Vec<String>>(),
        )
        .boxed()
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
            1 => select(standalone()).prop_map(|atom| atom.to_string()),
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
                    // No assertion that `atoms.len() <= EXP_LEN`: `Atoms` is
                    // `ArrayVec<Atom, EXP_LEN>`, so the type already bounds it
                    // and the check would hold just as well for a parse that
                    // truncated at capacity instead of refusing. The rendering
                    // equality below is what catches a truncation.
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

            // What was read is a prefix of what was handed in, and a
            // non-empty Source always moves: a caller that advances by
            // `consumed` walks the row rather than parking on a Cell.
            prop_assert!(analysis.consumed() <= spelled.len(), "{spelled:?}");
            prop_assert_eq!(analysis.consumed() > 0, !spelled.is_empty(), "{:?}", spelled);

            if analysis.is_complete() {
                prop_assert!(analysis.error().is_none());
                let atoms = expression
                    .atoms()
                    .expect("a complete analysis holds only complete entries");
                // A complete Expression spells exactly the Cells it consumed.
                // Anything after them is the next Expression's Source and is
                // neither read nor held against this one.
                prop_assert_eq!(rendered(atoms), &spelled[..analysis.consumed()]);
            } else {
                prop_assert!(analysis.error().is_some());
                // Every way of not completing records the Token it could not
                // read, and that record is what withholds the Atoms above.
                prop_assert!(
                    expression.atoms().is_none(),
                    "{spelled:?} produced runtime Atoms for {:?}",
                    analysis.error(),
                );
            }

            // Analysis reports a boundary rather than refusing what follows
            // it, so it never raises the diagnostic strict parsing raises for
            // Source it did not consume whole.
            prop_assert!(
                !matches!(
                    analysis.error(),
                    Some(Error::Syntax(SyntaxError::UnexpectedTrailingContent(_)))
                ),
                "{spelled:?} was refused for its trailing Source",
            );
        }

        ///
        /// The two contracts agree about exactly one thing: strict parsing
        /// accepts the Source analysis reads whole and calls complete, and no
        /// other. Analysis is the permissive path, so what separates them is
        /// that it also answers for the rest — not that it reads a different
        /// language. Reading whole is part of the agreement rather than a
        /// consequence of it: analysis calls `.+0102Z` complete at six Cells,
        /// and strict parsing refuses the `Z` it did not consume.
        ///
        #[test]
        fn strict_parsing_accepts_exactly_the_source_analysis_reads_whole(
            source in generated_source(),
        ) {
            let mut strict = source.clone();
            let mut permissive = source.clone();

            let parsed = Parser::from(&mut strict).try_parse();
            let analysis = Parser::from(&mut permissive).analyze();

            let complete = analysis.as_ref().is_ok_and(|analysis| {
                analysis.is_complete() && analysis.consumed() == source.len()
            });
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
        /// The Additions are wrapped in `wrappers` unary `.^`s so the Atom
        /// count reaches both parities. A chain of Additions alone spells
        /// `2 * depth + 1` Atoms, which is always odd and therefore steps
        /// straight over an even `EXP_LEN`: it would assert 31 accepted and 33
        /// refused and never spell exactly 32. Each `.^` takes one operand and
        /// adds one Atom, so `wrappers` of zero or one reaches every count in
        /// the range. Which counts a run actually draws is still up to the
        /// runner, so the boundary itself is pinned by
        /// `the_capacity_bound_falls_between_exp_len_atoms_and_one_more`
        /// rather than left to a sample.
        ///
        #[test]
        fn an_expression_that_outruns_the_parser_capacity_is_refused_rather_than_truncated(
            depth in 1usize..24,
            wrappers in 0usize..=1,
        ) {
            let (spelled, atoms_spelled) = addition_chain(wrappers, depth);
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
    /// because this claim is about the generator rather than about the parser.
    /// It does parse each draw — that is how the last of the four counts is
    /// taken — so the fixed 256 cases are 256 parses that neither verification
    /// tier can dial down. That is the cost of the claim rather than an
    /// oversight: a coverage guard that weakened with the tier would stop
    /// guarding exactly where the tier is cheapest.
    ///
    #[test]
    fn generated_source_covers_the_space_the_incomplete_hash_and_the_comment_introducer() {
        let config = Config {
            cases: 256,
            source_file: Some(file!()),
            ..Config::default()
        };
        let space = Counter::new(0usize);
        let incomplete = Counter::new(0usize);
        let comment = Counter::new(0usize);
        let complete = Counter::new(0usize);

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
                // A Function among the Atoms, not merely a parse that
                // succeeded. A lone standalone Atom parses whole and would
                // satisfy a bare `is_ok`, which leaves the guard passing on
                // Source that reaches none of the operand-typing the
                // properties above are about.
                if let Ok(atoms) = Parser::from(&mut spelled).try_parse()
                    && atoms.iter().any(|atom| matches!(atom, Atom::Function(_)))
                {
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
            "no generated Source spelled a Function-bearing Expression strict parsing accepts",
        );
    }
}
