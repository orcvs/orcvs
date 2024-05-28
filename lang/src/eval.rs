/*

    fn a b c

    fn (a b c) (a b c)

    fn a (fn a fn 1) fn 1

    Function Vec<Atom>

    Function Vec<Vec<Atom>>>

    Expression
    Function(Function, Vec<Atom>)

*/

use crate::{ArgumentError, Atom, Function, VthaError};
use tracing::{info, warn};

impl Atom {
    fn get_num(&self) -> Result<u8, VthaError> {
        match self {
            Atom::Num(n) => Ok(*n),
            Atom::Note(n) => Ok(*n),
            a => {
                Err(ArgumentError::NumberExpected(a.to_string())
                    .into())
            }
        }
    }

    fn get_str(&self) -> Result<&str, VthaError> {
        match self {
            Atom::String(s) => Ok(s),

            a => {
                Err(ArgumentError::StringExpected(a.to_string())
                    .into())
            }
        }
    }
}

pub fn eval(a: Atom) -> Result<Atom, VthaError> {
    let result = match a {
        Atom::Function(f) => match *f {
            Function::Add(a, b) => {
                let a = eval(a)?.get_num()?;
                let b = eval(b)?.get_num()?;
                add(a, b)?
            }
            Function::Play(c, v, n) => {
                let c = eval(c)?.get_num()?;
                let v = eval(v)?.get_num()?;
                let n = eval(n)?.get_num()?;
                play(c, v, n)?
            }
            Function::Sub(a, b) => {
                let a = eval(a)?.get_num()?;
                let b = eval(b)?.get_num()?;
                sub(a, b)?
            }
            Function::Ident(a) => a,
        },
        _ => a,
    };

    Ok(result)
}

fn add(a: u8, b: u8) -> Result<Atom, VthaError> {
    let res = a + b;
    Ok(Atom::Num(res))
}

fn sub(a: u8, b: u8) -> Result<Atom, VthaError> {
    let res = a - b;
    Ok(Atom::Num(res))
}

fn play(c: u8, v: u8, n: u8) -> Result<Atom, VthaError> {
    info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Ok(Atom::Num(0))
}

#[cfg(test)]
mod test {
    use tracing::error;

    use crate::{trace, ArgumentError, Atom, Function, VthaError};

    use super::eval;

    #[test]
    fn test_add_function() {
        trace();

        let a = Atom::from(Function::Add(Atom::Num(1), Atom::Num(2)));

        let result = eval(a).unwrap();
        let expected = Atom::Num(3);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sub_function() {
        trace();

        let a = Atom::from(Function::Sub(Atom::Num(1), Atom::Num(1)));

        let result = eval(a).unwrap();
        let expected = Atom::Num(0);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_num() {
        trace();

        let a = Atom::Num(1);

        let result = a.get_num().unwrap();
        let expected = 1;

        assert_eq!(result, expected);

        let a = Atom::from("vtha");

        let result = a.get_num().unwrap_err().to_string();
        let expected = VthaError::ArgumentError(
            ArgumentError::NumberExpected("vtha".to_string()),
        )
        .to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_play() {
        trace();
        let a = Atom::from(Function::Play(
            Atom::Num(1),
            Atom::Num(10),
            Atom::Note(60),
        ));

        let result = eval(a).unwrap();

        assert_eq!(result, Atom::Num(0));
    }

    #[test]
    fn test_eval_recursive_function() {
        trace();

        let a = Atom::from(Function::Add(Atom::Num(1), Atom::Num(2)));

        let a = Atom::from(Function::Add(Atom::Num(1), a));

        let a = Atom::from(Function::Add(Atom::Num(1), a));

        let result = eval(a).unwrap();
        let expected = Atom::Num(5);

        assert_eq!(result, expected);
    }
}
