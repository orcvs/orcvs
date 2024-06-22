use crate::to_atom_note;
use crate::to_atom_num;
use crate::to_atom_string;
use crate::Atom;
use crate::Error;
use crate::Function;
use crate::Stack;
use crate::SyntaxError;
use crate::Token;
use crate::Tokens;
use std::mem;
use std::ops::Not;

const DEFAULT_TOKEN_LEN: usize = 2;

pub struct Parser<'a> {
    stack: Stack<32>,
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
    pub fn new(source: &'a mut str) -> Self {
        Self {
            stack: Stack::new(),
            source,
            check: false,
            invalid: false,
        }
    }

    ///
    /// try_parse will error if the parse fails
    ///
    pub fn try_parse(&mut self) -> Result<(), Error> {
        self.take_function()
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///
    pub fn parse(&mut self) -> Result<bool, Error> {
        self.check = true;
        self.invalid = false;
        self.try_parse()?;
        Ok(self.is_valid())
    }

    pub fn stack(&self) -> &Stack<32> {
        &self.stack
    }

    pub fn take_stack(&mut self) -> Stack<32> {
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
                let f = match result {
                    Ok(f) => f,
                    Err(e) => self.check_function().ok_or(e)?,
                };

                let tokens = Tokens::from(&f);

                match f {
                    Function::Id => (),
                    _ => self.add(f),
                };

                for token in tokens {
                    if self.is_function_next() {
                        self.take_function()?;
                    } else {
                        let a = self.take_token(token)?;
                        self.add(a);
                    }
                }
                Ok(())
            }
            None => self
                .check_atom()
                .and_then(|a| Some(self.add(a)))
                .ok_or(SyntaxError::ExpectedFunction.into()),
        }
    }

    #[inline(always)]
    fn take_token(&mut self, token: Token) -> Result<Atom, Error> {
        let count = match token {
            Token::Number1 => 1,
            _ => DEFAULT_TOKEN_LEN,
        };

        let t = self.next_token(count);

        match t {
            Some(s) => match token {
                Token::Note => {
                    let a = to_atom_note(s)?;
                    Ok(a)
                }
                Token::Number | Token::Number1 => {
                    let a = to_atom_num(s)?;
                    Ok(a)
                }

                Token::String => {
                    let a = to_atom_string(s)?;
                    Ok(a)
                }
            },
            None => self.check_atom().ok_or(SyntaxError::ExpectedToken.into()),
        }
    }

    #[inline(always)]
    fn next_token(&mut self, count: usize) -> Option<&'a str> {
        match self.source.len() {
            0 | 1 => None,
            _ => {
                let (next_token, rest) = self.source.split_at(count);
                self.source = rest;
                Some(next_token)
            }
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

    fn add<A>(&mut self, atom: A)
    where
        A: Into<Atom>,
    {
        let a = atom.into();
        self.stack.push(a);
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
}

// Inlining causes performance regression
fn is_function(s: Option<&str>) -> bool {
    s.filter(|t| Function::try_from(*t).is_ok()).is_some()
}

#[cfg(test)]
mod test {

    use crate::{parser::Parser, trace, Atom, Error, Function, Stack, SyntaxError, TypeError};

    fn try_parse_with_result(exp: String) -> Result<(), Error> {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse()
    }

    fn try_parse(exp: String) -> Stack<32> {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse().unwrap();
        parser.stack().clone()
    }

    fn parse(exp: String) -> (bool, Stack<32>) {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        let result = parser.parse().unwrap();
        (result, parser.stack().clone())
    }

    fn stack_from(array: &[Atom]) -> Stack<32> {
        let mut stack = Stack::new();
        for a in array {
            stack.push(a.clone());
        }
        stack
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let s = String::from("++");
        let (success, result) = parse(s);

        let array: &[Atom] = &[Atom::Function(Function::Add), Atom::Empty, Atom::Empty];
        let stack = stack_from(array);

        assert!(!success); // expression is invalid
        assert_eq!(result, stack);

        let s = String::from("");
        let (success, result) = parse(s);

        let array: &[Atom] = &[Atom::Empty];
        let stack = stack_from(array);

        assert!(!success); // expression is invalid
        assert_eq!(result, stack);
    }

    #[test]
    fn test_try_parse_with_invalid() {
        trace();

        let s = String::from("id");
        let result = try_parse_with_result(s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Syntax(SyntaxError::ExpectedToken)));
    }

    #[test]
    fn test_with_bad_syntax() {
        trace();

        let s = String::from("++01XY");
        let result = try_parse_with_result(s);

        let error = result.unwrap_err();
        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    #[test]
    fn test_parse_id_function() {
        trace();

        let s = String::from("idFA");
        let stack = try_parse(s);

        let expected = stack_from(&[
            // Atom::Function(Function::Id),
            Atom::String("FA".to_string()),
        ]);

        assert_eq!(stack, expected);
    }

    #[test]
    fn test_parse_play_function() {
        trace();

        let s = String::from(">>10AC4");
        let stack = try_parse(s);

        let expected = stack_from(&[
            Atom::Function(Function::Play),
            Atom::Number(1),
            Atom::Number(10),
            Atom::Note(60),
        ]);

        assert_eq!(stack, expected);
    }

    #[test]
    fn test_with_function_parameter() {
        trace();

        let s = String::from("++id0Aid01");

        let stack = try_parse(s);

        let expected = stack_from(&[
            Atom::Function(Function::Add),
            // Atom::Function(Function::Id),
            Atom::String("0A".to_string()),
            // Atom::Function(Function::Id),
            Atom::String("01".to_string()),
        ]);

        assert_eq!(stack, expected);

        let s = String::from("++id0A01");

        let stack = try_parse(s);
        let expected = stack_from(&[
            Atom::Function(Function::Add),
            // Atom::Function(Function::Id),
            Atom::String("0A".to_string()),
            Atom::Number(1),
        ]);

        assert_eq!(stack, expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let s = String::from("idididAA");

        let stack = try_parse(s);
        let expected = stack_from(&[
            // Atom::Function(Function::Id),
            // Atom::Function(Function::Id),
            // Atom::Function(Function::Id),
            Atom::String("AA".to_string()),
        ]);

        assert_eq!(stack, expected);

        let s = String::from("++idididAA01");

        let stack = try_parse(s);
        let expected = stack_from(&[
            Atom::Function(Function::Add),
            // Atom::Function(Function::Id),
            // Atom::Function(Function::Id),
            // Atom::Function(Function::Id),
            Atom::String("AA".to_string()),
            Atom::Number(1),
        ]);

        assert_eq!(stack, expected);
    }
}
