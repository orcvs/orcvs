use crate::{Atom, Atoms, Error, Function, Portal, Stack};
use arrayvec::ArrayVec;
use tracing::{error, info};

pub type Args = Stack<16>;

pub struct Interpreter {}

impl Interpreter {
    #[inline(always)]
    pub fn execute(atoms: &Atoms) -> Result<Portal, Error> {
        let mut stack = Stack::new();

        let f = match atoms.first() {
            Some(Atom::Function(f)) => *f,
            _ => Function::Empty,
        };

        for atom in atoms.iter().rev() {
            // info!("atoms: {:?}", atoms);
            // info!("stack: {:?}", stack);
            let atom = match atom {
                Atom::Function(fun) => match fun {
                    Function::Add => add(&mut stack)?,
                    Function::Divide => divide(&mut stack)?,
                    Function::Multiply => multiply(&mut stack)?,
                    Function::Subtract => subtract(&mut stack)?,
                    Function::Id => ident(&mut stack)?,
                    Function::Play => play(&mut stack)?,
                    _ => Atom::Function(Function::Empty),
                },
                atom => *atom,
            };
            stack.push(atom);
        }

        let portal = Portal::new(stack.pop().into(), 0, 0);

        // Final element in stack is the result
        // let atom = stack.pop().into();
        Ok(portal)
    }
}

fn ident(stack: &mut Args) -> Result<Atom, Error> {
    Ok(stack.pop().into())
}

#[inline(always)]
fn add(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(add_impl(arg_1, arg_2))
}

#[inline(always)]
fn add_impl(a: u8, b: u8) -> Atom {
    let res: u8 = a + b;
    Atom::Number(res)
}

#[inline(always)]
fn divide(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;

    Ok(divide_impl(arg_1, arg_2))
}

#[inline(always)]
fn divide_impl(a: u8, b: u8) -> Atom {
    // Divide by zero is zero, which is terribly incorrect
    if b == 0 {
        return Atom::Number(0);
    }
    let res = a / b;
    Atom::Number(res)
}

#[inline(always)]
fn multiply(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(multiply_impl(arg_1, arg_2))
}

#[inline(always)]
fn multiply_impl(a: u8, b: u8) -> Atom {
    error!("a: {:?}", a);
    error!("b: {:?}", b);
    let res = a * b;
    Atom::Number(res)
}

#[inline(always)]
fn subtract(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(subtract_impl(arg_1, arg_2))
}

#[inline(always)]
fn subtract_impl(a: u8, b: u8) -> Atom {
    // No negative numbers
    if a < b {
        return Atom::Number(0);
    }
    let res = a - b;
    Atom::Number(res)
}

#[inline(always)]
fn play(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(3, 0)?;
    let arg_2 = stack.try_pop(3, 1)?;
    let arg_3 = stack.try_pop(3, 2)?;
    Ok(play_impl(arg_1, arg_2, arg_3))
}

#[inline(always)]
fn play_impl(c: u8, v: u8, n: u8) -> Atom {
    info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Atom::Number(0)
}

#[cfg(test)]
mod test {

    use crate::{
        interpreter::Interpreter, trace, ArgumentError, Atom, Error, Function, Parser, TypeError,
    };
    use tracing::{error, info};

    use super::Portal;

    fn interpret(exp: String) -> Atom {
        let mut exp = exp.clone();
        let parser = Parser::from(&mut exp);
        let parsed = parser.try_parse().unwrap();

        info!("Parsed: {:?}", parsed);

        let result = Interpreter::execute(&parsed);
        result.unwrap().atom
    }

    fn interpret_stack(exp: Vec<Atom>) -> Result<Portal, Error> {
        let atoms = exp.into_iter().collect();
        Interpreter::execute(&atoms)
    }

    #[test]
    fn test_add_function() {
        trace();

        let s = String::from("++0102");
        let result = interpret(s);

        let expected = Atom::Number(3);
        assert_eq!(result, expected);

        let s = String::from("++idA01");
        let result = interpret(s);

        let expected = Atom::Number(11);
        assert_eq!(result, expected);

        let s = String::from("++idAid1");
        let result = interpret(s);

        let expected = Atom::Number(11);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sub_function() {
        trace();

        let s = String::from("--0201");
        let result = interpret(s);

        let expected = Atom::Number(1);
        assert_eq!(result, expected);

        let s = String::from("--0102");
        let result = interpret(s);

        let expected = Atom::Number(0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiply_function() {
        trace();

        let s = String::from("**0201");
        let result = interpret(s);

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        let s = String::from("**0002");
        let result = interpret(s);

        let expected = Atom::Number(0);
        assert_eq!(result, expected);

        let s = String::from("**idAidA");
        let result = interpret(s);

        error!("Result: {:?}", result);

        let expected = Atom::Number(100);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_id_function() {
        trace();

        let s = String::from("id1");
        let result = interpret(s);

        // error!("Result: {:?}", result);

        let expected = Atom::Char('1');
        assert_eq!(result, expected);
    }

    #[test]
    fn test_divide() {
        trace();

        let s = String::from("//0402");
        let result = interpret(s);

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        let s = String::from("//0100");
        let result = interpret(s);

        let expected = Atom::Number(0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_recursive() {
        trace();

        let s = String::from("++ididid901");
        let result = interpret(s);

        let expected = Atom::Number(10);
        assert_eq!(result, expected);

        let s = String::from("++++0101--0A05");
        let result = interpret(s);

        let expected = Atom::Number(7);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_missing_argument() {
        trace();

        let stack = vec![Atom::Function(Function::Add), Atom::Number(1)];

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(
            error,
            Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            })
        ));
    }

    #[test]
    fn test_with_invalid_argument() {
        trace();

        let stack = vec![
            Atom::Function(Function::Add),
            Atom::Number(1),
            Atom::Char('v'),
            Atom::Char('t'),
            Atom::Char('h'),
            Atom::Char('a'),
        ];

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }
}
