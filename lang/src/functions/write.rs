use crate::{
    Atom, Error,
    atom::operands::{AbsoluteWrite, WriteEast, WriteNorth, WriteSouth, WriteWest},
    interpreter::Context,
    stack::Operands,
};

/// A Write: `@^ n value`, `@v n value`, `@< n value`, `@> n value` or
/// `@$ column row value`.
///
/// It answers `value`, the encoding it carries, unchanged. The Turn resolves
/// the pair its address operands select and writes the answer there, so
/// evaluation binds the operands and has nothing left to decide.
pub fn write<O: Operands + Carried>(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|operands: O| Ok(operands.value()))
}

/// The operands of a Write, which carry one `value`.
pub(crate) trait Carried {
    /// The encoding the Write carries to its pair.
    fn value(self) -> Atom;
}

macro_rules! carried {
    ($($write:ident),+) => {
        $(impl Carried for $write {
            fn value(self) -> Atom {
                self.value
            }
        })+
    };
}

carried!(AbsoluteWrite, WriteEast, WriteNorth, WriteSouth, WriteWest);

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, Interpretation, Interpreter, Note, Tick, TickInputs,
        TypeError,
    };

    const WRITES: [Function; 5] = [
        Function::AbsoluteWrite,
        Function::WriteEast,
        Function::WriteNorth,
        Function::WriteSouth,
        Function::WriteWest,
    ];

    /// Evaluates `function` with every address operand `01` and `value` last.
    fn write(function: Function, value: Atom) -> Result<Interpretation, Error> {
        let mut operands = vec![Atom::Number(1); crate::Tokens::from(&function).len() - 1];
        operands.push(value);
        Interpreter::execute_function(
            function,
            operands,
            TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
        )
    }

    #[test]
    fn a_write_answers_the_value_it_carries_unchanged() {
        let note = Atom::Note(Note::try_from(67).unwrap());
        for function in WRITES {
            for value in [
                Atom::Number(0xC4),
                note,
                Atom::Bang,
                Atom::Function(Function::Add),
            ] {
                assert_eq!(
                    write(function, value).unwrap(),
                    Interpretation::Cell(value),
                    "{function:?} {value:?}"
                );
            }
        }
    }

    #[test]
    fn a_write_refuses_an_absence_marker_as_its_value() {
        for function in WRITES {
            assert!(
                matches!(
                    write(function, Atom::Empty),
                    Err(Error::Type(TypeError::Unit(_)))
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_write_refuses_an_address_operand_that_is_not_a_number() {
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            Interpreter::execute_function(
                Function::WriteEast,
                [note, Atom::Number(1)],
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
    }
}
