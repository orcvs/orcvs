use std::io::Empty;

#[allow(unused)]
/*

    fn a b c

    fn (a b c) (a b c)

    fn a (fn a fn 1) fn 1

    Function Vec<Atom>

    Function Vec<Vec<Atom>>>

    Expression
    Function(Function, Vec<Atom>)

*/
use crate::{new_stack, ArgumentError, Atom, Error, Function, TypeError, VecStack};
use tracing::info;

#[derive(Default)]
pub struct Interpreter {
    pub expressions: VecStack,
}

pub struct Stack {
    inner: VecStack,
}

pub struct MaybeAtom(Option<Atom>);

impl Stack {
    pub fn new() -> Self {
        Self { inner: new_stack() }
    }

    #[inline(always)]
    fn push(&mut self, atom: Atom) {
        self.inner.push(atom);
    }

    #[inline(always)]
    fn pop(&mut self) -> MaybeAtom {
        MaybeAtom(self.inner.pop())
    }

    #[inline(always)]
    pub fn pop_arg<T: TryFrom<MaybeAtom, Error = Error>>(
        &mut self,
        expected: usize,
        count: usize,
    ) -> Result<T, Error> {
        self.pop()
            .try_into()
            .map_err(|err| map_arity(err, expected, count))
    }
}

impl<'a> Interpreter {
    pub fn new(pool: VecStack) -> Self {
        Self { expressions: pool }
    }

    #[inline(always)]
    pub fn interpret(&mut self) -> Result<Atom, Error> {
        let mut stack = Stack::new();
        for expression in self.expressions.drain(..) {
            let atom = match expression {
                Atom::Function(fun) => match fun {
                    Function::Add => add(&mut stack)?,
                    Function::Divide => divide(&mut stack)?,
                    Function::Multiply => multiply(&mut stack)?,
                    Function::Subtract => subtract(&mut stack)?,
                    Function::Ident => ident(&mut stack)?,
                    Function::Play => play(&mut stack)?,
                    _ => Atom::Function(Function::Empty),
                },
                _ => expression,
            };
            stack.push(atom);
        }

        // Final element in stack is the result
        let atom = stack.pop().try_into()?;
        Ok(atom)
    }
}

#[inline(always)]
fn map_arity(err: Error, expected: usize, found: usize) -> Error {
    match err {
        Error::Argument(ArgumentError::Expected) => ArgumentError::Arity { expected, found }.into(),
        _ => err.into(),
    }
}

fn ident(stack: &mut Stack) -> Result<Atom, Error> {
    stack.pop().try_into()
}

fn add(stack: &mut Stack) -> Result<Atom, Error> {
    let arg_2 = stack.pop_arg(2, 0)?;
    let arg_1 = stack.pop_arg(2, 1)?;
    Ok(add_impl(arg_1, arg_2))
}

fn add_impl(a: u8, b: u8) -> Atom {
    let res: u8 = a + b;
    Atom::Number(res)
}

fn divide(stack: &mut Stack) -> Result<Atom, Error> {
    let arg_2 = stack.pop_arg(2, 0)?;
    let arg_1 = stack.pop_arg(2, 1)?;
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

fn multiply(stack: &mut Stack) -> Result<Atom, Error> {
    let arg_2 = stack.pop_arg(2, 0)?;
    let arg_1 = stack.pop_arg(2, 1)?;
    Ok(multiply_impl(arg_1, arg_2))
}

fn multiply_impl(a: u8, b: u8) -> Atom {
    let res = a * b;
    Atom::Number(res)
}

fn subtract(stack: &mut Stack) -> Result<Atom, Error> {
    let arg_2 = stack.pop_arg(2, 0)?;
    let arg_1 = stack.pop_arg(2, 1)?;
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

fn play(stack: &mut Stack) -> Result<Atom, Error> {
    let arg_3 = stack.pop_arg(3, 0)?;
    let arg_2 = stack.pop_arg(3, 1)?;
    let arg_1 = stack.pop_arg(3, 2)?;
    Ok(play_impl(arg_1, arg_2, arg_3))
}

fn play_impl(c: u8, v: u8, n: u8) -> Atom {
    info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Atom::Number(0)
}

impl TryFrom<MaybeAtom> for u8 {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Number(num) | Atom::Note(num)) => Ok(num),
            Some(atom) => Err(TypeError::Number(atom.into()).into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for String {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::String(s)) => Ok(s),
            Some(atom) => Err(TypeError::String(atom.into()).into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for Atom {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(a) => Ok(a),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        interpreter::Interpreter, test::stack_from, trace, ArgumentError, Atom, Error, Function,
        Parser, TypeError, VecStack,
    };

    fn interpret(exp: String) -> Atom {
        let mut exp = exp.clone();
        let mut parser = Parser::new(&mut exp);
        parser.try_parse().unwrap();

        let mut interpreter = Interpreter::new(parser.stack);
        interpreter.interpret().unwrap()
    }

    fn interpret_stack(stack: VecStack) -> Result<Atom, Error> {
        let mut interpreter = Interpreter::new(stack);
        interpreter.interpret()
    }

    #[test]
    fn test_add_function() {
        trace();

        let s = String::from("++0102");
        let result = interpret(s);

        let expected = Atom::Number(3);
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
    fn test_with_missing_argument() {
        trace();

        let stack = stack_from(&[Atom::Number(1), Atom::Function(Function::Add)]);

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
        let stack = stack_from(&[
            Atom::String(vtha),
            Atom::Number(1),
            Atom::Function(Function::Add),
        ]);

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    // #[test]
    // fn test_sub_function() {
    //     trace();

    //     let a = Atom::from(Function::Sub(Atom::Number(1), Atom::Number(1)));

    //     let result = eval(a).unwrap();
    //     let expected = Atom::Number(0);

    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_get_num() {
    //     trace();

    //     let a = Atom::Number(1);

    //     let result = a.get_num().unwrap();
    //     let expected = 1;

    //     assert_eq!(result, expected);

    //     let a = Atom::from("vtha");

    //     let result = a.get_num().unwrap_err().to_string();
    //     let expected =
    //         VthaError::ArgumentError(ArgumentError::NumberExpected("vtha".to_string())).to_string();

    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_play() {
    //     trace();
    //     let a = Atom::from(Function::Play(
    //         Atom::Number(1),
    //         Atom::Number(10),
    //         Atom::Note(60),
    //     ));

    //     let result = eval(a).unwrap();

    //     assert_eq!(result, Atom::Number(0));
    // }

    // #[test]
    // fn test_eval_recursive_function() {
    //     trace();

    //     let a = Atom::from(Function::Add(Atom::Number(1), Atom::Number(2)));

    //     let a = Atom::from(Function::Add(Atom::Number(1), a));

    //     let a = Atom::from(Function::Add(Atom::Number(1), a));

    //     let result = eval(a).unwrap();
    //     let expected = Atom::Number(5);

    //     assert_eq!(result, expected);
    // }
}
