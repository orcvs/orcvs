/*

    fn a b c

    fn (a b c) (a b c)

    fn a (fn a fn 1) fn 1

    Function Vec<Atom>

    Function Vec<Vec<Atom>>>

    Expression
    Function(Function, Vec<Atom>)

*/

use crate::{ArgumentError, Atom, AtomRef, Function, VthaError};
use tracing::info;

#[derive(Default)]
struct Interpreter {
    pool: Vec<Atom>,
}

impl Interpreter {
    fn get(&self, atom_ref: AtomRef) -> &Atom {
        &self.pool[atom_ref.0]
    }

    fn add(&mut self, atom: Atom) -> AtomRef {
        let idx = self.pool.len();
        self.pool.push(atom);
        AtomRef(idx.into())
    }

    /// # Errors
    ///
    ///
    pub fn eval(&self) -> Result<Atom, VthaError> {
        let result = match atom {
            Atom::Function(fun) => match *fun {
                Function::Add(a, b) => {
                    let a = eval(a)?.get_num()?;
                    let b = eval(b)?.get_num()?;
                    add(a, b)
                }
                Function::Play(ch, vel, note) => {
                    let ch = eval(ch)?.get_num()?;
                    let vel = eval(vel)?.get_num()?;
                    let note = eval(note)?.get_num()?;
                    play(ch, vel, note)
                }
                Function::Sub(a, b) => {
                    let a = eval(a)?.get_num()?;
                    let b = eval(b)?.get_num()?;
                    sub(a, b)
                }
                Function::Ident(a) => a,
            },
            _ => atom,
        };

        Ok(result)
    }
}

impl Atom {
    fn get_num(&self) -> Result<u8, VthaError> {
        match self {
            Atom::Note(n) | Atom::Number(n) => Ok(*n),
            a => Err(ArgumentError::NumberExpected(a.to_string()).into()),
        }
    }

    fn _get_str(&self) -> Result<&str, VthaError> {
        match self {
            Atom::String(s) => Ok(s),

            a => Err(ArgumentError::StringExpected(a.to_string()).into()),
        }
    }
}

fn add(a: u8, b: u8) -> Atom {
    let res = a + b;
    Atom::Number(res)
}

fn sub(a: u8, b: u8) -> Atom {
    let res = a - b;
    Atom::Number(res)
}

fn play(c: u8, v: u8, n: u8) -> Atom {
    info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Atom::Number(0)
}

#[cfg(test)]
mod test {
    use tracing::error;

    use crate::{trace, ArgumentError, Atom, Function, VthaError};

    use super::eval;

    #[test]
    fn test_add_function() {
        trace();

        let a = Atom::from(Function::Add(Atom::Number(1), Atom::Number(2)));

        let result = eval(a).unwrap();
        let expected = Atom::Number(3);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sub_function() {
        trace();

        let a = Atom::from(Function::Sub(Atom::Number(1), Atom::Number(1)));

        let result = eval(a).unwrap();
        let expected = Atom::Number(0);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_num() {
        trace();

        let a = Atom::Number(1);

        let result = a.get_num().unwrap();
        let expected = 1;

        assert_eq!(result, expected);

        let a = Atom::from("vtha");

        let result = a.get_num().unwrap_err().to_string();
        let expected =
            VthaError::ArgumentError(ArgumentError::NumberExpected("vtha".to_string())).to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_play() {
        trace();
        let a = Atom::from(Function::Play(
            Atom::Number(1),
            Atom::Number(10),
            Atom::Note(60),
        ));

        let result = eval(a).unwrap();

        assert_eq!(result, Atom::Number(0));
    }

    #[test]
    fn test_eval_recursive_function() {
        trace();

        let a = Atom::from(Function::Add(Atom::Number(1), Atom::Number(2)));

        let a = Atom::from(Function::Add(Atom::Number(1), a));

        let a = Atom::from(Function::Add(Atom::Number(1), a));

        let result = eval(a).unwrap();
        let expected = Atom::Number(5);

        assert_eq!(result, expected);
    }
}
