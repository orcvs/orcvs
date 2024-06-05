use crate::midi_note_to_number;
use crate::midi_number_to_note;
use crate::ArgumentError;
use crate::Atom;
use crate::AtomFunction;
use crate::AtomNote;
use crate::AtomNumber;
use crate::AtomRef;
use crate::AtomString;
use crate::AtomTrait;
use crate::Error;
use crate::Function;
use crate::SyntaxError;

const DEFAULT_TOKEN_LEN: usize = 2;

struct Pool<'a> {
    pool: Vec<Atom>,
    source: &'a str,
    take_next: usize,
}

impl<'a> Pool<'a> {
    fn new(source: &'a mut str) -> Self {
        Self {
            pool: Vec::new(),
            take_next: DEFAULT_TOKEN_LEN,
            source,
        }
    }

    pub fn parse(&mut self) -> Result<AtomRef<AtomFunction>, Error> {
        self.take_function()
    }

    pub fn as_string(&mut self) -> Result<AtomRef<AtomString>, Error> {
        self.inner_take(|s| {
            let a = Atom::String(s.to_string());
            Ok(a)
        })
    }

    pub fn as_note(&mut self) -> Result<AtomRef<AtomNote>, Error> {
        self.inner_take(|s| match midi_note_to_number(&s) {
            Some(n) => {
                let a = Atom::Note(n);
                Ok(a)
            }
            None => Err(ArgumentError::NoteExpected(s.to_string()).into()),
        })
    }

    pub fn as_num(&mut self) -> Result<AtomRef<AtomNumber>, Error> {
        self.inner_take(|s| match u8::from_str_radix(&s, 16) {
            Ok(n) => {
                let a = Atom::Number(n);
                Ok(a)
            }
            Err(_) => Err(ArgumentError::NumberExpected(s.to_string()).into()),
        })
    }

    pub fn take_function<T: AtomTrait + 'static>(&mut self) -> Result<AtomRef<T>, Error> {
        let token = self.next_token(2);

        let f: fn(&mut Pool) -> Result<AtomFunction, Error> = match token {
            Some("pl") => play,
            Some("id") => ident,
            Some(s) => {
                return Err(Error::SyntaxError(SyntaxError::UnknownFunction {
                    f: s.to_string(),
                }));
            }
            None => {
                return Err(Error::SyntaxError(SyntaxError::ExpectedToken {}));
            }
        };

        let a = f(self)?;
        let r = self.add(a.into());
        Ok(r)
    }

    #[inline(always)]
    // pub fn get(&self, atom_ref: usize) -> &Box<dyn AtomTrait> {
    //     &self.pool[atom_ref]
    // }
    pub fn get<T: Into<usize>>(&self, atom_ref: T) -> &Atom {
        let index: usize = atom_ref.into();
        &self.pool[index]
    }

    #[inline(always)]
    fn add<T: AtomTrait + 'static>(&mut self, atom: Atom) -> AtomRef<T> {
        let idx = self.pool.len();
        self.pool.push(atom);
        AtomRef::new(idx)
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

    pub fn inner_take<F, T>(&mut self, atomizer: F) -> Result<AtomRef<T>, Error>
    where
        T: AtomTrait + 'static,
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
                    let r = self.add(a);
                    Ok(r)
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

fn ident(pool: &mut Pool) -> Result<AtomFunction, Error> {
    let param = pool.next().as_string()?;
    Ok(AtomFunction(Function::Ident(param)))
}

// channel, velocity, note
fn play(pool: &mut Pool) -> Result<AtomFunction, Error> {
    let ch = pool.take(1).as_num()?;
    let vel = pool.next().as_num()?;
    let note = pool.next().as_note()?;
    Ok(AtomFunction(Function::Play(ch, vel, note)))
}

#[cfg(test)]
mod test {

    use crate::{parser::Pool, trace, Atom, AtomRef, Function};

    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = String::from("idAA");
        let mut pool = Pool::new(&mut s);
        let atom_ref = pool.parse().unwrap();

        let expected = Atom::Function(Function::Ident(AtomRef::new(0)));

        let atom = pool.get(atom_ref);

        assert_eq!(atom, &expected);
    }

    // #[test]
    // fn test_parse_recursive_function() {
    //     trace();

    //     let mut s = String::from("idididAA");
    //     let mut pool = Pool::new(&mut s);
    //     let ast = pool.parse().unwrap();
    //     info!("{:?}", ast);

    //     let f = Atom::from(Function::Ident(AtomRef::new(2)));
    //     let f = Atom::from(Function::Ident(AtomRef::new(1)));
    //     let f = Atom::from(Function::Ident(AtomRef::new(0)));

    //     let expected = f;

    //     // assert_eq!(ast, expected);
    // }

    // #[test]
    // fn test_parse_function() {
    //     trace();
    //     // let mut s = "pl10AC4";
    //     // let ast = function(&mut s).unwrap();

    //     // let expected = Atom::Function(Box::new(Function::Play(
    //     //     Atom::Num(1),
    //     //     Atom::Num(10),
    //     //     Atom::Note(60),
    //     // )));

    //     // assert_eq!(ast, expected);

    //     let mut s = String::from("plidXY0AC4");
    //     let mut pool = Pool::new(&mut s);

    //     // let ast = parse(&mut s).unwrap();
    //     let ast = pool.parse().unwrap();
    //     info!("{:?}", ast);

    //     let expected = Atom::from(Function::Play(
    //         AtomRef::new(1),
    //         AtomRef::new(2),
    //         AtomRef::new(3),
    //     ));

    //     // assert_eq!(ast, expected);

    //     // let id = Atom::Function(Box::new(Function::Ident(
    //     //     Atom::String("A".to_string()),
    //     // )));
    //     // let expected =
    //     //     Atom::Function(Box::new(Function::Play(
    //     //         id,
    //     //         Atom::Num(255),
    //     //         Atom::Num(60),
    //     //     )));

    //     // assert_eq!(ast, expected);

    //     // let mut s = "plAFFC4";
    //     // let ast = parse_function(&mut s).unwrap();
    //     // let expected = Function::Play(10, 255, 60);

    //     // assert_eq!(ast, expected);
    // }

    // #[test]
    // fn test_atom_into() {
    //     trace();

    //     let num = Atom::Number(60);
    //     let str = Atom::String("3C".to_string());
    //     let note = Atom::Note(60);

    //     assert_eq!(num, num.clone().into_num().unwrap());
    //     assert_eq!(num, str.into_num().unwrap());
    //     assert_eq!(num, note.into_num().unwrap());

    //     let str = Atom::String("C4".to_string());
    //     let note = Atom::Note(60);
    //     assert_eq!(str, str.clone().into_string().unwrap());
    //     assert_eq!(str, note.into_string().unwrap());

    //     // Numbers convert to string literally, not as note value
    //     let str = Atom::String("A".to_string());
    //     let num = Atom::Number(10);
    //     assert_eq!(str, num.into_string().unwrap());

    //     let num = Atom::Number(60);
    //     let str = Atom::String("C4".to_string());
    //     let note = Atom::Note(60);

    //     assert_eq!(note, note.clone().into_note().unwrap());
    //     assert_eq!(note, str.into_note().unwrap());
    //     assert_eq!(note, num.into_note().unwrap());

    //     let str = Atom::String("XYZ".to_string());
    //     str.into_note().expect_err("Expected error");

    //     let str = Atom::String("XYZ".to_string());
    //     str.into_num().expect_err("Expected error");

    //     let str = Atom::String("CA".to_string());
    //     str.into_note().expect_err("Expected error");
    // }

    #[test]
    fn test_next_token() {
        trace();
        // let mut s = "xy";
        // let s = next_token(s).unwrap();
        // assert_eq!(s, "xy");

        // let mut s = "x";
        // let s = next_token(s).unwrap();
        // assert_eq!(s, None);

        // let mut s = "";
        // let s = next_token(s).unwrap();
        // assert_eq!(s, None);
    }

    // #[test]
    // fn test_parse_atom_list() {
    //     let mut s: &str = "1 2 3";
    //     let expected =
    //         list![Atom::Num(1), Atom::Num(2), Atom::Num(3)];
    //     let ast = parse_atoms(&mut s).unwrap();
    //     assert_eq!(ast, expected);

    //     let mut s: &str = "(1 2 3)";
    //     let expected = list![list![
    //         Atom::Num(1),
    //         Atom::Num(2),
    //         Atom::Num(3)
    //     ]];
    //     let ast = parse_atoms(&mut s).unwrap();
    //     assert_eq!(ast, expected);
    // }

    // #[test]
    // fn test_parse_nested_atom_lists() {
    //     let mut s: &str = "1 (2) 3";
    //     let expected = list![
    //         Atom::Num(1),
    //         list![Atom::Num(2)],
    //         Atom::Num(3)
    //     ];
    //     let ast = parse_atoms(&mut s).unwrap();
    //     assert_eq!(ast, expected);

    //     let mut s: &str = "(1 (2) 3)";
    //     let expected = list![list![
    //         Atom::Num(1),
    //         list![Atom::Num(2)],
    //         Atom::Num(3)
    //     ]];
    //     let ast = parse_atoms(&mut s).unwrap();
    //     assert_eq!(ast, expected);

    //     let mut s: &str = "(1 (2 3 (4)) 5)";
    //     let expected = list![list![
    //         Atom::Num(1),
    //         list![
    //             Atom::Num(2),
    //             Atom::Num(3),
    //             list![Atom::Num(4)]
    //         ],
    //         Atom::Num(5)
    //     ]];
    //     let ast = parse_atoms(&mut s).unwrap();
    //     assert_eq!(ast, expected);
    // }

    // #[test]
    // fn test_parse_atom() {
    //     let mut s = "1";
    //     let r = parse_atom(&mut s).unwrap();
    //     assert_eq!(r, Atom::Num(1));

    //     let mut s = "42";
    //     let r = parse_atom(&mut s).unwrap();
    //     assert_eq!(r, Atom::Num(42));
    // }

    // #[test]
    // fn test_parse_value() {
    //     let mut s = "1";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::Num(1));

    //     let mut s = "42";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::Num(42));

    //     let mut s = "1000";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::Num(1000));

    //     let mut s = "a";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::Char('a'));

    //     let mut s = "C4";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::String("C4".to_string()));

    //     let mut s = "42Vtha";
    //     let r = parse_value(&mut s).unwrap();
    //     assert_eq!(r, Atom::String("42Vtha".to_string()));
    // }

    // #[test]
    // fn test_flatten() {
    //     let list = list![Atom::Num(1)];
    //     let res = flatten_list(list);
    //     assert_eq!(res, Atom::Num(1));

    //     let list = list![Atom::Num(1), Atom::Num(2)];
    //     let res = flatten_list(list.clone());

    //     assert_eq!(res, list);

    //     let list = Atom::Num(1);
    //     let res = flatten_list(list);
    //     assert_eq!(res, Atom::Num(1));
    // }
}
