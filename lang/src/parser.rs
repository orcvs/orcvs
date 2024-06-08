use crate::midi_note_to_number;
use crate::new_stack;
use crate::Atom;
use crate::Error;
use crate::Function;
use crate::SyntaxError;
use crate::TypeError;
use crate::VecStack;

const DEFAULT_TOKEN_LEN: usize = 2;

pub struct Parser<'a> {
    pub pool: VecStack,
    source: &'a str,
    take_next: usize,
    check: bool,
    valid: bool,
}

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
            pool: new_stack(),
            take_next: DEFAULT_TOKEN_LEN,
            source,
            check: false,
            valid: false,
        }
    }

    ///
    /// try_parse will error if the parse fails
    ///

    pub fn try_parse(&mut self) -> Result<(), Error> {
        self.check = false;
        self.valid = true; // try_parse is going to return errors
        self.take_function()
    }

    ///
    /// parse will return a boolean indicating success or failure
    ///

    pub fn parse(&mut self) -> Result<bool, Error> {
        self.check = true;
        self.valid = true;
        self.take_function()?;
        Ok(self.valid)
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
    ///
    #[inline(always)]
    pub fn take_function(&mut self) -> Result<(), Error> {
        let token = self.next_token(2);

        let functionizer: fn(&mut Parser) -> Result<Function, Error> = match token {
            Some("++") => add,
            Some("//") => divide,
            Some("**") => multiply,
            Some("--") => subtract,
            Some("id") => ident,
            Some("pl") => play,
            Some(s) => {
                return self
                    .check_function()
                    .ok_or(SyntaxError::UnknownFunction(s.to_string()).into());
            }
            None => {
                return self.check().ok_or(SyntaxError::ExpectedFunction.into());
            }
        };

        let a = functionizer(self)?;
        self.add(a);
        Ok(())
    }

    #[inline(always)]
    pub fn inner_take<F>(&mut self, atomizer: F) -> Result<(), Error>
    where
        F: FnOnce(&str) -> Result<Atom, Error>,
    {
        let count = self.take_next;
        let token = self.peek_next();

        if token.filter(|t| is_function(t)).is_some() {
            self.take_function()
        } else {
            let t = self.next_token(count);
            self.take_next = DEFAULT_TOKEN_LEN; // reset the token count
            match t {
                Some(t) => {
                    let a = atomizer(t)?;
                    self.add(a);
                    Ok(())
                }
                None => self.check().ok_or(SyntaxError::ExpectedToken.into()),
            }
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
        self.pool.push(atom.into());
    }

    fn check_and_add<T, F>(&mut self, atomizer: F) -> Option<()>
    where
        F: FnOnce() -> T,
        T: Into<Atom>,
    {
        if self.check {
            let a = atomizer();
            self.add(a.into());
            self.valid = false;
            Some(())
        } else {
            None
        }
    }

    fn check(&mut self) -> Option<()> {
        self.check_and_add(|| Atom::Empty)
    }

    fn check_function(&mut self) -> Option<()> {
        self.check_and_add(|| Function::Empty)
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

    pub fn as_string(&mut self) -> Result<(), Error> {
        self.inner_take(|s| {
            let a = Atom::String(s.to_string());
            Ok(a)
        })
    }

    pub fn as_note(&mut self) -> Result<(), Error> {
        self.inner_take(|s| match midi_note_to_number(&s) {
            Some(n) => {
                let a = Atom::Note(n);
                Ok(a)
            }
            None => Err(TypeError::Note(s.to_string()).into()),
        })
    }

    pub fn as_num(&mut self) -> Result<(), Error> {
        self.inner_take(|s| match u8::from_str_radix(&s, 16) {
            Ok(n) => {
                let a = Atom::Number(n);
                Ok(a)
            }
            Err(_) => Err(TypeError::Number(s.to_string()).into()),
        })
    }
}

fn is_function(s: &str) -> bool {
    matches!(s, "++" | "//" | "**" | "--" | "id" | "pl")
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

fn ident(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_string()?;
    Ok(Function::Ident)
}

fn play(pool: &mut Parser) -> Result<Function, Error> {
    pool.take(1).as_num()?;
    pool.next().as_num()?;
    pool.next().as_note()?;
    Ok(Function::Play)
}

#[cfg(test)]
mod test {

    use crate::{
        parser::Parser, test::stack_from, trace, Atom, Error, Function, SyntaxError, TypeError,
        VecStack,
    };

    fn try_parse_with_result(exp: String) -> Result<(), Error> {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse()
    }

    fn try_parse(exp: String) -> VecStack {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse().unwrap();
        parser.pool
    }

    fn parse(exp: String) -> (bool, VecStack) {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        let result = parser.parse().unwrap();
        (result, parser.pool)
    }

    #[test]
    fn test_parse_with_invalid() {
        trace();

        let s = String::from("++");
        let (success, result) = parse(s);

        let stack = stack_from(&[Atom::Empty, Atom::Empty, Atom::Function(Function::Add)]);

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

        let expected = Atom::Function(Function::Ident);
        assert_eq!(pool[1], expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let s = String::from("idididAA");
        let pool = try_parse(s);

        let expected = Atom::String("AA".to_string());
        assert_eq!(pool[0], expected);

        let expected = Atom::Function(Function::Ident);
        assert_eq!(pool[1], expected);

        let expected = Atom::Function(Function::Ident);
        assert_eq!(pool[2], expected);

        let expected = Atom::Function(Function::Ident);
        assert_eq!(pool[3], expected);
    }
}
