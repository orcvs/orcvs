use crate::to_atom_note;
use crate::to_atom_num;
use crate::to_atom_string;
use crate::Atom;
use crate::Error;
use crate::Expression;
use crate::Expressions;
use crate::Function;
use crate::SyntaxError;
use crate::Token;
use crate::Tokens;
use arrayvec::ArrayVec;
use std::mem;
use std::ops::Not;

const DEFAULT_TOKEN_LEN: usize = 2;

const EMPTY: &str = "";

pub struct Parser<'a> {
    stack: Expressions,
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
    pub fn new() -> Self {
        Self {
            stack: Expressions::new(),
            source: EMPTY,
            check: false,
            invalid: false,
        }
    }

    ///
    /// try_parse will error if the parse fails
    ///
    pub fn try_parse(&mut self, source: &'a mut str) -> Result<(), Error> {
        self.source = source;
        self.take_function()
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///
    pub fn parse(&mut self, source: &'a mut str) -> Result<bool, Error> {
        self.check = true;
        self.invalid = false;
        self.try_parse(source)?;
        Ok(self.is_valid())
    }

    pub fn stack(&self) -> &Expressions {
        &self.stack
    }

    pub fn take_stack(&mut self) -> Expressions {
        mem::take(&mut self.stack)
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
        let token = self.next_token(2);

        match token {
            Some(t) => {
                let result = Function::try_from(t);

                let tokens = match result {
                    // Ok(Function::Id) => Tokens::from(&Function::Id),
                    Ok(f) => {
                        let tokens = Tokens::from(&f);
                        let exp = Expression::from(f);
                        self.add(exp);
                        tokens
                    }
                    Err(e) => {
                        self.check_function().ok_or(e)?;
                        Tokens::new()
                    }
                };

                for mut exp in tokens {
                    if self.is_function_next() {
                        self.take_function()?;
                    } else {
                        let atom = self.take_token(&exp)?;
                        exp.set_atom(atom);
                        self.add(exp);
                    }
                }
                Ok(())
            }
            None => self
                .check_atom()
                .and_then(|a| {
                    let mut exp = Expression::new(Token::Function);
                    exp.set_atom(a);
                    self.add(exp);

                    Some(())
                })
                .ok_or(SyntaxError::ExpectedFunction.into()),
        }
    }

    #[inline(always)]
    fn take_token(&mut self, exp: &Expression) -> Result<Atom, Error> {
        let token = exp.token;
        let count = match token {
            Token::Number1 => 1,
            _ => DEFAULT_TOKEN_LEN,
        };

        let t = self.next_token(count);
        let atom = match t {
            Some(s) => match token {
                Token::Note => to_atom_note(s)?,
                Token::Number => to_atom_num(s)?,
                Token::Number1 => to_atom_num(s)?,
                Token::String => to_atom_string(s)?,
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

    fn add(&mut self, exp: Expression) {
        self.stack.push(exp);
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

    fn check_function(&mut self) -> Option<Function> {
        if self.check {
            self.invalid = true;
            Some(Function::Empty)
        } else {
            None
        }
    }

    fn atoms(&self) -> ArrayVec<Atom, 32> {
        self.stack
            .iter()
            .filter_map(|exp| exp.atom.clone())
            .collect()
    }

    fn tokens(&self) -> ArrayVec<Token, 32> {
        self.stack.iter().map(|exp| exp.token).collect()
    }
}

// Inlining causes performance regression
fn is_function(s: Option<&str>) -> bool {
    s.filter(|t| Function::try_from(*t).is_ok()).is_some()
}

#[cfg(test)]
mod test {

    use arrayvec::ArrayVec;
    use tracing::info;

    use crate::{parser::Parser, trace, Atom, Error, Function, SyntaxError, Token, TypeError};

    fn try_parse_with_result(exp: &mut str) -> Result<(), Error> {
        let mut parser = Parser::new();
        parser.try_parse(exp)
    }

    fn try_parse(exp: &mut str) -> Parser {
        let mut parser = Parser::new();
        parser.try_parse(exp).unwrap();
        parser
    }

    fn parse(exp: &mut str) -> (bool, Parser) {
        let mut parser = Parser::new();
        let result = parser.parse(exp).unwrap();

        (result, parser)
    }

    macro_rules! array_vec {
        ($($items:tt),*) => {
            {
                let mut ary = ArrayVec::new();

                $(
                    for item in $items.iter() {
                        ary.push(item.clone());
                    }
                )*
                ary
            }
        };
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let mut s = String::from("++");
        let (success, parser) = parse(&mut s);

        assert!(!success); // expression is invalid

        let stack = array_vec!([Atom::Function(Function::Add), Atom::Empty, Atom::Empty]);
        let result = parser.atoms();
        assert_eq!(result, stack);

        let mut s = String::from("");
        let (success, parser) = parse(&mut s);

        assert!(!success); // expression is invalid

        let stack = array_vec!([Atom::Empty]);

        let result = parser.atoms();
        assert_eq!(result, stack);
    }

    #[test]
    fn test_try_parse_with_invalid() {
        trace();

        let mut s = String::from("id");
        let result = try_parse_with_result(&mut s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Syntax(SyntaxError::ExpectedToken)));
    }

    #[test]
    fn test_with_bad_syntax() {
        trace();

        let mut s = String::from("++01XY");
        let result = try_parse_with_result(&mut s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = String::from("idFA");
        let parser = try_parse(&mut s);

        let expected = array_vec!([Atom::Function(Function::Id), Atom::String("FA".to_string()),]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);
    }

    #[test]
    fn test_parse_play_function() {
        trace();

        let mut s = String::from(">>10AC4");
        let parser = try_parse(&mut s);

        let expected = array_vec!([
            Atom::Function(Function::Play),
            Atom::Number(1),
            Atom::Number(10),
            Atom::Note(60),
        ]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);
    }

    #[test]
    fn test_with_function_parameter() {
        trace();

        let mut s = String::from("++id0Aid01");

        let parser = try_parse(&mut s);

        let expected = array_vec!([
            Token::Function,
            Token::Function,
            Token::String,
            Token::Function,
            Token::String,
        ]);

        let result = parser.tokens();
        assert_eq!(result, expected);

        let expected = array_vec!([
            Atom::Function(Function::Add),
            Atom::Function(Function::Id),
            Atom::String("0A".to_string()),
            Atom::Function(Function::Id),
            Atom::String("01".to_string()),
        ]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);

        let mut s = String::from("++id0A01");
        let parser = try_parse(&mut s);

        let expected = array_vec!([
            Atom::Function(Function::Add),
            Atom::Function(Function::Id),
            Atom::String("0A".to_string()),
            Atom::Number(1),
        ]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let mut s = String::from("idididAA");
        let parser = try_parse(&mut s);

        let expected = array_vec!([
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::String("AA".to_string()),
        ]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);

        let mut s = String::from("++idididAA01");
        let parser = try_parse(&mut s);

        let expected = array_vec!([
            Token::Function,
            Token::Function,
            Token::Function,
            Token::Function,
            Token::String,
            Token::Number,
        ]);

        let result = parser.tokens();
        assert_eq!(result, expected);

        let expected = array_vec!([
            Atom::Function(Function::Add),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::Function(Function::Id),
            Atom::String("AA".to_string()),
            Atom::Number(1),
        ]);

        let stack = parser.atoms();
        assert_eq!(stack, expected);
    }
}
