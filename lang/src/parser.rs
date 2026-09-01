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
use std::ops::Not;

pub struct Parser<'a> {
    expression: Expression,
    source: &'a str,
    check: bool,
    invalid: bool,
}

impl<'a> Parser<'a> {
    pub fn from(source: &'a mut str) -> Self {
        Self {
            expression: Expression::new(),
            source,
            check: false,
            invalid: false,
        }
    }

    ///
    /// try_parse will error if the parse fails
    ///
    pub fn try_parse(mut self) -> Result<Atoms, Error> {
        self.take_language_unit()?;
        if !self.source.is_empty() {
            return Err(SyntaxError::UnexpectedTrailingContent(self.source.to_string()).into());
        }
        Ok(self
            .expression
            .take_atoms()
            .expect("strict parsing cannot produce analysis-only entries"))
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///
    pub fn parse(mut self) -> Result<Expression, Error> {
        self.check = true;
        self.invalid = false;
        match self.take_language_unit() {
            Err(error @ Error::Syntax(SyntaxError::ExpressionTooLong { .. })) => Err(error),
            _ => Ok(self.expression),
        }
    }

    pub fn take(self) -> Expression {
        self.expression
    }

    pub fn is_valid(&self) -> bool {
        self.invalid.not()
    }

    ///
    /// A Language Unit may be a Function or a standalone Atom.
    #[inline(always)]
    fn take_language_unit(&mut self) -> Result<(), Error> {
        match self.next_token(2) {
            Some(t) => {
                match t {
                    "**" => {
                        self.add(Token::Bang, Atom::Bang)?;
                        return Ok(());
                    }
                    ">>" => {
                        self.add(Token::Activation, Atom::Activation(crate::Activation::East))?;
                        return Ok(());
                    }
                    _ => {}
                }
                let result = Function::try_from(t);

                let tokens = match result {
                    Ok(f) => {
                        self.add(Token::Function, Atom::from(f))?;
                        Tokens::from(&f)
                    }
                    Err(e) => {
                        if self.check_function().is_some() {
                            self.expression.add_invalid()?;
                        } else {
                            return Err(e);
                        }
                        Tokens::new()
                    }
                };

                for t in tokens {
                    if self.is_function_next() {
                        self.take_language_unit()?;
                    } else {
                        match self.take_token(&t) {
                            Ok(Some(atom)) => self.add(t, atom)?,
                            Ok(None) => {}
                            Err(error) if self.check => {
                                self.invalid = true;
                                self.expression.add_invalid()?;
                                return Err(error);
                            }
                            Err(error) => return Err(error),
                        }
                    }
                }
                Ok(())
            }
            None => {
                if self.check {
                    self.invalid = true;
                    self.expression.add_incomplete(Token::Char)?;
                    Ok(())
                } else {
                    Err(SyntaxError::ExpectedFunction.into())
                }
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
                if self.check_atom().is_some() {
                    self.expression.add_incomplete(*token)?;
                    return Ok(None);
                }
                return Err(Error::Syntax(SyntaxError::ExpectedToken));
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
        match self.source.len() {
            0 | 1 => None,
            _ => {
                let (next_token, _) = self.source.split_at(2);
                Some(next_token)
            }
        }
    }

    #[inline(always)]
    fn add(&mut self, t: Token, a: Atom) -> Result<(), Error> {
        self.expression.add(t, a)?;
        Ok(())
    }

    #[inline(always)]
    fn check_atom(&mut self) -> Option<Atom> {
        if self.check {
            self.invalid = true;
            Some(Atom::Empty)
        } else {
            None
        }
    }

    #[inline(always)]
    fn check_function(&mut self) -> Option<Function> {
        if self.check {
            self.invalid = true;
            Some(Function::Empty)
        } else {
            None
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
    use arrayvec::ArrayVec;

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    #[test]
    fn every_real_function_parses_and_renders_from_its_canonical_spelling() {
        assert!(!Function::ALL.contains(&Function::Empty));

        for function in Function::ALL {
            let spelling = function.spelling();

            assert_eq!(Function::try_from(spelling).unwrap(), *function);
            assert_eq!(function.to_string(), spelling);
        }
    }

    fn parse(exp: &mut str) -> Result<Vec<Atom>, Error> {
        let parser = Parser::from(exp);
        Ok(parser
            .parse()?
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
            .parse()
            .unwrap();
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
            .parse()
            .unwrap();
        assert_eq!(
            invalid.tokens().collect::<Vec<_>>(),
            vec![Token::Function, Token::Number, Token::Char]
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
            &[Atom::Function(Function::ConvertToNumber), Atom::Note(60)]
        );
        assert_eq!(
            try_parse(&mut ".^3C".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Number(60)]
        );
    }

    #[test]
    fn every_note_source_encoding_parses_in_context() {
        for value in 0x00..=0x7F {
            let note = Atom::Note(value).to_string();
            let mut source = format!(".v{note}");
            assert_eq!(
                try_parse(&mut source).unwrap().as_slice(),
                &[Atom::Function(Function::ConvertToNumber), Atom::Note(value)],
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
    fn bang_and_east_activation_parse_as_complete_language_units() {
        assert_eq!(
            try_parse(&mut "**".to_owned()).unwrap().as_slice(),
            &[Atom::Bang]
        );
        assert_eq!(
            try_parse(&mut ">>".to_owned()).unwrap().as_slice(),
            &[Atom::Activation(crate::Activation::East)]
        );
    }

    #[test]
    fn test_parse_play_function() {
        trace();

        let mut s = String::from("!>010AC4");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![
            Atom::Function(Function::Play),
            Atom::Number(1),
            Atom::Number(10),
            Atom::Note(60),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }
}
