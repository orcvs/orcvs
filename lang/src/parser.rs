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
        let atoms = self.expression.take_atoms();
        Ok(atoms.into_iter().collect())
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
                        self.check_function().ok_or(e)?;
                        Tokens::new()
                    }
                };

                for t in tokens {
                    if self.is_function_next() {
                        self.take_language_unit()?;
                    } else {
                        let a = self.take_token(&t)?;
                        let typed_token = match (t, a) {
                            (Token::NumericToNote | Token::NumericToNumber, Atom::Note(_)) => {
                                Token::Note
                            }
                            (Token::NumericToNote | Token::NumericToNumber, Atom::Number(_)) => {
                                Token::Number
                            }
                            _ => t,
                        };
                        self.add(typed_token, a)?;
                    }
                }
                Ok(())
            }
            None => {
                if self.check {
                    Ok(())
                } else {
                    Err(SyntaxError::ExpectedFunction.into())
                }
            }
        }
    }

    #[inline(always)]
    fn take_token(&mut self, token: &Token) -> Result<Atom, Error> {
        let t = self.next_token(token.len());
        let atom = match t {
            Some(s) => match token {
                Token::Note => to_atom_note(s)?,
                // A conversion Function accepts either member of the
                // Number/Note pair, so unlike every other signature its own
                // signature cannot tell an overlapping hexadecimal spelling
                // from a Note spelling. Resolve the overlap towards the Note:
                // every Cell pair valid in both domains begins with a pitch
                // letter and so reads as `A0` or greater, outside the `00`-`7F`
                // domain `.^` converts. Reading it as a Number could therefore
                // only ever diagnose, while the explicit `.v C4` / `.^ 3C`
                // source forms are unaffected.
                Token::NumericToNote | Token::NumericToNumber => {
                    to_atom_note(s).or_else(|_| to_atom_num(s))?
                }
                Token::Number => to_atom_num(s)?,
                Token::NumberN(_) => to_atom_num(s)?,
                Token::Char => to_atom_char(s)?,
                Token::Activation | Token::Bang | Token::Function => unreachable!(),
            },
            None => self
                .check_atom()
                .ok_or(Error::Syntax(SyntaxError::ExpectedToken))?,
        };

        Ok(atom)
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

    use crate::{Atom, Atoms, Error, Function, SyntaxError, TypeError, parser::Parser, trace};
    use arrayvec::ArrayVec;

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    fn parse(exp: &mut str) -> Result<Vec<Atom>, Error> {
        let parser = Parser::from(exp);
        let a = parser.parse()?.take_atoms();
        Ok(a.into_iter().collect())
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let mut s = String::from(".+");
        let parsed = parse(&mut s).unwrap();

        let stack = vec![Atom::Function(Function::Add), Atom::Empty, Atom::Empty];
        assert_eq!(parsed, stack);

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
        assert_eq!(
            try_parse(&mut ".v3C".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNumber), Atom::Number(60)]
        );
        assert_eq!(
            try_parse(&mut ".^G9".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Note(127)]
        );
    }

    #[test]
    fn conversion_operands_resolve_an_overlapping_spelling_as_a_note() {
        // Every Cell pair valid in both domains begins with a pitch letter, so
        // its hexadecimal reading is always `A0` or greater and could only ever
        // diagnose as a `.^` operand. Both conversions therefore read it as a
        // Note, which `.^` returns unchanged and `.v` lowers.
        assert_eq!(
            try_parse(&mut ".^C4".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Note(60)]
        );
        assert_eq!(
            try_parse(&mut ".^a0".to_owned()).unwrap().as_slice(),
            &[Atom::Function(Function::ConvertToNote), Atom::Note(22)]
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

        let mut s = String::from("!>10AC4");
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
