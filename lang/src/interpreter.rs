use crate::{
    Atom, Atoms, Error, Function, InterpretationError, PlayCommand, Stack,
    functions::{self, math},
};

pub type Args = Stack<16>;

pub struct Interpreter {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interpretation {
    Cell(Atom),
    Play(PlayCommand),
}

pub struct Context {
    pub stack: Args,
}

impl Context {
    pub fn new() -> Self {
        Self { stack: Args::new() }
    }
}

impl Interpreter {
    #[inline(always)]
    pub fn execute(atoms: &Atoms) -> Result<Interpretation, Error> {
        let mut ctx = Context::new();

        for (index, atom) in atoms.iter().enumerate().rev() {
            // info!("atoms: {:?}", atoms);
            // info!("stack: {:?}", stack);
            let atom = match atom {
                Atom::Function(fun) => match fun {
                    Function::Add => math::add(&mut ctx)?,
                    Function::Divide => math::divide(&mut ctx)?,
                    Function::Multiply => math::multiply(&mut ctx)?,
                    Function::Subtract => math::subtract(&mut ctx)?,
                    Function::Id => functions::ident(&mut ctx)?,
                    Function::Play if index == 0 => {
                        return Ok(Interpretation::Play(functions::play(&mut ctx)?));
                    }
                    Function::Play => return Err(InterpretationError::NestedPlay.into()),
                    _ => Atom::Function(Function::Empty),
                },
                atom => *atom,
            };
            ctx.stack.push(atom);
        }

        // A non-terminal Expression leaves one Cell value on the stack. Source
        // decides where that value belongs when it builds the Tick Plan.
        let atom = ctx.stack.pop().into();
        Ok(Interpretation::Cell(atom))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        ArgumentError, Atom, Error, Function, Parser, TypeError, interpreter::Interpreter, trace,
    };
    use tracing::{error, info};

    fn interpret(exp: String) -> Atom {
        let mut exp = exp.clone();
        let parser = Parser::from(&mut exp);
        let parsed = parser.try_parse().unwrap();

        info!("Parsed: {:?}", parsed);

        match Interpreter::execute(&parsed).unwrap() {
            super::Interpretation::Cell(atom) => atom,
            super::Interpretation::Play(_) => panic!("expected a Cell result"),
        }
    }

    fn interpret_stack(exp: Vec<Atom>) -> Result<Atom, Error> {
        let atoms = exp.into_iter().collect();
        Interpreter::execute(&atoms).map(|result| match result {
            super::Interpretation::Cell(atom) => atom,
            super::Interpretation::Play(_) => panic!("expected a Cell result"),
        })
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
    fn test_add_saturates_at_255_on_overflow() {
        trace();

        // 0xFF + 0xFF would overflow u8; saturates at 255 rather than panicking
        let s = String::from("++FFFF");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        // Exact boundary: 0xFF + 0x01 is the first value that cannot be represented
        let s = String::from("++FF01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        // Just below the boundary still computes normally
        let s = String::from("++FE01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        let s = String::from("++FD01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(254));
    }

    #[test]
    fn test_multiply_saturates_at_255_on_overflow() {
        trace();

        // 0x99 * 0x99 == 153 * 153 == 23409; saturates at 255 rather than panicking
        let s = String::from("**9999");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        let s = String::from("**FFFF");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        // Exact boundary: 0x10 * 0x10 == 256 is the first product that cannot be represented
        let s = String::from("**1010");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        // Largest exactly-representable product is computed, not saturated
        let s = String::from("**0F11");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        let s = String::from("**0F10");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(240));
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
