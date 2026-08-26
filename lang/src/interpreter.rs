use crate::{
    functions::{self, math},
    Atom, Atoms, Error, Function, Portal, Stack,
};
use arrayvec::ArrayVec;
use tracing::{error, info, warn};

pub type Args = Stack<16>;

pub struct Interpreter {}

pub struct Context {
    pub stack: Args,
    pub portals: ArrayVec<Portal, 16>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack: Args::new(),
            portals: ArrayVec::new(),
        }
    }

    pub fn push_portal(&mut self, portal: Portal) {
        self.portals.push(portal);
    }
}

impl Interpreter {
    #[inline(always)]
    pub fn execute(atoms: &Atoms) -> Result<Atom, Error> {
        let mut ctx = Context::new();

        for atom in atoms.iter().rev() {
            // info!("atoms: {:?}", atoms);
            // info!("stack: {:?}", stack);
            let atom = match atom {
                Atom::Function(fun) => match fun {
                    Function::Add => math::add(&mut ctx)?,
                    Function::Divide => math::divide(&mut ctx)?,
                    Function::Multiply => math::multiply(&mut ctx)?,
                    Function::Subtract => math::subtract(&mut ctx)?,
                    Function::Id => functions::ident(&mut ctx)?,
                    Function::Play => functions::play(&mut ctx)?,
                    _ => Atom::Function(Function::Empty),
                },
                atom => *atom,
            };
            ctx.stack.push(atom);
        }

        let atom = ctx.stack.pop().into();
        // let portal = Portal::new(ctx.stack.pop().into(), 0, 0);

        // warn!("{:?}", portal);

        // Final element in stack is the result
        // let atom = stack.pop().into();
        // Ok(portal)
        Ok(atom)
    }
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
        result.unwrap()
    }

    fn interpret_stack(exp: Vec<Atom>) -> Result<Atom, Error> {
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
