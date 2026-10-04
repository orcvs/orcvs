use crate::{ArgumentError, Atom, Error, Function, InterpretationError, PlayCommand};
use arrayvec::ArrayVec;

/// The operands one Function declares, named by the role each position plays.
///
/// `define_functions!` generates one implementation per Function from the same
/// table that declares its spelling, kind, and operand types, so a role, its
/// position, and its type are declared together and once. A Function body
/// destructures the struct instead of indexing the operands it was handed,
/// which is what leaves the declaration as the only place an operand order
/// exists.
///
/// Each role's type is an [`crate::operand::Operand`], and every method below
/// reads a role through that type's token and binds exactly the payload the
/// token yields. There is no reading a role's token can disagree with.
pub(crate) trait Operands: Sized {
    /// The Function whose signature these operands are extracted against.
    const FUNCTION: Function;

    /// Checks every operand against its role's token, in signature order.
    fn check(operands: &[Atom]) -> Result<(), Error>;

    /// Binds each declared role to its operand, in signature order.
    ///
    /// Fallible because a declared operand type may be narrower than the
    /// `Token` the signature checks: a MIDI channel is read as a Number and is
    /// a channel only once its domain conversion succeeds. Every arity and
    /// type diagnostic is already raised by the time this runs, so a domain
    /// diagnostic can never displace one.
    fn bind(operands: &[Atom]) -> Result<Self, Error>;
}

/// The arity diagnostic for `function` handed `found` operands.
///
/// Cold: every extraction pops exactly the declared count before a bind reads
/// it, so the generated binds reach this only if that ever stops holding.
#[cold]
pub(crate) fn arity(function: Function, found: usize) -> Error {
    ArgumentError::Arity {
        expected: function.signature().len(),
        found,
    }
    .into()
}

/// The widest operand list any Function declares, and the capacity of every
/// per-operation buffer below.
///
/// Read off the Function table rather than written down beside it, so a
/// Function that declared a fifth operand would widen these buffers by being
/// declared rather than by someone remembering to. An operand list is bounded
/// by its Function's signature, independently of the Expression's Atom count.
const MAX_OPERANDS: usize = {
    let mut widest = 0;
    let mut index = 0;

    while index < Function::ALL.len() {
        let declared = Function::ALL[index].signature().len();

        if declared > widest {
            widest = declared;
        }

        index += 1;
    }

    widest
};

/// The operand stack one Function evaluation runs against.
///
/// It holds one Atom per operand: every operand a Function reads is a single
/// two-Cell value, whether an Operand Literal supplied it or a nested Function
/// answered it.
///
/// The storage is inline, `MAX_OPERANDS` slots, so building one for a Turn
/// asks the allocator for nothing. That capacity is sufficient because the
/// stack holds one Function's operands and no answer: the Interpreter refuses
/// an operand count other than the Function's signature length before
/// building the stack, and no signature is longer than `MAX_OPERANDS`.
#[derive(Debug)]
pub struct Stack {
    inner: ArrayVec<Atom, MAX_OPERANDS>,
    /// Never above `MAX_OPERANDS`, so every push [`Stack::push`] admits has an
    /// inline slot and exhaustion is always its diagnostic, never a panic.
    limit: usize,
}

impl Stack {
    /// An empty stack that admits at most `limit` values, clamped to the
    /// inline capacity so the limit is one the storage can always honour.
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            inner: ArrayVec::new(),
            limit: limit.min(MAX_OPERANDS),
        }
    }

    /// The stack for one Function's `operands`, given in signature order.
    ///
    /// Pushed last to first, so the declared pops take them back in
    /// signature order. The limit is the operand count, so an operand list
    /// longer than the inline capacity answers `OperandStackExhausted` rather
    /// than panicking.
    #[inline(always)]
    pub(crate) fn with_operands<I>(operands: I) -> Result<Self, Error>
    where
        I: DoubleEndedIterator<Item = Atom> + ExactSizeIterator,
    {
        let mut stack = Self::new(operands.len());
        for operand in operands.rev() {
            stack.push(operand)?;
        }
        Ok(stack)
    }

    /// Pushes one value, diagnosing a stack with no slot left.
    #[inline(always)]
    pub(crate) fn push(&mut self, atom: Atom) -> Result<(), Error> {
        if self.inner.len() == self.limit {
            return Err(InterpretationError::OperandStackExhausted {
                capacity: self.limit,
            }
            .into());
        }
        self.inner.push(atom);
        Ok(())
    }

    /// Pops one slot, so a test can read what a Function left behind.
    /// Evaluation consumes operands only through the declared pops below.
    #[cfg(test)]
    pub(crate) fn pop_value(&mut self) -> Option<Atom> {
        self.inner.pop()
    }

    /// Pops and validates the operands `O` declares, in signature order.
    ///
    /// Arity first, then every operand's type, then every operand's domain:
    /// a diagnostic about the operand list as a whole precedes one about any
    /// operand, and a type fault precedes a domain fault wherever either
    /// stands. Each stage walks the operands in signature order, so of two
    /// faulty operands the earlier one is what the Source is told about.
    #[inline(always)]
    pub(crate) fn extract<O: Operands>(&mut self) -> Result<O, Error> {
        let expected = O::FUNCTION.signature().len();
        // One Atom per operand the signature declares, and `MAX_OPERANDS` is
        // the widest signature the table holds, so the push below is total: no
        // Function can declare an operand list this buffer cannot take.
        let mut operands: ArrayVec<Atom, MAX_OPERANDS> = ArrayVec::new();

        for found in 0..expected {
            operands.push(
                self.inner
                    .pop()
                    .ok_or(ArgumentError::Arity { expected, found })?,
            );
        }

        O::check(&operands)?;
        O::bind(&operands)
    }

    /// Evaluates one value Function over its declared operands.
    ///
    /// `answer` states what the Function is for the operands it binds, and
    /// runs only after every operand has passed its type and domain checks,
    /// so an evaluation fault never displaces an operand fault.
    #[inline(always)]
    pub(crate) fn apply<O, F>(&mut self, answer: F) -> Result<Atom, Error>
    where
        O: Operands,
        F: FnOnce(O) -> Result<Atom, Error>,
    {
        answer(self.extract::<O>()?)
    }

    /// Performs one Terminal Output Function over its declared operands.
    ///
    /// The effect twin of [`Stack::apply`]: `command` states one Play Command
    /// exactly as an arithmetic body states one Atom, and runs only once
    /// every operand has bound, so a Function that diagnoses performs nothing.
    #[inline(always)]
    pub(crate) fn perform<O, F>(&mut self, command: F) -> Result<PlayCommand, Error>
    where
        O: Operands,
        F: FnOnce(O) -> Result<PlayCommand, Error>,
    {
        command(self.extract::<O>()?)
    }

    /// Evaluates one predicate over its declared operands, answering a Bang
    /// where it holds and the Absence Marker where it does not.
    #[inline(always)]
    pub(crate) fn predicate<O, F>(&mut self, holds: F) -> Result<Atom, Error>
    where
        O: Operands,
        F: FnOnce(O) -> bool,
    {
        Ok(if holds(self.extract::<O>()?) {
            Atom::Bang
        } else {
            Atom::Empty
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, ArgumentError, Atom, BendLsb, BendMsb, ControlValue, Controller, Error, Function,
        InterpretationError, MidiChannel, Note, PlayCommand, Stack, Tick, TickInputs, TypeError,
        Velocity,
        atom::operands,
        functions::{self, math, numeric_conversion},
        interpreter::Context,
        stack::MAX_OPERANDS,
    };

    fn empty_stack() -> Stack {
        Stack::new(MAX_OPERANDS)
    }

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    /// Runs a Function body the Interpreter dispatches over `stack`, so each
    /// fixture below is that body rather than a restatement of it that could
    /// drift from it. The Tick inputs are fixed because no body used here reads
    /// them.
    fn on_stack<T>(
        stack: &mut Stack,
        body: fn(&mut Context<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let mut ctx = Context {
            stack: std::mem::replace(stack, Stack::new(0)),
            inputs: TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
        };
        let answer = body(&mut ctx);
        *stack = ctx.stack;
        answer
    }

    /// Subtraction: `math::subtract`, whose operands are not interchangeable.
    fn difference(stack: &mut Stack) -> Result<Atom, Error> {
        on_stack(stack, math::subtract)
    }

    /// Division: `math::divide`, the one arithmetic Function with an operand
    /// pair that has no answer, which is what makes an evaluation fault
    /// observable.
    fn quotient(stack: &mut Stack) -> Result<Atom, Error> {
        on_stack(stack, math::divide)
    }

    /// Equality: `math::equality`, the predicate.
    fn equal(stack: &mut Stack) -> Result<Atom, Error> {
        on_stack(stack, math::equality)
    }

    /// `.^`: `numeric_conversion::to_note`.
    fn to_note(stack: &mut Stack) -> Result<Atom, Error> {
        on_stack(stack, numeric_conversion::to_note)
    }

    /// Raw Play: `functions::raw_play`.
    fn play(stack: &mut Stack) -> Result<PlayCommand, Error> {
        on_stack(stack, functions::raw_play)
    }

    /// Control Change: `functions::control_change`.
    fn control_change(stack: &mut Stack) -> Result<PlayCommand, Error> {
        on_stack(stack, functions::control_change)
    }

    /// Pitch Bend: `functions::pitch_bend`.
    fn pitch_bend(stack: &mut Stack) -> Result<PlayCommand, Error> {
        on_stack(stack, functions::pitch_bend)
    }

    /// One Control Change Command, from the bytes a Source would have written.
    fn cc(channel: u8, controller: u8, value: u8) -> PlayCommand {
        PlayCommand::ControlChange {
            channel: MidiChannel::try_from(channel).unwrap(),
            controller: Controller::try_from(controller).unwrap(),
            value: ControlValue::try_from(value).unwrap(),
        }
    }

    /// One Pitch Bend Command, from the bytes a Source would have written.
    fn bend(channel: u8, lsb: u8, msb: u8) -> PlayCommand {
        PlayCommand::PitchBend {
            channel: MidiChannel::try_from(channel).unwrap(),
            lsb: BendLsb::try_from(lsb).unwrap(),
            msb: BendMsb::try_from(msb).unwrap(),
        }
    }

    /// One Raw Play Command, from the bytes a Source would have written.
    fn raw(channel: u8, velocity: u8, note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(channel).unwrap(),
            velocity: Velocity::try_from(velocity).unwrap(),
            note: Note::try_from(note).unwrap(),
        }
    }

    /// Pushes `operands` so extraction pops them in signature order.
    fn push_all(stack: &mut Stack, operands: impl IntoIterator<Item = Atom>) {
        let operands: Vec<Atom> = operands.into_iter().collect();
        for operand in operands.into_iter().rev() {
            stack.push(operand).unwrap();
        }
    }

    #[test]
    fn no_function_declares_more_operands_than_the_inline_storage_holds() {
        // Extraction sizes its buffer to the widest signature rather than to
        // Expression length, and `ArrayVec::push` panics on overflow — inside a
        // Tick, under the Source write guard ADR 0028 rules that out. The
        // capacity is derived from the same table the signatures come from, so
        // this reads that table a second way rather than restating a number.
        let widest = Function::ALL
            .iter()
            .map(|function| function.signature().len())
            .max()
            .expect("the Function table declares at least one Function");

        assert_eq!(
            MAX_OPERANDS, widest,
            "a declared operand list outgrows the buffer extraction pops it into"
        );
    }

    #[test]
    fn a_play_answers_one_command_and_consumes_its_operands() {
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x01), Atom::Number(0x7F), note(60)],
        );

        assert_eq!(play(&mut stack).unwrap(), raw(0x01, 0x7F, 60));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn a_control_change_and_a_bend_bind_each_data_byte_to_its_own_role() {
        // Every operand value differs from every other, so a swap of the two
        // data-byte roles answers a different command.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x01), Atom::Number(0x07), Atom::Number(0x40)],
        );

        assert_eq!(control_change(&mut stack).unwrap(), cc(0x01, 0x07, 0x40));

        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x03), Atom::Number(0x2A), Atom::Number(0x33)],
        );

        assert_eq!(pitch_bend(&mut stack).unwrap(), bend(0x03, 0x2A, 0x33));
    }

    #[test]
    fn an_arithmetic_function_answers_one_atom() {
        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(0x20), Atom::Number(0x02)]);

        assert_eq!(difference(&mut stack).unwrap(), Atom::Number(0x1E));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn a_predicate_answers_a_bang_or_the_absence_marker() {
        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(0x20), Atom::Number(0x20)]);
        assert_eq!(equal(&mut stack).unwrap(), Atom::Bang);

        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(0x20), Atom::Number(0x21)]);
        assert_eq!(equal(&mut stack).unwrap(), Atom::Empty);
    }

    #[test]
    fn an_operation_type_checks_its_operands_before_it_evaluates() {
        // `./ C4 00` is mistyped in its left operand and has no quotient in its
        // right, so a path that bound and evaluated before checking the operand
        // list would answer `DivisionByZero`.
        let mut stack = empty_stack();
        push_all(&mut stack, [note(60), Atom::Number(0)]);

        assert!(
            matches!(
                quotient(&mut stack),
                Err(Error::Type(TypeError::Number(found))) if found == "C4"
            ),
            "a type fault was displaced by an evaluation fault"
        );
    }

    #[test]
    fn an_evaluation_fault_answers_the_fault_the_function_raised() {
        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(0x10), Atom::Number(0x00)]);

        assert!(matches!(
            quotient(&mut stack),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn a_conversion_over_one_atom_answers_an_ordinary_atom() {
        let mut stack = empty_stack();
        stack.push(Atom::Number(0x3C)).unwrap();

        assert_eq!(to_note(&mut stack).unwrap(), note(0x3C));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn an_evaluation_fault_in_a_conversion_answers_the_fault_the_conversion_raised() {
        // `80` names no MIDI Note.
        let mut stack = empty_stack();
        stack.push(Atom::Number(0x80)).unwrap();

        assert!(matches!(
            to_note(&mut stack),
            Err(Error::Interpretation(InterpretationError::NoteConversion(
                0x80
            )))
        ));
    }

    #[test]
    fn a_non_numeric_operand_diagnoses_where_a_numeric_conversion_pops_it() {
        // `TypeError::Numeric` is reachable from Source as `.^.=0101`: equal
        // operands make `.=` answer a Bang, which `.^` then pops.
        let mut stack = empty_stack();
        stack.push(Atom::Bang).unwrap();

        assert!(matches!(
            to_note(&mut stack),
            Err(Error::Type(TypeError::Numeric(found))) if found == "**"
        ));
    }

    #[test]
    fn operand_diagnostics_name_the_type_or_the_missing_operand() {
        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(1), note(60)]);

        assert!(matches!(
            stack.extract::<operands::Add>(),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));

        let mut stack = empty_stack();
        stack.push(Atom::Number(1)).unwrap();

        assert!(matches!(
            stack.extract::<operands::Add>(),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            }))
        ));
    }

    #[test]
    fn every_arity_and_type_diagnostic_precedes_every_domain_diagnostic() {
        // Each case below supplies an operand that is out of its domain *and*
        // a second fault the earlier stage sees; the earlier stage's diagnostic
        // must win.

        // Too few operands, with the one supplied outside the channel domain.
        let mut stack = empty_stack();
        stack.push(Atom::Number(0xFF)).unwrap();

        assert!(
            matches!(
                play(&mut stack),
                Err(Error::Argument(ArgumentError::Arity {
                    expected: 3,
                    found: 1
                }))
            ),
            "an arity fault was displaced by a domain fault"
        );

        // Every operand present, the note operand mistyped, and both Numbers
        // outside their domains.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0xFF), Atom::Number(0xFF), Atom::Number(60)],
        );

        assert!(
            matches!(
                play(&mut stack),
                Err(Error::Type(TypeError::Note(found))) if found == "3C"
            ),
            "a type fault was displaced by a domain fault"
        );

        // With nothing left for the earlier stages to answer, the domain fault
        // is reached, which is what makes the two cases above meaningful.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0xFF), Atom::Number(0xFF), note(60)],
        );

        assert!(matches!(
            play(&mut stack),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0xFF
            )))
        ));
    }

    #[test]
    fn a_domain_diagnostic_names_the_first_operand_in_signature_order() {
        // Two operands out of their domains at once: the earlier role in the
        // declaration is the one the Source is told about.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x10), Atom::Number(0x80), note(60)],
        );

        assert!(matches!(
            play(&mut stack),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
    }

    #[test]
    fn an_exhausted_operand_stack_diagnoses_rather_than_panicking() {
        // The Interpreter sizes the stack so no Expression the parser accepts
        // can reach this, and the answer exists anyway: the Evaluator runs
        // inside a Tick under the Source write guard, where a panic costs
        // Playback rather than the Expression. A two-slot stack states the
        // behaviour independently of any Expression's size.
        let mut stack = Stack::new(2);
        stack.push(Atom::Number(0)).unwrap();
        stack.push(Atom::Number(1)).unwrap();

        assert!(matches!(
            stack.push(Atom::Number(2)),
            Err(Error::Interpretation(
                InterpretationError::OperandStackExhausted { capacity: 2 }
            ))
        ));

        // The refused value displaced nothing already on the stack.
        assert_eq!(stack.pop_value(), Some(Atom::Number(1)));
        assert_eq!(stack.pop_value(), Some(Atom::Number(0)));
    }

    #[test]
    fn an_operand_list_wider_than_the_inline_storage_diagnoses_rather_than_panicking() {
        // The storage holds `MAX_OPERANDS` values and `ArrayVec::push` panics
        // past that. The Interpreter never asks for more, and a caller that
        // did still gets the diagnostic: the limit is clamped to the storage,
        // so the checked push refuses before the storage could overflow.
        let operands = (0..MAX_OPERANDS + 1).map(|number| Atom::Number(number as u8));

        assert!(matches!(
            Stack::with_operands(operands),
            Err(Error::Interpretation(
                InterpretationError::OperandStackExhausted { capacity }
            )) if capacity == MAX_OPERANDS
        ));
    }

    #[test]
    fn operands_pop_back_in_signature_order() {
        let mut stack =
            Stack::with_operands([Atom::Number(1), Atom::Number(2), Atom::Number(3)].into_iter())
                .unwrap();

        assert_eq!(stack.pop_value(), Some(Atom::Number(1)));
        assert_eq!(stack.pop_value(), Some(Atom::Number(2)));
        assert_eq!(stack.pop_value(), Some(Atom::Number(3)));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn an_empty_stack_pops_no_value() {
        let mut stack = empty_stack();

        assert_eq!(stack.pop_value(), None);
    }
}
