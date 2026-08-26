pub(crate) mod math;
use crate::{interpreter::Context, Atom, Error, InterpretationError, PlayCommand};

#[inline(always)]
pub fn ident(ctx: &mut Context) -> Result<Atom, Error> {
    Ok(ctx.stack.pop().into())
}

#[inline(always)]
pub fn play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let arg_1 = ctx.stack.try_pop(3, 0)?;
    let arg_2 = ctx.stack.try_pop(3, 1)?;
    let arg_3 = ctx.stack.try_pop(3, 2)?;
    play_impl(arg_1, arg_2, arg_3)
}

#[inline(always)]
fn play_impl(channel: u8, velocity: u8, note: u8) -> Result<PlayCommand, Error> {
    if velocity > 0x7F {
        return Err(InterpretationError::PlayVelocity(velocity).into());
    }

    Ok(PlayCommand {
        channel,
        velocity,
        note,
    })
}

#[cfg(test)]
mod test {
    use super::play;
    use crate::{interpreter::Context, ArgumentError, Atom, Error, PlayCommand};

    /// Pins the Play arity contract before issue 04 replaces the placeholder.
    /// See `.scratch/source-playback-engine/issues/04-interpret-terminal-play-functions-into-play-commands.md`
    #[test]
    fn test_play_consumes_exactly_three_arguments() {
        let mut ctx = Context::new();

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Char('z'));
        ctx.stack.push(Atom::Note(60)); // n
        ctx.stack.push(Atom::Number(0x7F)); // v
        ctx.stack.push(Atom::Number(0x0)); // c

        let result = play(&mut ctx).unwrap();

        assert_eq!(
            result,
            PlayCommand {
                channel: 0,
                velocity: 0x7F,
                note: 60,
            }
        );

        // Exactly three arguments were consumed
        assert_eq!(Atom::from(ctx.stack.pop()), Atom::Char('z'));
        assert_eq!(Atom::from(ctx.stack.pop()), Atom::Empty);
    }

    #[test]
    fn test_play_requires_three_arguments() {
        for found in 0..3 {
            let mut ctx = Context::new();
            for _ in 0..found {
                ctx.stack.push(Atom::Number(1));
            }

            let error = play(&mut ctx).unwrap_err();

            assert!(
                matches!(
                    error,
                    Error::Argument(ArgumentError::Arity { expected: 3, found: f }) if f == found
                ),
                "{found} argument(s) gave {error:?}"
            );
        }
    }
}
