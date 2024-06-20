use crate::midi_note_to_number;
use crate::to_atom_note;
use crate::to_atom_num;
use crate::to_atom_string;
use crate::Atom;
use crate::Error;
use crate::Function;
use crate::SyntaxError;
use crate::TypeError;
use arrayvec::ArrayVec;
use tracing::info;

const DEFAULT_TOKEN_LEN: usize = 2;

pub type Stack = ArrayVec<Atom, 48>;

pub struct Parser<'a> {
    stack: Stack,
    source: &'a str,
    take_next: usize,
    check: bool,
    valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Note,
    Number,
    String,
    // Number(u8),
}
type T = Token;

type Tokens = ArrayVec<Token, 16>;

// #[derive(Debug, Clone, PartialEq)]
// struct Tokens {
//     inner: ArrayVec<Token, 16>,
// }

macro_rules! map_from {
    ($($items:tt),*) => {
        {
            let mut ary = ArrayVec::new();
            $(
                for item in $items.iter() {
                    ary.push(*item);
                }
            )*
            ary
        }
    };
}

#[must_use]
fn tokens_for(f: &Function) -> Tokens {
    match f {
        Function::Add => map_from!([T::Number, T::Number]),
        Function::Id => map_from!([T::String]),
        _ => map_from!(([])),
    }
}

// pub struct Tokenizer<'a> {
//     working_stack: ArrayVec<Atom, 8>,
//     source: &'a str,
//     take_next: usize,
//     check: bool,
//     valid: bool,
// }

///
///
/// #[inline(always)]
/// Inline on take_function and inner_take improves performance by 5%
/// Additional inlines do not improve performance
///
///
impl<'a> Parser<'a> {
    pub fn new(source: &'a mut str) -> Self {
        Self {
            stack: ArrayVec::new(),
            take_next: DEFAULT_TOKEN_LEN,
            source,
            check: false,
            valid: true,
        }
    }

    ///
    /// try_parse will error if the parse fails
    ///
    pub fn try_parse(&mut self) -> Result<(), Error> {
        let _a = self.take_function()?;
        // self.stack.push(a);
        Ok(())
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///
    pub fn parse(&mut self) -> Result<bool, Error> {
        self.check = true;
        self.try_parse()?;
        Ok(self.valid)
    }

    pub fn take(&mut self, count: usize) -> &mut Self {
        self.take_next = count;
        self
    }

    ///
    /// next takes the next token
    /// unless preceded by take with a count, will use the default TOKEN_SIZE
    /// exists mostly to provide symetry with take
    /// pool.take(2).as_string();
    /// pool.next().as_string();
    ///
    pub fn next(&mut self) -> &mut Self {
        self.take_next = DEFAULT_TOKEN_LEN; // reset the token count
        self
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
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
    ///   #[inline(always)]

    #[inline(always)]
    fn take_function(&mut self) -> Result<(), Error> {
        let token = self.next_token(2);

        match token {
            Some(t) => {
                let result = Function::try_from(t);
                let f = match result {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(e);
                        // return self.check_function().ok_or(e);
                    }
                };

                info!("{:?}", f);

                let tokens = tokens_for(&f);

                info!("{:?}", tokens);

                self.stack.push(f.into());
                // let mut stack = Stack::new();

                for token in tokens {
                    if self.is_function_next() {
                        let f_stack = self.take_function()?;
                        info!("f_stack {:?}", f_stack);
                        stack.extend(f_stack.into_iter());
                    } else {
                        let a = self.take_token(token)?;
                        stack.push(a);
                    }
                }

                info!("stack {:?}", stack);

                // let a = Atom::from(f);
                // info!("{:?}", a);
                // self.stack.push(a.clone());

                // let result = tokens
                //     .into_iter()
                //     .map(|t| self.take_token(t))
                //     .collect::<Result<Stack, _>>();

                // match result {
                //     Ok(mut atoms) => {
                //         atoms.insert(0, a.clone());

                //         info!("{:?}", atoms);

                //         self.stack.extend(atoms.into_iter())
                //     }
                //     Err(e) => return Err(e),
                // }
                // info!("{:?}", self.stack);

                Ok(())
            }
            None => {
                // self
                // .check_atom()
                // .ok_or(SyntaxError::ExpectedFunction.into())
                return Err(SyntaxError::ExpectedFunction.into());
            }
        }
    }

    fn take_token(&mut self, token: Token) -> Result<Atom, Error> {
        let count = DEFAULT_TOKEN_LEN;
        let s = self.next_token(count);

        match s {
            Some(t) => {
                match token {
                    Token::Note => {
                        let a = to_atom_note(s)?;
                        Ok(a)
                    }
                    Token::Number => {
                        let a = to_atom_num(s)?;
                        Ok(a)
                    }
                    Token::String => {
                        let a = to_atom_string(s)?;
                        Ok(a)
                    }
                }

                Ok(a)
            }
            None => self.check_atom().ok_or(SyntaxError::ExpectedToken.into()),
        }
    }

    #[inline(always)]
    fn inner_take<F>(&mut self, atomizer: F) -> Result<Atom, Error>
    where
        F: FnOnce(&str) -> Result<Atom, Error>,
    {
        let token = self.peek_next();

        if is_function(token) {
            self.take_function()
        } else {
            self.take_atom(atomizer)
        }
    }

    #[inline(always)]
    fn take_atom<F>(&mut self, atomizer: F) -> Result<Atom, Error>
    where
        F: FnOnce(&str) -> Result<Atom, Error>,
    {
        let count = self.take_next;

        let t = self.next_token(count);
        self.take_next = DEFAULT_TOKEN_LEN; // reset the token count
        match t {
            Some(t) => {
                let a = atomizer(t)?;
                Ok(a)
            }
            None => self.check_atom().ok_or(SyntaxError::ExpectedToken.into()),
        }
    }

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

    fn is_function_next(&self) -> bool {
        let peek = self.peek_next();
        is_function(peek)
    }

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
        info!("add: {:?}", a);
        // self.stack.push(a);
        // self.working_stack
    }

    fn check<T, F>(&mut self, atomizer: F) -> Option<Atom>
    where
        F: FnOnce() -> T,
        T: Into<Atom>,
    {
        if self.check {
            let a = atomizer();
            self.valid = false;
            Some(a.into())
        } else {
            None
        }
    }

    fn check_atom(&mut self) -> Option<Atom> {
        self.check(|| Atom::Empty)
    }

    fn check_function(&mut self) -> Option<Atom> {
        self.check(|| Function::Empty)
    }

    fn start(&mut self) -> Result<Atom, Error> {
        let token = self.next_token(2);

        match token {
            Some(t) => {
                let result = Function::try_from(t);

                let f = match result {
                    Ok(f) => f,
                    Err(e) => {
                        return self.check_function().ok_or(e);
                    }
                };
                info!("{:?}", f);

                let tokens = tokens_for(&f);

                let a = Atom::from(f);

                let result = tokens
                    .into_iter()
                    .map(|t| self.take_token(t))
                    .collect::<Result<Stack, _>>();

                match result {
                    Ok(mut atoms) => {
                        atoms.insert(0, a.clone());

                        info!("{:?}", atoms);

                        self.stack.extend(atoms.into_iter())
                    }
                    Err(e) => return Err(e),
                }

                info!("{:?}", self.stack);

                Ok(a)
            }
            None => self
                .check_atom()
                .ok_or(SyntaxError::ExpectedFunction.into()),
        }
    }
}

fn is_function(s: Option<&str>) -> bool {
    s.filter(|t| Function::try_from(*t).is_ok()).is_some()
}

fn add(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_num()?;
    pool.next().as_num()?;
    Ok(Function::Add)
}

fn subtract(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_num()?;
    pool.next().as_num()?;
    Ok(Function::Subtract)
}

fn multiply(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_num()?;
    pool.next().as_num()?;
    Ok(Function::Multiply)
}

fn divide(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_num()?;
    pool.next().as_num()?;
    Ok(Function::Divide)
}

fn id(pool: &mut Parser) -> Result<Function, Error> {
    info!("ident====");
    pool.next().as_string()?;
    Ok(Function::Id)
}

fn play(pool: &mut Parser) -> Result<Function, Error> {
    pool.take(1).as_num()?;
    pool.next().as_num()?;
    pool.next().as_note()?;
    Ok(Function::Play)
}

#[cfg(test)]
mod test {

    use tracing::{info, span::Id};

    use crate::{parser::Parser, trace, Atom, Error, Function, SyntaxError, TypeError};

    use super::Stack;

    fn try_parse_with_result(exp: String) -> Result<(), Error> {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse()
    }

    fn try_parse(exp: String) -> Stack {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse().unwrap();
        parser.stack().clone()
    }

    fn parse(exp: String) -> (bool, Stack) {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        let result = parser.parse().unwrap();
        (result, parser.stack().clone())
    }

    fn stack_from(array: &[Atom]) -> Stack {
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

        let array: &[Atom] = &[Atom::Empty, Atom::Empty, Atom::Function(Function::Add)];
        let stack: Stack = stack_from(array);

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

        let s = String::from("idAA");
        let pool = try_parse(s);

        let expected = Atom::String("AA".to_string());
        assert_eq!(pool[0], expected);

        let expected = Atom::Function(Function::Id);
        assert_eq!(pool[1], expected);
    }

    #[test]
    fn test_with_function_parameter() {
        trace();

        let s = String::from("++id0Aid01");
        info!("exp {:?}", s);

        let pool = try_parse(s);

        info!("pool {:?}", pool);

        // Add
        //     take_function
        //         Id
        //             take_next
        //                 0A
        //     take_next
        //         0A

        // [String("0A"), Function(Id), Number(10), Function(Add)] }

        let expected = Atom::Number(10);
        assert_eq!(pool[0], expected);

        let expected = Atom::String("0A".to_string());
        assert_eq!(pool[0], expected);

        let expected = Atom::Function(Function::Id);
        assert_eq!(pool[1], expected);

        let expected = Atom::Function(Function::Add);
        assert_eq!(pool[2], expected);

        // let s = String::from("++0Aid0A");
        // let pool = try_parse(s);

        // let expected = Atom::String("0A".to_string());
        // assert_eq!(pool[0], expected);

        // let expected = Atom::Number(10);
        // assert_eq!(pool[0], expected);

        // let expected = Atom::Function(Function::Id);
        // assert_eq!(pool[1], expected);

        // let expected = Atom::Function(Function::Add);
        // assert_eq!(pool[2], expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let s = String::from("idididAA");
        let pool = try_parse(s);

        let expected = Atom::String("AA".to_string());
        assert_eq!(pool[0], expected);

        let expected = Atom::Function(Function::Id);
        assert_eq!(pool[1], expected);

        let expected = Atom::Function(Function::Id);
        assert_eq!(pool[2], expected);

        let expected = Atom::Function(Function::Id);
        assert_eq!(pool[3], expected);
    }
}
