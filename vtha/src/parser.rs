use winnow::ascii::alpha1;
use winnow::ascii::alphanumeric1;
// #![cfg(feature = "alloc")]
use winnow::ascii::digit1;
use winnow::ascii::multispace0;
use winnow::ascii::space0;
use winnow::ascii::space1;
use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::combinator::eof;
use winnow::combinator::opt;
use winnow::combinator::peek;
use winnow::combinator::preceded;
use winnow::combinator::separated;
use winnow::combinator::terminated;
use winnow::combinator::trace;
use winnow::error::ParserError;
use winnow::stream::AsChar;
use winnow::stream::Stream;
use winnow::token::literal;

use winnow::token::one_of;
use winnow::PResult;
use winnow::Parser;

use crate::empty_list;
use crate::Atom;
use crate::Expression;
use crate::Function;

const OPEN_LIST: [char; 4] = ['(', '{', '[', '<'];
const CLOSE_LIST: [char; 4] = [')', '}', ']', '>'];

///
///
/// ` ` `
/// f 1 2 3
///
///   f _
///     1 2 3
///     _
///
/// f 1 2 [3]
///
///   f _
///     1 2 _
///         3
///         _
///     _
///
///
/// f 1 [2, 3]
///
///   f _
///     1 _
///       2 3
///     _
///   _
///
/// f (1 2) 3
///
///   f _
///     (1 2) 3
///     _
///   _
///
///
///  f (f 1) [2, 3]
///
///   f _
///     f _
///       1
///     _
///     2 3
///     _
///   _
///    
///
/// ` ` `
///
///

///
/// entry point for parsing
///
/// ` ` `
/// f
/// ` ` `
///
pub fn parse(i: &mut &'_ str) -> PResult<Expression> {
    trace("parse", parse_function_expression).parse_next(i)
}

///
/// Parse a function expression
/// Function expressions are of the form:
///    
/// ` ` `
///
///   f
///   {
///     {_...}a
///   }
///
/// ` ` `
///
fn parse_function_expression<'a>(input: &mut &'a str) -> PResult<Expression> {
    // (parse_function, opt(whitespace(parse_atoms)))
    (parse_function, opt(parse_atoms))
        .map(|(f, a)| {
            let a = match a {
                Some(a) => flatten_list(a),
                None => empty_list![],
            };
            Expression::Function(f, a)
        })
        .parse_next(input)
}

///
/// Flatten an Atom list with a single element into the element itself
///
fn flatten_list(a: Atom) -> Atom {
    match a {
        Atom::List(mut v) => {
            if v.len() == 1 {
                v.pop().unwrap()
            } else {
                Atom::List(v)
            }
        }
        _ => a,
    }
}

///
/// parses a list of Atoms separated by spaces
///
/// ` ` `
///   
///   [a...,_]
///   
///
/// ` ` `
///
fn parse_atoms<'a>(input: &mut &'a str) -> PResult<Atom> {
    trace(
        "atom_list",
        separated(0.., parse_atom, space).map(|a| Atom::List(a)),
    )
    .parse_next(input)
}

///
/// Parses an Atom
/// An Atom is either a value or a list of Atoms
///
///
/// ` ` `
///   
///   (a...,_)
///   v
///
/// ` ` `
///
///
fn parse_atom<'a>(input: &mut &'a str) -> PResult<Atom> {
    alt((parse_value, parse_atom_list)).parse_next(input)
}

///
/// Parses an Atom list - a Vec of Atoms
/// A List is delimited by braces `(...)`
/// A list is an Atom, so lists can be nested
/// e.g.
///     (1 2 3)
///     (1 2 (3))
///
fn parse_atom_list<'a>(input: &mut &'a str) -> PResult<Atom> {
    trace(
        "atom_list",
        delimited(parse_open_list, parse_atoms, parse_close_list),
    )
    .parse_next(input)
}

fn parse_open_list<'a>(input: &mut &'a str) -> PResult<char> {
    trace("open", one_of(OPEN_LIST)).parse_next(input)
}

fn parse_close_list<'a>(input: &mut &'a str) -> PResult<char> {
    trace("open", one_of(CLOSE_LIST)).parse_next(input)
}

///
/// Parses a Value
/// At the lowest level, an Atom wraps a value
/// A value is either a number or a string
/// e.g.
///     1
///     a
///
fn parse_value<'a>(input: &mut &'a str) -> PResult<Atom> {
    let parser = alt((parse_num, parse_string));
    trace("atom_value", parser).parse_next(input)
}

///
/// numbers
/// Matches 1 or more numbers, terminated by a space
/// Negatives not supported atm
///
fn parse_num(i: &mut &'_ str) -> PResult<Atom> {
    // alt((
    //     digit1.try_map(|d: &str| d.parse::<i32>().map(Atom::Num)),
    //     preceded("-", digit1).map(|d: &str| Atom::Num(-d.parse::<i32>().unwrap())),
    // )),

    // terminated(digit1, opt())

    // let p = peek((digit1, (space, eof))).parse_next(i);
    // let p = peek(digit1).parse_next(i)?;
    // println!("p: {:?}", p);

    // using ? ensures that peek will error if the number is not free standing ie 1One should be parsed as a string
    let _ = peek(terminated(digit1, alt((literal(')'), space1, eof)))).parse_next(i)?;

    trace(
        "num",
        digit1.try_map(|d: &str| d.parse::<i32>().map(Atom::Num)),
    )
    .parse_next(i)
}

fn parse_string(i: &mut &'_ str) -> PResult<Atom> {
    trace(
        "string",
        alphanumeric1.map(|s: &str| match s.len() {
            1 => Atom::Char(s.chars().next().unwrap_or_default()),
            _ => Atom::String(s.to_owned()),
        }),
    )
    .parse_next(i)
}

///
/// Initial Separator between f and a in `f a`
/// Happens to be a single space
///
///
fn space(i: &mut &'_ str) -> PResult<char> {
    trace("space", one_of(AsChar::is_space)).parse_next(i)
}

///
/// Parses literal function identifiers
/// e.g.
///
fn parse_function(i: &mut &'_ str) -> PResult<Function> {
    trace(
        "parse_function",
        (
            alt((
                literal("p").map(|_| Function::Play),
                literal("t").map(|_| Function::Take),
                literal("x").map(|_| Function::X),
                literal("y").map(|_| Function::Y),
            )),
            space,
        )
            .map(|(f, _)| f),
    )
    .parse_next(i)
}

// fn function_char(input: &mut &str) -> PResult<Function> {
//     dispatch! {any;
//         'p' => Function::Play,
//         _ => fail,
//     }
//     .parse_next(input)
//     // dispatch!(vec![(literal('p'), |_| Ok(Function::Play))])
// }

// fn parse_end(i: &mut &'_ str) -> PResult<Function> {
//     eof.map(|_| Function::End).parse_next(i)
// }

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn whitespace<'a, F, O, E: ParserError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod test {

    use crate::{
        list,
        parser::{flatten_list, parse, parse_atom, parse_atoms, parse_value},
        Atom, Expression, Function,
    };

    #[test]
    fn test_parse_function() {
        let mut s = "p a";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(Function::Play, Atom::Char('a'));
        assert_eq!(ast, expected);

        let mut s = "p (1)";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(Function::Play, list![Atom::Num(1)]);
        assert_eq!(ast, expected);

        let mut s = "p(1 C4 100)";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(
            Function::Play,
            list![Atom::Num(1), Atom::String("C4".to_string()), Atom::Num(100)],
        );
        assert_eq!(ast, expected);

        let mut s = "p 1 (2)";
        let ast = parse(&mut s).unwrap();
        let expected =
            Expression::Function(Function::Play, list![Atom::Num(1), list![Atom::Num(2)]]);
        assert_eq!(ast, expected);

        let mut s = "p (1 2 3)";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(
            Function::Play,
            list![Atom::Num(1), Atom::Num(2), Atom::Num(3)],
        );
        assert_eq!(ast, expected);

        let mut s = "p (1 (2) 3)";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(
            Function::Play,
            list![Atom::Num(1), list![Atom::Num(2)], Atom::Num(3)],
        );
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_nested_functions() {
        let mut s = "x y";
        let ast = parse(&mut s).unwrap();
        let expected = Expression::Function(Function::X, Atom::Char('y'));
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_atom_list() {
        let mut s: &str = "1 2 3";
        let expected = list![Atom::Num(1), Atom::Num(2), Atom::Num(3)];
        let ast = parse_atoms(&mut s).unwrap();
        assert_eq!(ast, expected);

        let mut s: &str = "(1 2 3)";
        let expected = list![list![Atom::Num(1), Atom::Num(2), Atom::Num(3)]];
        let ast = parse_atoms(&mut s).unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_nested_atom_lists() {
        let mut s: &str = "1 (2) 3";
        let expected = list![Atom::Num(1), list![Atom::Num(2)], Atom::Num(3)];
        let ast = parse_atoms(&mut s).unwrap();
        assert_eq!(ast, expected);

        let mut s: &str = "(1 (2) 3)";
        let expected = list![list![Atom::Num(1), list![Atom::Num(2)], Atom::Num(3)]];
        let ast = parse_atoms(&mut s).unwrap();
        assert_eq!(ast, expected);

        let mut s: &str = "(1 (2 3 (4)) 5)";
        let expected = list![list![
            Atom::Num(1),
            list![Atom::Num(2), Atom::Num(3), list![Atom::Num(4)]],
            Atom::Num(5)
        ]];
        let ast = parse_atoms(&mut s).unwrap();
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_atom() {
        let mut s = "1";
        let r = parse_atom(&mut s).unwrap();
        assert_eq!(r, Atom::Num(1));

        let mut s = "42";
        let r = parse_atom(&mut s).unwrap();
        assert_eq!(r, Atom::Num(42));
    }

    #[test]
    fn test_parse_value() {
        let mut s = "1";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::Num(1));

        let mut s = "42";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::Num(42));

        let mut s = "1000";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::Num(1000));

        let mut s = "a";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::Char('a'));

        let mut s = "C4";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::String("C4".to_string()));

        let mut s = "42Vtha";
        let r = parse_value(&mut s).unwrap();
        assert_eq!(r, Atom::String("42Vtha".to_string()));
    }

    #[test]
    fn test_flatten() {
        let list = list![Atom::Num(1)];
        let res = flatten_list(list);
        assert_eq!(res, Atom::Num(1));

        let list = list![Atom::Num(1), Atom::Num(2)];
        let res = flatten_list(list.clone());

        assert_eq!(res, list);

        let list = Atom::Num(1);
        let res = flatten_list(list);
        assert_eq!(res, Atom::Num(1));
    }
}
