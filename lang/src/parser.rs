use crate::midi_note_to_number;
use crate::midi_number_to_note;
use crate::ArgumentError;
use crate::Atom;
use crate::Function;
use crate::SyntaxError;
use crate::VthaError;

use tracing::debug;

// ///
// ///
// /// ` ` `
// /// f 1 2 3
// ///
// ///   f _
// ///     1 2 3
// ///     _
// ///
// /// f 1 2 [3]
// ///
// ///   f _
// ///     1 2 _
// ///         3
// ///         _
// ///     _
// ///
// ///
// /// f 1 [2, 3]
// ///
// ///   f _
// ///     1 _
// ///       2 3
// ///     _
// ///   _
// ///
// /// f (1 2) 3
// ///
// ///   f _
// ///     (1 2) 3
// ///     _
// ///   _
// ///
// ///
// ///  f (f 1) [2, 3]
// ///
// ///   f _
// ///     f _
// ///       1
// ///     _
// ///     2 3
// ///     _
// ///   _
// //
// ///
// /// ` ` `
// ///
// ///

///
/// Parses literal function identifiers
/// e.g.
/// # Errors
///
pub fn parse<'a, 'b: 'a>(s: &'a mut &'b str) -> Result<Atom, VthaError> {
    let token = next_token(s, 2);

    let f: fn(&mut &str) -> Result<Atom, VthaError> = match token {
        Some("pl") => play,
        Some("id") => ident,
        Some(s) => {
            return Err(VthaError::SyntaxError(SyntaxError::UnknownFunction {
                f: s.to_string(),
            }));
        }
        None => {
            return Err(VthaError::SyntaxError(SyntaxError::ExpectedToken {}));
        }
    };
    debug!("Params {:?}", s);
    f(s)
}

impl Atom {
    #[allow(dead_code)]
    fn into_string(self) -> Result<Atom, VthaError> {
        match self {
            Atom::String(s) => Ok(Atom::String(s)),
            Atom::Num(n) => {
                let s = format!("{n:X}");
                Ok(Atom::String(s))
            }
            Atom::Note(n) => {
                if let Some(s) = midi_number_to_note(n) {
                    Ok(Atom::String(s.to_string()))
                } else {
                    let s = n.to_string();
                    Err(ArgumentError::StringExpected(s).into())
                }
            }
            Atom::Function(_) => Ok(self),
        }
    }

    fn into_num(self) -> Result<Atom, VthaError> {
        match self {
            Atom::String(s) => match u8::from_str_radix(&s, 16) {
                Ok(n) => Ok(Atom::Num(n)),
                Err(_) => Err(ArgumentError::NumberExpected(s).into()),
            },
            Atom::Num(_) => Ok(self.clone()),
            Atom::Note(n) => Ok(Atom::Num(n)),
            Atom::Function(_) => Ok(self),
        }
    }

    fn into_note(self) -> Result<Atom, VthaError> {
        match self {
            Atom::String(s) => {
                if let Some(n) = midi_note_to_number(&s) {
                    Ok(Atom::Note(n))
                } else {
                    Err(ArgumentError::NoteExpected(s).into())
                }
            }
            Atom::Num(n) => Ok(Atom::Note(n)),
            Atom::Note(_) => Ok(self.clone()),
            Atom::Function(_) => Ok(self),
        }
    }
}

pub trait Token<'a> {
    fn is_function(&'a self) -> bool;

    fn take_count(&'a mut self, count: usize) -> Result<Atom, VthaError>;

    fn take(&'a mut self) -> Result<Atom, VthaError>;
}

impl<'a, 'b: 'a> Token<'a> for &'b str {
    fn is_function(&'a self) -> bool {
        matches!(*self, "pl" | "id")
    }

    fn take_count(&'a mut self, count: usize) -> Result<Atom, VthaError> {
        let token = peek_next(self);

        if token.filter(|t| t.is_function()).is_some() {
            // info!("is_function {:?}", token);
            // Always Some because we peeked
            // let mut t = next_token(self, 2).unwrap();
            parse(self)
        } else {
            let t = next_token(self, count);
            // info!("take_count {:?}", t);
            match t {
                Some(t) => {
                    // error!("take_count {:?}", t);
                    // let s = t.to_string();
                    // error!("take_count {:?}", s);
                    Ok(Atom::String(t.to_string()))
                }
                None => Err(VthaError::SyntaxError(SyntaxError::ExpectedToken {})),
            }
        }
    }

    fn take(&mut self) -> Result<Atom, VthaError> {
        self.take_count(2)
    }
}

#[inline(always)]
fn next_token<'a>(s: &mut &'a str, count: usize) -> Option<&'a str> {
    // info!("next_token {:?}", s);
    match s.len() {
        0 | 1 => None,
        _ => {
            let (next_token, rest) = s.split_at(count);
            // info!("next_token {:?}", next_token);
            *s = rest;
            Some(next_token)
        }
    }
}

#[inline(always)]
fn peek_next(s: &str) -> Option<&str> {
    match s.len() {
        0 | 1 => None,
        _ => {
            // info!("peek_next {:?}", s);
            let (next_token, _) = s.split_at(2);
            Some(next_token)
        }
    }
}

fn ident(s: &mut &str) -> Result<Atom, VthaError> {
    let param = s.take()?;
    Ok(Atom::from(Function::Ident(param)))
}

// channel, velocity, note
fn play(s: &mut &str) -> Result<Atom, VthaError> {
    let ch = s.take_count(1)?.into_num()?;
    let vel = s.take()?.into_num()?;
    let note = s.take()?.into_note()?;

    Ok(Atom::from(Function::Play(ch, vel, note)))
}

#[cfg(test)]
mod test {

    use crate::{parser::parse, trace, Atom, Function};

    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = "idAA";
        let ast = parse(&mut s).unwrap();
        // info!("{:?}", ast);

        let expected = Atom::from(Function::Ident(Atom::String("AA".to_string())));

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_recursive_function() {
        trace();

        let mut s = "idididAA";
        let ast = parse(&mut s).unwrap();
        // info!("{:?}", ast);

        let f = Atom::from(Function::Ident(Atom::String("AA".to_string())));
        let f = Atom::from(Function::Ident(f));
        let f = Atom::from(Function::Ident(f));

        let expected = f;

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_function() {
        trace();
        // let mut s = "pl10AC4";
        // let ast = function(&mut s).unwrap();

        // let expected = Atom::Function(Box::new(Function::Play(
        //     Atom::Num(1),
        //     Atom::Num(10),
        //     Atom::Note(60),
        // )));

        // assert_eq!(ast, expected);

        let mut s = "plidXY0AC4";
        let ast = parse(&mut s).unwrap();

        let id = Atom::from(Function::Ident(Atom::String("XY".to_string())));

        let expected = Atom::from(Function::Play(id, Atom::Num(10), Atom::Note(60)));

        assert_eq!(ast, expected);

        // let id = Atom::Function(Box::new(Function::Ident(
        //     Atom::String("A".to_string()),
        // )));
        // let expected =
        //     Atom::Function(Box::new(Function::Play(
        //         id,
        //         Atom::Num(255),
        //         Atom::Num(60),
        //     )));

        // assert_eq!(ast, expected);

        // let mut s = "plAFFC4";
        // let ast = parse_function(&mut s).unwrap();
        // let expected = Function::Play(10, 255, 60);

        // assert_eq!(ast, expected);
    }

    #[test]
    fn test_atom_into() {
        trace();

        let num = Atom::Num(60);
        let str = Atom::String("3C".to_string());
        let note = Atom::Note(60);

        assert_eq!(num, num.clone().into_num().unwrap());
        assert_eq!(num, str.into_num().unwrap());
        assert_eq!(num, note.into_num().unwrap());

        let str = Atom::String("C4".to_string());
        let note = Atom::Note(60);
        assert_eq!(str, str.clone().into_string().unwrap());
        assert_eq!(str, note.into_string().unwrap());

        // Numbers convert to string literally, not as note value
        let str = Atom::String("A".to_string());
        let num = Atom::Num(10);
        assert_eq!(str, num.into_string().unwrap());

        let num = Atom::Num(60);
        let str = Atom::String("C4".to_string());
        let note = Atom::Note(60);

        assert_eq!(note, note.clone().into_note().unwrap());
        assert_eq!(note, str.into_note().unwrap());
        assert_eq!(note, num.into_note().unwrap());

        let str = Atom::String("XYZ".to_string());
        str.into_note().expect_err("Expected error");

        let str = Atom::String("XYZ".to_string());
        str.into_num().expect_err("Expected error");

        let str = Atom::String("CA".to_string());
        str.into_note().expect_err("Expected error");
    }

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
