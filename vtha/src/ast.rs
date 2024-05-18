/*

    fn a b c

    fn (a b c) (a b c)

    fn a (fn a fn 1) fn 1

    Function Vec<Atom>

    Function Vec<Vec<Atom>>>

    Expression
    Function(Function, Vec<Atom>)

*/
use crate::ArgumentError;
use crate::Atom;
use crate::Expression;
use crate::Function;
use crate::Play;
use crate::SyntaxError;
use crate::VthaError;

// pub fn to_expression(f: Function, a: Option<Atom>) -> Result<Expression, VthaError> {
//     match f {
//         Function::Play => {
//             let play = play(a)?;
//             Ok(Expression::Play(play))
//         }
//         _ => Err(SyntaxError::UnknownFunction { f: f.to_string() }.into()),
//     }
// }

// midi(ch oct note velocity*):
//channel, octave, note, velocity
fn play(a: Option<Atom>) -> Result<Play, VthaError> {
    const ARITY: usize = 3;
    const INDEX_CHANNEL: usize = 0;
    const INDEX_NOTE: usize = 1;
    const INDEX_VELOCITY: usize = 2;

    let a = match a {
        Some(a) => a,
        None => {
            return Err(ArgumentError::Arity {
                expected: 4,
                found: 0,
            }
            .into())
        }
    };

    match a {
        Atom::List(v) => {
            if v.len() != ARITY {
                return Err(ArgumentError::Arity {
                    expected: 3,
                    found: v.len(),
                }
                .into());
            }
            let ch = match v[INDEX_CHANNEL] {
                Atom::Num(n) => n,
                _ => {
                    return Err(
                        ArgumentError::NumberExpected(
                            "channel",
                        )
                        .into(),
                    )
                }
            };
            let note = match &v[INDEX_NOTE] {
                Atom::String(s) => s.to_owned(),
                _ => {
                    return Err(
                        ArgumentError::StringExpected(
                            "note",
                        )
                        .into(),
                    )
                }
            };
            let vel = match v[INDEX_VELOCITY] {
                Atom::Num(n) => n,
                _ => {
                    return Err(
                        ArgumentError::NumberExpected(
                            "velocity",
                        )
                        .into(),
                    )
                }
            };

            let play = Play::new(ch, note, vel);
            Ok(play)
        }
        _ => {
            return Err(ArgumentError::Arity {
                expected: ARITY,
                found: 1,
            }
            .into())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::play, list, ArgumentError, Atom, Play,
        VthaError,
    };

    #[test]
    fn test_play_function() {
        let params = Some(list![
            Atom::Num(1),
            Atom::String("C4".to_string()),
            Atom::Num(4)
        ]);

        let result = play(params).unwrap();
        assert_eq!(result.note, "C4");

        let params = Some(list![]);
        let result = play(params);

        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(VthaError::ArgumentError(
                ArgumentError::Arity {
                    expected: 3,
                    found: 0
                }
            ))
        ));

        let params = Some(list![Atom::Num(1)]);
        let result = play(params);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(VthaError::ArgumentError(
                ArgumentError::Arity {
                    expected: 3,
                    found: 1
                }
            ))
        ));
    }
}
