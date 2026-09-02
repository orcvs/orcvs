pub(crate) mod math;
pub(crate) mod numeric_conversion;
use crate::{Error, Function, InterpretationError, PlayCommand, interpreter::Context};

#[inline(always)]
pub fn play(ctx: &mut Context) -> Result<PlayCommand, Error> {
    let operands = ctx.stack.extract(Function::Play)?;
    play_impl(
        operands.number(0),
        operands.number(1),
        operands.note(2).value(),
    )
}

#[inline(always)]
fn play_impl(channel: u8, velocity: u8, note: u8) -> Result<PlayCommand, Error> {
    if channel > 0x0F {
        return Err(InterpretationError::PlayChannel(channel).into());
    }

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
    use crate::{ArgumentError, Atom, Error, PlayCommand, interpreter::Context};

    /// Pins the Play arity contract before issue 04 replaces the placeholder.
    /// See `.scratch/source-playback-engine/issues/04-interpret-terminal-play-functions-into-play-commands.md`
    #[test]
    fn test_play_consumes_exactly_three_arguments() {
        let mut ctx = Context::new();

        // A fourth atom below the three arguments must survive untouched
        ctx.stack.push(Atom::Char('z'));
        ctx.stack
            .push(Atom::Note(crate::Note::try_from(60).unwrap())); // n
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

    #[test]
    fn play_rejects_implicit_number_note_conversions() {
        for arguments in [
            [
                Atom::Note(crate::Note::try_from(0).unwrap()),
                Atom::Number(0x7F),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ],
            [
                Atom::Number(0),
                Atom::Note(crate::Note::try_from(0x7F).unwrap()),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ],
            [Atom::Number(0), Atom::Number(0x7F), Atom::Number(60)],
        ] {
            let mut ctx = Context::new();
            for argument in arguments.into_iter().rev() {
                ctx.stack.push(argument);
            }

            assert!(matches!(play(&mut ctx), Err(Error::Type(_))));
        }
    }

    #[test]
    fn play_rejects_channels_outside_the_midi_range() {
        for channel in 0x10..=u8::MAX {
            let mut ctx = Context::new();
            for argument in [
                Atom::Number(channel),
                Atom::Number(0x7F),
                Atom::Note(crate::Note::try_from(60).unwrap()),
            ]
            .into_iter()
            .rev()
            {
                ctx.stack.push(argument);
            }

            assert!(matches!(
                play(&mut ctx),
                Err(Error::Interpretation(crate::InterpretationError::PlayChannel(value)))
                    if value == channel
            ));
        }
    }
}
