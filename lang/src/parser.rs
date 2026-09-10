use crate::Atom;
use crate::Atoms;
use crate::Error;
use crate::Expression;
use crate::Function;
use crate::SyntaxError;
use crate::Token;

use std::ops::Range;

///
/// What was read from one Source, and where in that Source it was read.
///
/// Function signatures and nested Functions determine each Expression's extent.
/// Reporting that boundary lets a caller resume at the next Expression.
///
/// The boundary is an address rather than a width. A caller hands the Parser
/// the Cell its Source begins at and gets back the Cells the Expression
/// occupies, so there is no offset relative to a fragment for a consumer to
/// re-base against wherever that fragment came from. A caller with no Grid to
/// answer to reads from Cell zero and gets offsets, which is the same thing.
///
#[derive(Debug)]
pub struct SourceAnalysis {
    expression: Expression,
    error: Option<Error>,
    start: usize,
    consumed: usize,
}

impl SourceAnalysis {
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn into_expression(self) -> Expression {
        self.expression
    }

    pub fn error(&self) -> Option<&Error> {
        self.error.as_ref()
    }

    ///
    /// The Cells this Expression occupies, in the address space of the Source
    /// the Parser was given.
    ///
    /// Half-open, and never empty for a non-empty Source: `analyze` reads at
    /// least one Cell, so a caller resuming at `end` always advances. When the
    /// Source handed in was a Grid row and `start` the Cell it begins at,
    /// these are the Grid's own Cell indices — which is the whole point, and
    /// why there is no width to re-base against an anchor.
    ///
    pub fn cells(&self) -> Range<usize> {
        self.start..self.start + self.consumed
    }

    pub fn is_complete(&self) -> bool {
        self.error.is_none()
    }
}

pub struct Parser<'a> {
    expression: Expression,
    source: &'a str,
    /// The Source as it was handed in, so how much was read is a fact about
    /// that rather than about whatever is left of it.
    len: usize,
    /// The Cell the Source begins at, so what was read can be reported as an
    /// address rather than as an offset into a fragment.
    start: usize,
}

impl<'a> Parser<'a> {
    pub fn from(source: &'a mut str) -> Self {
        Self::at(source, 0)
    }

    ///
    /// Reads `source`, whose first Cell is `start`.
    ///
    /// This is the constructor the Language Map's row walk uses: it hands over
    /// the row's remaining Cells and the Cell index they begin at, so the
    /// positions the analysis reports are the Grid's own and no consumer has
    /// to add an anchor back to them. `start` is a number the Parser reports
    /// and never reads with — `lang` owns no Grid and places nothing.
    ///
    pub fn at(source: &'a str, start: usize) -> Self {
        Self {
            expression: Expression::new(),
            len: source.len(),
            source,
            start,
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
        if let Some(error) = self.take_language_unit() {
            return Err(error);
        }
        if !self.source.is_empty() {
            return Err(SyntaxError::UnexpectedTrailingContent(self.source.to_string()).into());
        }
        // Every record of an Expression that reported no error carries an
        // Atom, with one exception: a Comment is a complete Language Unit
        // that is not a value (ADR 0035), so it records a Token and nothing
        // else. Strict parsing yields values, and has none to yield here.
        self.expression
            .take_atoms()
            .ok_or_else(|| SyntaxError::CommentIsNotAValue.into())
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
    pub fn analyze(mut self) -> SourceAnalysis {
        let error = self.take_language_unit();
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

        SourceAnalysis {
            expression: self.expression,
            error,
            start: self.start,
            consumed,
        }
    }

    ///
    /// A Language Unit may be a Function or a standalone Atom.
    #[inline(always)]
    fn take_language_unit(&mut self) -> Option<Error> {
        // Most Expressions need only a few pending operands. Avoid allocating
        // a stack for each parse, but spill deeper expressions to the heap:
        // this inline capacity is an optimization, not a language limit.
        let mut pending = arrayvec::ArrayVec::<_, 16>::new();
        let mut overflow = Vec::new();
        pending.push((Token::Function, None));
        let mut error = None;
        while let Some((token, parent)) = overflow.pop().or_else(|| pending.pop()) {
            let cell_start = self.start + self.consumed();
            if token != Token::Function && !self.is_function_next() {
                match self.take_token(&token) {
                    Ok(atom) => self.expression.add_positioned(
                        token,
                        Some(atom),
                        cell_start..self.start + self.consumed(),
                        parent,
                    ),
                    Err(failure) => {
                        error.get_or_insert(failure);
                        self.expression.add_positioned(
                            token,
                            None,
                            cell_start..self.start + self.consumed(),
                            parent,
                        );
                    }
                }
                continue;
            }
            let start = self.source;
            let atom = match self.next_token(2) {
                // A Comment claims every remaining Cell and records a Token
                // with no Atom (ADR 0035). It is answered here rather than in
                // the `(Token, Atom)` match below because there is no Atom to
                // answer with: an Atom is a fixed-width value and a Comment
                // carries a row of arbitrary text that is never decoded.
                //
                // This arm is reached only where a new Expression could start.
                // An operand slot takes the branch above unless a Function
                // spelling is next, and `||` is not one, so a `||` inside a
                // Function's arity-determined claim is an operand Cell that
                // fails to bind.
                Some("||") => {
                    let _ = self.next_token(self.source.len());
                    self.expression.add_positioned(
                        Token::Comment,
                        None,
                        cell_start..self.start + self.consumed(),
                        parent,
                    );
                    continue;
                }
                Some("**") => Ok((Token::Bang, Atom::Bang)),
                Some(t) => match crate::Activation::try_from(t) {
                    Ok(activation) => Ok((Token::Activation, Atom::Activation(activation))),
                    Err(_) => Function::try_from(t)
                        .map(|function| (Token::Function, Atom::Function(function))),
                },
                None => Err(SyntaxError::ExpectedFunction.into()),
            };
            match atom {
                Ok((token, atom)) => {
                    let index = self.expression.len();
                    self.expression.add_positioned(
                        token,
                        Some(atom),
                        cell_start..self.start + self.consumed(),
                        parent,
                    );
                    if let Atom::Function(function) = atom {
                        // Reverse signature order keeps the next operand on top,
                        // without growing the native call stack for nested Functions.
                        for token in function.signature().iter().rev() {
                            let item = (*token, Some(index));
                            // Overflow is always popped first, so any occupied
                            // overflow has a full inline stack beneath it.
                            if pending.is_full() {
                                overflow.push(item);
                            } else {
                                pending.push(item);
                            }
                        }
                    }
                }
                Err(failure) => {
                    // A refused Function advances one character (ADR 0018).
                    // Use a character boundary for callers outside ASCII Source.
                    self.source = &start[start.chars().next().map_or(0, char::len_utf8)..];
                    error.get_or_insert(failure);
                    self.expression.add_positioned(
                        Token::Function,
                        None,
                        cell_start..self.start + self.consumed(),
                        parent,
                    );
                }
            }
        }
        error
    }

    ///
    /// Reads the operand a signature declares here and converts it.
    ///
    /// A slot whose Cells run past the end of the Source is refused like any
    /// other slot that does not hold what it declares. The Source ending is
    /// not a state of its own: a row is as long as it is, and a Function whose
    /// arity reaches past the edge is one the Grid cannot hold.
    ///
    #[inline(always)]
    fn take_token(&mut self, token: &Token) -> Result<Atom, Error> {
        let Some(t) = self.next_token(token.len()) else {
            // The available tail belongs to this operand, even when it is
            // too short to spell one. It cannot open another Expression.
            if self.source.len() < token.len() {
                self.source = "";
            }
            return Err(SyntaxError::ExpectedToken.into());
        };
        token.decode(t)
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
        Atom, Atoms, Error, Function, SyntaxError, Token, TypeError, parser::Parser, trace,
    };

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    #[test]
    fn source_analysis_represents_complete_incomplete_and_invalid_source() {
        assert!(
            Parser::from(&mut ".+0102".to_owned())
                .analyze()
                .error
                .is_none()
        );
        // Cut short by the end of the Source rather than by a character that
        // does not convert, and refused all the same: there is no third state.
        assert!(
            Parser::from(&mut ".+01".to_owned())
                .analyze()
                .error
                .is_some()
        );
        assert!(
            Parser::from(&mut ".+01XY".to_owned())
                .analyze()
                .error
                .is_some()
        );
    }

    ///
    /// An Expression is as wide as its root Function's arity, and analysis
    /// says so instead of judging the Source that follows it. `.+0102Z` is a
    /// complete Addition with a `Z` nobody has read yet.
    ///
    #[test]
    fn source_analysis_reports_a_complete_expression_and_leaves_the_source_after_it() {
        let analysis = Parser::from(&mut ".+0102Z".to_owned()).analyze();

        assert!(analysis.is_complete());
        assert!(analysis.error().is_none());
        assert_eq!(analysis.cells().end, 6);
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
    /// An analysis reports the Cells it read, not an offset into the fragment
    /// it was handed.
    ///
    /// This is the whole of what `Parser::at` buys: the caller says which Cell
    /// its Source begins at and the boundary comes back in that same address
    /// space, so a row walk resuming at `cells().end` needs no arithmetic of
    /// its own. The three Sources are the three shapes a walk meets — one
    /// whole Expression, one refused character, and the Expression after it —
    /// read from successive Cells of one row.
    ///
    #[test]
    fn an_analysis_reports_the_cells_it_read_from_the_cell_it_was_told_it_began_at() {
        let row = "Z.+0304";

        let refused = Parser::at(&row[0..], 12).analyze();
        assert_eq!(refused.cells(), 12..13);

        let addition = Parser::at(&row[1..], 13).analyze();
        assert!(addition.is_complete());
        assert_eq!(addition.cells(), 13..19);

        // The same Source read as though it began the Grid: `Parser::from` is
        // `Parser::at` at Cell zero, so a caller with no Grid to answer to
        // reads offsets and a caller with one reads addresses.
        let anywhere = Parser::from(&mut String::from(&row[1..])).analyze();
        assert_eq!(anywhere.cells(), 0..6);
        assert_eq!(anywhere.cells().len(), addition.cells().len());
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
        let analysis = Parser::from(&mut "Z.+0304".to_owned()).analyze();

        assert_eq!(analysis.cells().end, 1);
        assert!(analysis.error.is_some());
        assert_eq!(
            analysis
                .expression()
                .positioned()
                .map(|entry| (entry.cells.start, entry.token, entry.atom))
                .collect::<Vec<_>>(),
            vec![(0, Token::Function, None)]
        );

        // Resuming where it says to reaches the Addition that is really there.
        let resumed = Parser::from(&mut ".+0304".to_owned()).analyze();
        assert!(resumed.is_complete());
        assert_eq!(resumed.cells().end, 6);
    }

    ///
    /// An Expression cut short reports the Cells it read, not the ones its
    /// layout claims: `.+01` lays out six and reached four.
    ///
    #[test]
    fn an_expression_cut_short_reports_what_it_read_rather_than_what_it_claims() {
        let analysis = Parser::from(&mut ".+01".to_owned()).analyze();

        assert!(analysis.error.is_some());
        assert_eq!(analysis.cells().end, 4);
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
        let bang = Parser::from(&mut "***".to_owned()).analyze();
        assert!(bang.is_complete());
        assert_eq!(bang.cells().end, 2);

        // One Cell is not a spelling, so it is refused as one — the same
        // answer a two-Cell spelling the table does not hold gets. Running out
        // of Source is not a state of its own.
        let tail = Parser::from(&mut "*".to_owned()).analyze();
        assert!(tail.error.is_some());
        assert_eq!(tail.cells().end, 1);

        // Nothing to read consumes nothing, which is the one case the
        // invariant exempts.
        let empty = Parser::from(&mut String::new()).analyze();
        assert_eq!(empty.cells().end, 0);
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

    ///
    /// A Comment is a Language Unit the Parser establishes: `||` claims every
    /// remaining Cell of the Source it was handed, records a Token and no
    /// Atom, and completes. ADR 0035.
    ///
    #[test]
    fn a_comment_claims_the_rest_of_the_source_and_records_no_atom() {
        let analysis = Parser::from(&mut "|| .+0102 anything at all".to_owned()).analyze();

        assert!(analysis.is_complete());
        assert_eq!(analysis.cells(), 0..25);
        assert_eq!(
            analysis
                .expression()
                .positioned()
                .map(|entry| (entry.cells.clone(), entry.token, entry.atom))
                .collect::<Vec<_>>(),
            vec![(0..25, Token::Comment, None)]
        );
        // Complete and yet answering with nothing: the one Language Unit that
        // is not a value. An Expression yields Atoms only when every record
        // it holds carries one, so a Comment withholds them without an error.
        assert!(analysis.expression().atoms().is_none());
        assert!(analysis.expression().entries().next().is_none());
    }

    ///
    /// `||` opens a Comment only where an Expression could start. Inside a
    /// Function's arity-determined claim it is an operand Cell that fails to
    /// bind, which is ADR 0033's partition by parse read straight: a spelling
    /// is recognized only in the position where a spelling is read.
    ///
    #[test]
    fn a_comment_introducer_inside_an_operand_claim_is_a_refused_operand() {
        let analysis = Parser::from(&mut ".+||02".to_owned()).analyze();

        assert!(analysis.error.is_some());
        assert_eq!(analysis.cells(), 0..6);
        assert_eq!(
            analysis
                .expression()
                .positioned()
                .map(|entry| (entry.cells.start, entry.token, entry.atom))
                .collect::<Vec<_>>(),
            vec![
                (0, Token::Function, Some(Atom::Function(Function::Add))),
                (2, Token::Number, None),
                (4, Token::Number, Some(Atom::Number(2))),
            ]
        );
    }

    ///
    /// One `|` alone is incomplete or invalid Source, exactly as one `#` was.
    /// It costs the Cell it occupies and leaves the rest of the row readable,
    /// which is ADR 0018's recovery rather than a rule of its own.
    ///
    #[test]
    fn a_lone_vertical_rule_is_not_a_comment() {
        let analysis = Parser::from(&mut "|.+0304".to_owned()).analyze();

        assert!(analysis.error.is_some());
        assert_eq!(analysis.cells(), 0..1);
        assert_eq!(
            analysis
                .expression()
                .positioned()
                .map(|entry| (entry.cells.start, entry.token, entry.atom))
                .collect::<Vec<_>>(),
            vec![(0, Token::Function, None)]
        );

        let resumed = Parser::from(&mut ".+0304".to_owned()).analyze();
        assert!(resumed.is_complete());
    }

    ///
    /// Strict parsing yields values, and ADR 0035 makes a Comment a complete
    /// Language Unit that is not one. It reports that rather than unwrapping
    /// an Expression that holds no Atoms.
    ///
    #[test]
    fn strict_parsing_refuses_a_comment() {
        let error = try_parse(&mut "||whatever".to_owned()).unwrap_err();

        assert!(matches!(
            error,
            Error::Syntax(SyntaxError::CommentIsNotAValue)
        ));
    }

    #[test]
    fn layout_preserves_invalid_and_missing_slots_and_later_nested_operands() {
        let analysis = Parser::from(&mut "!>**7F.^3C".to_owned()).analyze();
        assert!(analysis.error.is_some());
        assert_eq!(
            analysis
                .expression()
                .positioned()
                .map(|entry| (entry.cells.start, entry.token, entry.atom))
                .collect::<Vec<_>>(),
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
        let incomplete = Parser::from(&mut "!>00".to_owned()).analyze();
        assert_eq!(
            incomplete
                .expression()
                .positioned()
                .map(|entry| (entry.cells.start, entry.token, entry.atom))
                .collect::<Vec<_>>(),
            vec![
                (0, Token::Function, Some(Atom::Function(Function::RawPlay))),
                (2, Token::Number, Some(Atom::Number(0))),
                (4, Token::Number, None),
                (4, Token::Note, None),
            ]
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

    fn parse(exp: &mut str) -> Vec<Atom> {
        Parser::from(exp)
            .analyze()
            .into_expression()
            .take_atoms()
            .unwrap_or_default()
            .into_iter()
            .collect()
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let mut s = String::from(".+");
        let parsed = parse(&mut s);

        assert!(parsed.is_empty());

        let mut s = String::from("+");
        let parsed = parse(&mut s);

        let stack = vec![];
        assert_eq!(parsed, stack);

        let mut s = String::from("..");
        let parsed = parse(&mut s);
        assert!(parsed.is_empty());

        let mut s = String::from("ABC");
        let parsed = parse(&mut s);
        assert!(parsed.is_empty());

        let mut s = String::from("A           ");
        let parsed = parse(&mut s);
        assert!(parsed.is_empty());
    }

    #[test]
    fn permissive_parse_keeps_non_values_out_of_runtime_atoms() {
        let incomplete = Parser::from(".+01".to_owned().as_mut_str())
            .analyze()
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

        let mut expected = Vec::new();
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

        let mut expected = Vec::new();
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

        let mut expected = Vec::new();
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
        // only ever spells literals inside a slot, and the long-expression property
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
            let analysis = Parser::from(&mut source).analyze();
            assert!(
                spelled.is_char_boundary(analysis.cells().end),
                "{spelled:?} consumed {} bytes, which is not a character boundary",
                analysis.cells().end,
            );
        }

        // A Function spelling the Parser read whole and refused costs its
        // first character.
        let analysis = Parser::from(&mut String::from("é!")).analyze();
        assert_eq!(analysis.cells().end, "é".len());

        // A character too wide to read a spelling across costs the same one
        // character. Draining the rest instead would step over the Addition
        // that follows and lose the row to a single mistyped Cell, which is
        // the whole point of skipping one.
        let analysis = Parser::from(&mut String::from("€.+0304")).analyze();
        assert_eq!(analysis.cells().end, "€".len());
        let resumed = Parser::from(&mut String::from(".+0304")).analyze();
        assert!(resumed.is_complete());
        assert_eq!(resumed.cells().end, 6);
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
/// ends a run, the incomplete `|`, and the `||` Comment introducer are drawn as
/// text like everything else — and
/// `generated_source_covers_the_space_the_incomplete_rule_and_the_comment_introducer`
/// pins that they are actually reached rather than merely reachable.
///
/// The `Atom` round trip is `mod test`'s
/// `every_atom_the_parser_yields_round_trips_through_display_in_the_position_that_types_it`
/// rather than a property: the two operand domains hold 384 values between
/// them, which is small enough to enumerate and too small to be worth
/// sampling. `mod test` also holds the non-ASCII regression and the
/// `every_atom_of` helper this module uses; `addition_chain` and `rendered`
/// are drawn on from here alone, so they sit here and leave the WASM build
/// with this module rather than needing a `cfg` of their own.
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

    use super::test::every_atom_of;
    use crate::{Atom, Error, Function, SyntaxError, Token, parser::Parser};
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
    const FRAGMENTS: usize = 24;

    /// Every Atom of an Expression, rendered back to Source text.
    fn rendered(atoms: impl IntoIterator<Item = Atom>) -> String {
        atoms.into_iter().map(|atom| atom.to_string()).collect()
    }

    /// A chain of `depth` Additions over `depth + 1` Numbers, wrapped in
    /// `wrappers` unary `.^`s, with the number of Atoms it spells.
    ///
    fn addition_chain(wrappers: usize, depth: usize) -> (String, usize) {
        let spelled = ".^".repeat(wrappers) + &".+".repeat(depth) + &"00".repeat(depth + 1);
        (spelled, wrappers + 2 * depth + 1)
    }

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
    /// the space that ends a run, the `||` Comment introducer that claims the
    /// rest of it, the `|` that is incomplete Source rather than a Comment, a
    /// Function spelling with no operands after it, a standalone Atom, and an
    /// Operand Literal outside any slot. They are concatenated in whatever
    /// order they are drawn, so the result is raw text rather than a grammar.
    fn fragment() -> BoxedStrategy<String> {
        prop_oneof![
            8 => proptest::char::range(' ', '~').prop_map(String::from),
            2 => Just(" ".to_owned()),
            2 => Just("|".to_owned()),
            2 => Just("||".to_owned()),
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

            // Total: analysis answers for every printable-ASCII Source,
            // including invalid Expressions.
            let analysis = Parser::from(&mut source).analyze();

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
            prop_assert!(analysis.cells().end <= spelled.len(), "{spelled:?}");
            prop_assert_eq!(analysis.cells().end > 0, !spelled.is_empty(), "{:?}", spelled);

            if analysis.is_complete() {
                prop_assert!(analysis.error().is_none());
                // A complete analysis holds only complete entries, or is a
                // Comment. ADR 0035 makes a Comment a complete Language Unit
                // that is not a value: it records a Token and no Atom, so the
                // Expression withholds its Atoms with nothing to report, and
                // there is no Source to render back because its text is
                // arbitrary and was never decoded.
                match expression.atoms() {
                    // A complete Expression spells exactly the Cells it
                    // consumed. Anything after them is the next Expression's
                    // Source and is neither read nor held against this one.
                    Some(atoms) => {
                        prop_assert_eq!(rendered(atoms), &spelled[..analysis.cells().end])
                    }
                    None => {
                        prop_assert_eq!(
                            expression.tokens().collect::<Vec<_>>(),
                            vec![Token::Comment],
                            "{:?} completed with no Atoms and no Comment",
                            spelled,
                        );
                        // And the claim is every Cell there was to read.
                        prop_assert_eq!(analysis.cells(), 0..spelled.len(), "{:?}", spelled);
                    }
                }
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
        /// accepts the Source analysis reads whole, calls complete, and reads
        /// as values, and no other. Analysis is the permissive path, so what
        /// separates them is that it also answers for the rest — not that it
        /// reads a different language. Reading whole is part of the agreement
        /// rather than a consequence of it: analysis calls `.+0102Z` complete
        /// at six Cells, and strict parsing refuses the `Z` it did not
        /// consume.
        ///
        /// The values clause is the Comment, and it is the only Source the
        /// two contracts read alike and answer differently. ADR 0035 makes a
        /// Comment a complete Language Unit that is not a value, so `||x` is
        /// read whole and called complete by analysis and still refused by
        /// the path whose whole output is Atoms.
        ///
        #[test]
        fn strict_parsing_accepts_exactly_the_source_analysis_reads_whole(
            source in generated_source(),
        ) {
            let mut strict = source.clone();
            let mut permissive = source.clone();

            let parsed = Parser::from(&mut strict).try_parse();
            let analysis = Parser::from(&mut permissive).analyze();

            let comment = analysis
                .expression()
                .tokens()
                .any(|token| token == Token::Comment);
            let complete =
                analysis.is_complete() && analysis.cells().end == source.len() && !comment;
            prop_assert_eq!(parsed.is_ok(), complete, "{:?}", source);

            if let Ok(atoms) = parsed {
                prop_assert_eq!(
                    Some(atoms),
                    analysis.into_expression().take_atoms(),
                    "{:?}",
                    source,
                );
            }
        }

        /// Parsing retains every Atom, including expressions beyond the old storage limit.
        #[test]
        fn long_expressions_roundtrip_without_truncation(
            depth in 1usize..128,
            wrappers in 0usize..=1,
        ) {
            let (spelled, atoms_spelled) = addition_chain(wrappers, depth);
            let parsed = Parser::at(&spelled, 0).try_parse().map_err(|error|
                TestCaseError::fail(format!("{spelled:?} was refused with {error:?}"))
            )?;
            prop_assert_eq!(parsed.len(), atoms_spelled);
            prop_assert_eq!(rendered(parsed), spelled.as_str());
            let analysis = Parser::at(&spelled, 0).analyze();
            prop_assert!(analysis.is_complete());
            prop_assert_eq!(analysis.cells(), 0..spelled.len());
        }
    }

    ///
    /// The generator reaches the three pieces of Source the language treats
    /// specially — the space that ends a run, the `|` that is incomplete
    /// Source rather than a Comment, and the `||` Comment introducer — and it
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
    /// It does read each draw twice — analysis for the Comment count and
    /// strict parsing for the last — so the fixed 256 cases are 512 reads
    /// that neither verification tier can dial down. That is the cost of
    /// counting what the parser established rather than what the text held,
    /// and it is the cost of the claim rather than an
    /// oversight: a coverage guard that weakened with the tier would stop
    /// guarding exactly where the tier is cheapest.
    ///
    #[test]
    fn generated_source_covers_the_space_the_incomplete_rule_and_the_comment_introducer() {
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
                // An analysis that actually established a Comment, not a `||`
                // anywhere in the text. `||` opens a Comment only where an
                // Expression could start, so `.+01||` holds the introducer and
                // reaches none of the Comment arm the guard exists to protect.
                let mut permissive = source.clone();
                if Parser::from(&mut permissive)
                    .analyze()
                    .expression()
                    .tokens()
                    .any(|token| token == Token::Comment)
                {
                    comment.set(comment.get() + 1);
                }
                // A `|` with no `|` beside it: incomplete Source rather than
                // the introducer, which is the distinction CONTEXT.md draws.
                let bytes = source.as_bytes();
                if bytes.iter().enumerate().any(|(index, byte)| {
                    *byte == b'|'
                        && bytes.get(index + 1) != Some(&b'|')
                        && (index == 0 || bytes[index - 1] != b'|')
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
            "no generated Source held an incomplete `|`",
        );
        assert!(
            comment.get() > 0,
            "no generated Source analyzed as a Comment",
        );
        // And the accepting half of every property above has to be reached by
        // something, or those properties pass by never running.
        assert!(
            complete.get() > 0,
            "no generated Source spelled a Function-bearing Expression strict parsing accepts",
        );
    }
}

#[cfg(test)]
mod positioned_tests {
    use super::Parser;

    #[test]
    fn parser_positions_keep_nested_ownership_and_truncated_inputs() {
        let parse = Parser::at(".+02.x03", 40).analyze();
        let slots: Vec<_> = parse
            .expression()
            .positioned()
            .map(|slot| (slot.cells.clone(), slot.parent))
            .collect();
        assert_eq!(
            slots,
            vec![
                (40..42, None),
                (42..44, Some(0)),
                (44..46, Some(0)),
                (46..48, Some(2)),
                (48..48, Some(2))
            ]
        );
    }
}
