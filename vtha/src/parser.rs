#![allow(unused_imports)]

use tracing::error;
use tracing::info;

use crate::empty_list;
use crate::Atom;
use crate::Expression;
use crate::Function;
use crate::Glyph;
use crate::SyntaxError;
use crate::VthaError;
use crate::MIDI_NOTES;

const FUNCTIONS: [&str; 2] = ["pl", "id"];

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
// ///
// ///
// /// ` ` `
// ///
// ///

///
/// Parses literal function identifiers
/// e.g.
///
fn function(s: &mut &str) -> Result<Atom, VthaError> {
    let token = s.take();

    let f: fn(&mut &str) -> Result<Atom, VthaError> =
        match token {
            Some("pl") => play,
            Some("id") => ident,
            Some(&_) => todo!(),
            None => {
                return Err(VthaError::SyntaxError(
                    SyntaxError::UnknownFunction {
                        f: s.to_string(),
                    },
                ))
            }
        };

    f(s)
}

// enum Either {
//     Atom(Atom),
//     Token(&str),
// }

pub trait Token {
    // fn next_token(&mut self) -> Option<&str>;

    // fn peek_next(&mut self) -> Option<&str>;

    fn is_function(self) -> bool;

    fn take_count(&mut self, count: usize) -> Option<&str>;

    fn take(&mut self) -> Option<&str>;

    fn to_atom(
        &mut self,
        atomizer: fn(&str) -> Atom,
    ) -> Result<Atom, VthaError>;

    fn as_string(&mut self) -> Result<Atom, VthaError>;

    fn as_num(&mut self) -> Result<Atom, VthaError>;

    fn as_note(&mut self) -> Result<Atom, VthaError>;
}

impl Token for &str {
    fn is_function(self) -> bool {
        match self {
            "pl" | "id" => true,
            _ => false,
        }
    }

    fn take_count(&mut self, count: usize) -> Option<&str> {
        let token = peek_next(self);

        if token.filter(|t| t.is_function()).is_some() {
            next_token(self, 2)
        } else {
            next_token(self, count)
        }
    }

    fn take(&mut self) -> Option<&str> {
        self.take_count(2)
    }

    fn to_atom(
        &mut self,
        atomizer: fn(&str) -> Atom,
    ) -> Result<Atom, VthaError> {
        if self.is_function() {
            function(self)
        } else {
            Ok(atomizer(self))
        }
    }

    fn as_string(&mut self) -> Result<Atom, VthaError> {
        self.to_atom(|s| Atom::String(s.to_string()))
    }

    fn as_num(&mut self) -> Result<Atom, VthaError> {
        self.to_atom(|s| {
            let n = u8::from_str_radix(s, 16).unwrap();
            Atom::Num(n)
        })
    }

    fn as_note(&mut self) -> Result<Atom, VthaError> {
        self.to_atom(|s| {
            let n = MIDI_NOTES.get(s).unwrap_or(&0).clone();
            Atom::Note(n)
        })
    }
}

#[inline(always)]
fn next_token<'a>(
    s: &'a mut &'a str,
    count: usize,
) -> Option<&str> {
    match s.len() {
        0 | 1 => None,
        _ => {
            let (next_token, rest) = s.split_at(count);
            *s = rest;
            Some(next_token)
        }
    }
}

#[inline(always)]
fn peek_next<'a>(s: &'a mut &'a str) -> Option<&str> {
    match s.len() {
        0 | 1 => None,
        _ => {
            let (next_token, _) = s.split_at(2);
            Some(next_token)
        }
    }
}

fn ident<'a>(s: &mut &str) -> Result<Atom, VthaError> {
    let param = s.take().map(|s| s.as_string())?;
    Ok(Atom::Function(Box::new(Function::Ident(param))))
}

// match self {
//     Some(token) => Ok(atomizer(self)),
//     None => {
//         return Err(VthaError::SyntaxError(
//             SyntaxError::ExpectedToken {},
//         ));
//     }
// }

// channel, velocity, note
fn play(s: &mut &str) -> Result<Atom, VthaError> {
    let ch = s.take_count(1).to_atom_num()?;

    let ch = s.take_atom_num()?;
    let vel = s.take_atom_num()?;
    let note = s.take_atom_note()?;

    Ok(Atom::Function(Box::new(Function::Play(
        ch, vel, note,
    ))))
}

#[cfg(test)]
mod test {

    use tracing::info;

    use crate::{
        list,
        parser::{function, Token},
        Atom, Expression, Function,
    };

    fn trace() {
        use tracing_subscriber::FmtSubscriber;

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
    #[test]
    fn test_parse_id_function() {
        trace();

        let mut s = "idAA";
        let ast = function(&mut s);
        info!("{:?}", ast);

        let expected = Atom::Function(Box::new(
            Function::Ident(Atom::String("AA".to_string())),
        ));
        // assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_function() {
        let mut s = "pl010AC4";
        let ast = function(&mut s).unwrap();

        let expected =
            Atom::Function(Box::new(Function::Play(
                Atom::Num(1),
                Atom::Num(10),
                Atom::Note(60),
            )));

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
    fn test_next_token() {
        trace();
        let mut s = "xy";
        let s = s.next_token().unwrap();
        assert_eq!(s, "xy");

        let mut s = "x";
        let s = s.next_token();
        assert_eq!(s, None);

        let mut s = "";
        let s = s.next_token();
        assert_eq!(s, None);
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
