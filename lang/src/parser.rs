use crate::midi_note_to_number;
use crate::ArgumentError;
use crate::Atom;
use crate::Error;
use crate::Function;
use crate::SyntaxError;

const DEFAULT_TOKEN_LEN: usize = 2;

pub struct Parser<'a> {
    pub pool: Vec<Atom>,
    source: &'a str,
    take_next: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a mut str) -> Self {
        Self {
            pool: Vec::new(),
            take_next: DEFAULT_TOKEN_LEN,
            source,
        }
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        self.take_function()
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
            None => Err(ArgumentError::NoteExpected(s.to_string()).into()),
        })
    }

    pub fn as_num(&mut self) -> Result<(), Error> {
        self.inner_take(|s| match u8::from_str_radix(&s, 16) {
            Ok(n) => {
                let a = Atom::Number(n);
                Ok(a)
            }
            Err(_) => Err(ArgumentError::NumberExpected(s.to_string()).into()),
        })
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
    pub fn take_function(&mut self) -> Result<(), Error> {
        let token = self.next_token(2);

        let functionizer: fn(&mut Parser) -> Result<Function, Error> = match token {
            Some("++") => add,
            Some("pl") => play,
            Some("id") => ident,

            Some(s) => {
                return Err(Error::SyntaxError(SyntaxError::UnknownFunction {
                    f: s.to_string(),
                }));
            }
            None => {
                return Err(Error::SyntaxError(SyntaxError::ExpectedFunction {}));
            }
        };

        let a = functionizer(self)?;
        self.add(a);
        Ok(())
    }

    // #[inline(always)]
    // pub fn get<T: Into<usize>>(&self, atom_ref: T) -> &Atom {
    //     let index: usize = atom_ref.into();
    //     &self.pool[index]
    // }

    #[inline(always)]
    fn add<A>(&mut self, atom: A)
    where
        A: Into<Atom>,
    {
        self.pool.push(atom.into());
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

    pub fn inner_take<F>(&mut self, atomizer: F) -> Result<(), Error>
    where
        F: Fn(&str) -> Result<Atom, Error>,
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
                None => Err(Error::SyntaxError(SyntaxError::ExpectedToken {})),
            }
        }
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

    // pub fn take_pool(&mut self) -> Vec<Atom> {
    //     std::mem::take(&mut self.pool)
    // }
}

#[inline(always)]
fn is_function(s: &str) -> bool {
    matches!(s, "pl" | "id")
}

fn add(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_num()?;
    pool.next().as_num()?;
    Ok(Function::Add)
}

fn ident(pool: &mut Parser) -> Result<Function, Error> {
    pool.next().as_string()?;
    Ok(Function::Ident)
}

// channel, velocity, note
fn play(pool: &mut Parser) -> Result<Function, Error> {
    pool.take(1).as_num()?;
    pool.next().as_num()?;
    pool.next().as_note()?;
    Ok(Function::Play)
}

#[cfg(test)]
mod test {

    // use tracing::info;

    use crate::{parser::Parser, trace, Atom, Function};

    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = String::from("idAA");
        let mut parser = Parser::new(&mut s);
        parser.parse();

        let expected = Atom::String("AA".to_string());
        assert_eq!(parser.pool[0], expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let mut s = String::from("idididAA");
        let mut parser = Parser::new(&mut s);
        parser.parse();

        let expected = Atom::String("AA".to_string());
        assert_eq!(parser.pool[0], expected);

        let expected = Atom::Function(Function::Ident);
        assert_eq!(parser.pool[1], expected);
    }

    #[test]
    fn test_parse_function() {
        trace();

        let mut s = String::from("pl0AC4");
        let mut parser = Parser::new(&mut s);
        parser.parse().unwrap();

        let expected = Atom::Note(60);
        assert_eq!(parser.pool[0], expected);

        let expected = Atom::Number(10);
        assert_eq!(parser.pool[1], expected);
    }
}
