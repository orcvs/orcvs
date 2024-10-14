use crate::atom::to_atom_char;
use crate::to_atom_note;
use crate::to_atom_num;
use crate::Atom;
use crate::Atoms;
use crate::Error;
use crate::Expression;
use crate::Function;
use crate::SyntaxError;
use crate::Token;
use crate::Tokens;
use std::ops::Not;

pub struct Parser<'a> {
    expression: Expression,
    source: &'a str,
    check: bool,
    invalid: bool,
}

///
/// #[inline(always)]
/// Inline on take_function and inner_take improves performance by 5%
/// Additional inlines do not improve performance
///
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
        self.take_function()?;
        let atoms = self.expression.take_atoms();
        Ok(atoms.into_iter().collect())
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///
    pub fn parse(mut self) -> Expression {
        self.check = true;
        self.invalid = false;
        let _ = self.take_function();
        self.expression
    }

    pub fn take(self) -> Expression {
        self.expression
    }

    pub fn is_valid(&self) -> bool {
        self.invalid.not()
    }

    ///
    /// Functions are slightly magical
    /// A function can be evaluated as any other Atom type
    /// Although take_function always retruns an AtomRef<AtomFunction>,
    ///     making it generic over <T> means that the return value will
    ///     be handled as the expected type.
    ///
    /// eg
    ///     Given the expression "adad010101"
    ///     parser.next().as_num() will parse "ad0101" and handle it as the expected num type
    ///
    #[inline(always)]
    fn take_function(&mut self) -> Result<(), Error> {
        match self.next_token(2) {
            Some(t) => {
                let result = Function::try_from(t);

                let tokens = match result {
                    Ok(f) => {
                        self.add(Token::Function, Atom::from(f));
                        Tokens::from(&f)
                    }
                    Err(e) => {
                        self.check_function().ok_or(e)?;
                        Tokens::new()
                    }
                };

                for t in tokens {
                    if self.is_function_next() {
                        self.take_function()?;
                    } else {
                        let a = self.take_token(&t)?;
                        self.add(t, a);
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
                Token::Number => to_atom_num(s)?,
                Token::NumberN(_) => to_atom_num(s)?,
                Token::Char => to_atom_char(s)?,
                Token::Function => unreachable!(),
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

    // Inlining causes performance regression
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
    fn add(&mut self, t: Token, a: Atom) {
        self.expression.add(t, a);
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

// Inlining causes performance regression
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

    use crate::{parser::Parser, trace, Atom, Atoms, Error, Function, SyntaxError, TypeError};
    use arrayvec::ArrayVec;
    use tracing::error;

    fn try_parse(exp: &mut str) -> Result<Atoms, Error> {
        let parser = Parser::from(exp);
        parser.try_parse()
    }

    fn parse(exp: &mut str) -> Result<Vec<Atom>, Error> {
        let parser = Parser::from(exp);
        let a = parser.parse().take_atoms();
        Ok(a.into_iter().collect())
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let mut s = String::from("++");
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

        // let mut s = String::from("id");
        // let result = try_parse(&mut s);

        // let error = result.unwrap_err();
        // assert!(matches!(error, Error::Syntax(SyntaxError::ExpectedToken)));

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

        let mut s = String::from("++01XY");
        let result = try_parse(&mut s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = String::from("idA");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![Atom::Function(Function::Id), Atom::Char('A')];
        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_play_function() {
        trace();

        let mut s = String::from(">>10AC4");
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

    #[test]
    fn test_with_function_parameter() {
        trace();

        let mut s = String::from("++id0Aid01");

        let parsed = try_parse(&mut s);

        error!("{:?}", parsed);
        // let expected = vec!([
        //     Token::Function,
        //     Token::Function,
        //     Token::String,
        //     Token::Function,
        //     Token::String,
        // ]);

        // let result = parser.take_tokens();
        // assert_eq!(result, expected);

        // let v = vec![
        //     Atom::Function(Function::Add),
        //     Atom::Function(Function::Id),
        //     Atom::String("0A".to_string()),
        //     Atom::Function(Function::Id),
        //     Atom::String("01".to_string()),
        // ];
        // let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        // v.into_iter().for_each(|a| expected.push(a));

        // assert_eq!(parsed, expected);

        // let mut s = String::from("++id0A01");
        // let parsed = try_parse(&mut s).unwrap();

        // let v = vec![
        //     Atom::Function(Function::Add),
        //     Atom::Function(Function::Id),
        //     Atom::String("0A".to_string()),
        //     Atom::Number(1),
        // ];
        // let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        // v.into_iter().for_each(|a| expected.push(a));

        // assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let mut s = String::from("idididA");
        let parsed = try_parse(&mut s).unwrap();

        let v = vec![
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Char('A'),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);

        let mut s = String::from("++idididA01");
        let parsed = try_parse(&mut s).unwrap();
        // assert_eq!(result, expected);

        let v = vec![
            Atom::Function(Function::Add),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Char('A'),
            Atom::Number(1),
        ];

        let mut expected: ArrayVec<Atom, 32> = ArrayVec::new();
        v.into_iter().for_each(|a| expected.push(a));

        assert_eq!(parsed, expected);
    }
}
