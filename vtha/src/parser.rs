#![allow(unused_imports)]

use tracing::info;

use crate::empty_list;
use crate::Atom;
use crate::Expression;
use crate::Function;
use crate::SyntaxError;
use crate::VthaError;

const OPEN_LIST: [char; 4] = ['(', '{', '[', '<'];
const CLOSE_LIST: [char; 4] = [')', '}', ']', '>'];

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
    if s.len() < 2 {
        return Err(VthaError::SyntaxError(
            SyntaxError::UnknownFunction {
                f: s.to_string(),
            },
        ));
    }

    let (first_two, rest) = s.split_at(2);
    *s = rest;

    let f: fn(&mut &str) -> Result<Atom, VthaError> =
        match first_two {
            "pl" => play,
            "id" => ident,
            _ => {
                return Err(VthaError::SyntaxError(
                    SyntaxError::UnknownFunction {
                        f: first_two.to_string(),
                    },
                ))
            }
        };

    f(s)
    // Now you can use first_two and call the next function with s
    // For example:
    // let result = next_function(s);

    // OK

    // "pl" => play,
    // "id" => ident,

    // OK
}

fn ident<'a>(s: &mut &str) -> Result<Atom, VthaError> {
    // Get the next two characters
    let (next_two, _) = s.split_at(2);

    // If the next two characters are a function, parse function
    // else parse the parameter
    let param: Atom = if is_function(next_two) {
        info!("next {:?}", next_two);
        function(s)?
    } else {
        let (next, rest) = s.split_at(1);
        info!("next {:?}", next);
        *s = rest;

        Atom::String(next.to_string())
    };

    Ok(Atom::Function(Box::new(Function::Ident(param))))
}

fn is_function(s: &str) -> bool {
    let functions = ["pl", "id"];
    functions.contains(&s)
}

fn parameter(
    s: &mut &str,
    len: usize,
) -> Result<Atom, VthaError> {
    // Get the next two characters
    let (next_two, _) = s.split_at(2);

    // If the next two characters are a function, parse function
    // else parse the parameter
    if is_function(next_two) {
        info!("next {:?}", next_two);
        function(s)
    } else {
        let (next, rest) = s.split_at(len);
        info!("next {:?}", next);
        *s = rest;

        Ok(Atom::Num(0))
        // let ch = u8::from_str_radix(next, 16).unwrap();
        // Atom::Num(ch)
    }
}

// channel, velocity, note
fn play(s: &mut &str) -> Result<Atom, VthaError> {
    use std::collections::HashMap;

    let note_to_midi: HashMap<&str, u8> = [
        ("A0", 21),
        ("a0", 22),
        ("B0", 23),
        ("C1", 24),
        ("c1", 25),
        ("D1", 26),
        ("d1", 27),
        ("E1", 28),
        ("F1", 29),
        ("f1", 30),
        ("G1", 31),
        ("g1", 32),
        ("A1", 33),
        ("a1", 34),
        ("B1", 35),
        ("C2", 36),
        ("c2", 37),
        ("D2", 38),
        ("d2", 39),
        ("E2", 40),
        ("F2", 41),
        ("f2", 42),
        ("G2", 43),
        ("g2", 44),
        ("A2", 45),
        ("a2", 46),
        ("B2", 47),
        ("C3", 48),
        ("c3", 49),
        ("D3", 50),
        ("d3", 51),
        ("E3", 52),
        ("F3", 53),
        ("f3", 54),
        ("G3", 55),
        ("g3", 56),
        ("A3", 57),
        ("a3", 58),
        ("B3", 59),
        ("C4", 60),
        ("c4", 61),
        ("D4", 62),
        ("d4", 63),
        ("E4", 64),
        ("F4", 65),
        ("f4", 66),
        ("G4", 67),
        ("g4", 68),
        ("A4", 69),
        ("a4", 70),
        ("B4", 71),
        ("C5", 72),
        ("c5", 73),
        ("D5", 74),
        ("d5", 75),
        ("E5", 76),
        ("F5", 77),
        ("f5", 78),
        ("G5", 79),
        ("g5", 80),
        ("A5", 81),
        ("a5", 82),
        ("B5", 83),
        ("C6", 84),
        ("c6", 85),
        ("D6", 86),
        ("d6", 87),
        ("E6", 88),
        ("F6", 89),
        ("d6", 90),
        ("G6", 91),
        ("g6", 92),
        ("A6", 93),
        ("a6", 94),
        ("B6", 95),
        ("C7", 96),
        ("c7", 97),
        ("D7", 98),
        ("d7", 99),
        ("E7", 100),
        ("F7", 101),
        ("f7", 102),
        ("G7", 103),
        ("g7", 104),
        ("A7", 105),
        ("a7", 106),
        ("B7", 107),
        ("C8", 108),
        ("c8", 109),
        ("D8", 110),
        ("d8", 111),
        ("E8", 112),
        ("F8", 113),
        ("f8", 114),
        ("G8", 115),
        ("g8", 116),
        ("A8", 117),
        ("a8", 118),
        ("B8", 119),
        ("C9", 120),
        ("c9", 121),
        ("D9", 122),
        ("d9", 123),
        ("E9", 124),
        ("F9", 125),
        ("f9", 126),
        ("G9", 127),
    ]
    .iter()
    .cloned()
    .collect();

    // Get the next two characters
    let (next_two, _) = s.split_at(2);

    // If the next two characters are a function, parse function
    // else parse the parameter
    let ch: Atom = if is_function(next_two) {
        info!("next {:?}", next_two);
        function(s)?
    } else {
        let (next, rest) = s.split_at(1);
        info!("next {:?}", next);
        *s = rest;
        let ch = u8::from_str_radix(next, 16).unwrap();
        Atom::Num(ch)
    };

    // and repeat
    let (next_two, _) = s.split_at(2);

    let vel: Atom = if is_function(next_two) {
        info!("next {:?}", next_two);
        function(s)?
    } else {
        let (next, rest) = s.split_at(2);
        info!("next {:?}", next);
        *s = rest;
        let vel = u8::from_str_radix(next, 16).unwrap();
        Atom::Num(vel)
    };

    let (next_two, _) = s.split_at(2);

    let note: Atom = if is_function(next_two) {
        info!("next {:?}", next_two);
        function(s)?
    } else {
        let (next, rest) = s.split_at(2);
        info!("next {:?}", next);
        *s = rest;

        let note =
            note_to_midi.get(next).unwrap_or(&0).clone();

        Atom::Num(note)
    };

    Ok(Atom::Function(Box::new(Function::Play(
        ch, vel, note,
    ))))
}

#[cfg(test)]
mod test {

    use crate::{
        list, parser::function, Atom, Expression, Function,
    };

    #[test]
    fn test_parse_function() {
        use tracing_subscriber::FmtSubscriber;

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let mut s = "plidAFFC4";
        let ast = function(&mut s).unwrap();

        let id = Atom::Function(Box::new(Function::Ident(
            Atom::String("A".to_string()),
        )));
        let expected =
            Atom::Function(Box::new(Function::Play(
                id,
                Atom::Num(255),
                Atom::Num(60),
            )));

        assert_eq!(ast, expected);

        // let mut s = "plAFFC4";
        // let ast = parse_function(&mut s).unwrap();
        // let expected = Function::Play(10, 255, 60);

        // assert_eq!(ast, expected);
    }

    // #[test]
    // fn test_parse_nested_functions() {
    //     let mut s = "x y";
    //     let ast = parse(&mut s).unwrap();
    //     let expected = Expression::Function(
    //         Function::X,
    //         Atom::Char('y'),
    //     );
    //     assert_eq!(ast, expected);
    // }

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
