// #[allow(unused)]

use crate::{Atom, Error, Function, Parser, Stack};
use tracing::info;

pub struct Interpreter {
    stack: Stack<32>,
}

type Args = Stack<6>;

impl<'a> Interpreter {
    pub fn new(parser: &mut Parser) -> Self {
        // if parser.is_valid() {
        // }
        let stack = parser.take_stack();
        Self { stack }
    }

    pub fn from_stack(stack: Stack<32>) -> Self {
        Self { stack }
    }

    // assert!(self.stack.inner.len() != 0);
    // let stack = self.stack.deref_mut();
    // let atom = stack.take_first_mut()
    // if let Some((atom, rest)) = stack.split_first_mut() {
    // let args = &rest[0..=1];
    // let atom = self.stack.inner.remove(0);
    #[inline(always)]
    pub fn interpret(&mut self) -> Result<Atom, Error> {
        let mut stack = Args::new();

        while let Some(element) = self.stack.inner.pop() {
            let atom = match element {
                Atom::Function(fun) => match fun {
                    Function::Add => add(&mut stack)?,
                    Function::Divide => divide(&mut stack)?,
                    Function::Multiply => multiply(&mut stack)?,
                    Function::Subtract => subtract(&mut stack)?,
                    Function::Id => ident(&mut stack)?,
                    Function::Play => play(&mut stack)?,
                    _ => Atom::Function(Function::Empty),
                },
                _ => element,
            };
            stack.push(atom);
        }

        // Final element in stack is the result
        let atom = stack.pop().into();
        Ok(atom)
    }
}

fn ident(stack: &mut Args) -> Result<Atom, Error> {
    Ok(stack.pop().into())
}

fn add(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(add_impl(arg_1, arg_2))
}

fn add_impl(a: u8, b: u8) -> Atom {
    let res: u8 = a + b;
    Atom::Number(res)
}

fn divide(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;

    Ok(divide_impl(arg_1, arg_2))
}

fn divide_impl(a: u8, b: u8) -> Atom {
    // Divide by zero is zero, which is terribly incorrect
    if b == 0 {
        return Atom::Number(0);
    }
    let res = a / b;
    Atom::Number(res)
}

fn multiply(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(multiply_impl(arg_1, arg_2))
}

fn multiply_impl(a: u8, b: u8) -> Atom {
    let res = a * b;
    Atom::Number(res)
}

fn subtract(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(2, 0)?;
    let arg_2 = stack.try_pop(2, 1)?;
    Ok(subtract_impl(arg_1, arg_2))
}

fn subtract_impl(a: u8, b: u8) -> Atom {
    // No negative numbers
    if a < b {
        return Atom::Number(0);
    }
    let res = a - b;
    Atom::Number(res)
}

fn play(stack: &mut Args) -> Result<Atom, Error> {
    let arg_1 = stack.try_pop(3, 0)?;
    let arg_2 = stack.try_pop(3, 1)?;
    let arg_3 = stack.try_pop(3, 2)?;
    Ok(play_impl(arg_1, arg_2, arg_3))
}

fn play_impl(c: u8, v: u8, n: u8) -> Atom {
    info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Atom::Number(0)
}

#[cfg(test)]
mod test {

    use crate::{
        interpreter::Interpreter, trace, ArgumentError, Atom, Error, Function, Parser, Stack,
        TypeError,
    };
    use arrayvec::ArrayVec;

    macro_rules! stack {
        ($($items:tt),*) => {
            {
                let mut inner: ArrayVec<Atom, 32> = ArrayVec::new();
                $(
                    for item in $items.iter() {
                        inner.push(item.clone());
                    }
                )*
                Stack::new_with(inner)
            }
        };
    }

    fn interpret(exp: String) -> Atom {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse().unwrap();

        let mut interpreter = Interpreter::new(&mut parser);
        interpreter.interpret().unwrap()
    }

    fn interpret_stack(stack: Stack<32>) -> Result<Atom, Error> {
        let mut interpreter = Interpreter::from_stack(stack);
        interpreter.interpret()
    }

    #[test]
    fn test_add_function() {
        trace();

        let s = String::from("++0102");
        let result = interpret(s);

        let expected = Atom::Number(3);
        assert_eq!(result, expected);

        let s = String::from("++id0A01");
        let result = interpret(s);

        let expected = Atom::Number(11);
        assert_eq!(result, expected);

        let s = String::from("++id0Aid01");
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

        let s = String::from("**id0Aid0A");
        let result = interpret(s);

        let expected = Atom::Number(100);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_id_function() {
        trace();

        let s = String::from("id02");
        let result = interpret(s);

        let expected = Atom::String("02".to_string());
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

        let s = String::from("++ididid0901");
        let result = interpret(s);

        let expected = Atom::Number(10);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_missing_argument() {
        trace();

        let array: &[Atom] = &[Atom::Function(Function::Add), Atom::Number(1)];
        let stack = stack!(array);

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
        let vtha = "VTHA".to_string();

        let array: &[Atom] = &[
            Atom::Function(Function::Add),
            Atom::Number(1),
            Atom::String(vtha),
        ];
        let stack = stack!(array);

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }
}
