use crate::{
    ArgumentError, Atom, EXP_LEN, Error, Function, Note, SequenceError, Token, TypeError, Value,
};
use arrayvec::ArrayVec;
use std::ops::Deref;

pub struct MaybeAtom(pub Option<Atom>);

pub(crate) enum NumericValue {
    Note(Note),
    Number(u8),
}

pub(crate) struct Operands {
    inner: ArrayVec<Atom, EXP_LEN>,
}

impl Operands {
    #[inline(always)]
    pub(crate) fn number(&self, index: usize) -> u8 {
        match self.inner[index] {
            Atom::Number(value) => value,
            _ => unreachable!("typed extraction guarantees a Number operand"),
        }
    }

    #[inline(always)]
    pub(crate) fn note(&self, index: usize) -> Note {
        match self.inner[index] {
            Atom::Note(value) => value,
            _ => unreachable!("typed extraction guarantees a Note operand"),
        }
    }
}

/// The operand stack one Expression evaluates against.
///
/// It holds a [`Value`] rather than an `Atom` so a Sequence produced by one
/// Function can be consumed by another without becoming Source writes
/// prematurely. Everything below this line is still scalar: no Function
/// signature accepts a Sequence yet, so the stack's job is to carry one intact
/// and to refuse it wherever a scalar operand is required.
#[derive(Debug)]
pub struct Stack<const N: usize> {
    inner: ArrayVec<Value, N>,
}

impl<const N: usize> Stack<N> {
    pub fn new() -> Self {
        Self {
            inner: ArrayVec::new(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, value: impl Into<Value>) {
        self.inner.push(value.into());
    }

    /// Pops one slot, requiring the scalar Atom every current signature asks
    /// for.
    ///
    /// A Sequence arriving at a scalar operand position is a shape error
    /// today. Issue 02 gives the Atomic Functions a Sequence-aware path and
    /// replaces this diagnostic with broadcasting; the Functions that stay
    /// scalar keep it.
    #[inline(always)]
    pub fn pop(&mut self) -> Result<MaybeAtom, Error> {
        match self.inner.pop() {
            None => Ok(MaybeAtom(None)),
            Some(Value::Atom(atom)) => Ok(MaybeAtom(Some(atom))),
            Some(Value::Sequence(sequence)) => {
                Err(SequenceError::ExpectedAtom(sequence.into()).into())
            }
        }
    }

    /// Pops one slot as the whole language value it is.
    ///
    /// The Interpreter answers with whatever the Expression left here, so a
    /// Sequence leaves evaluation intact instead of being refused as a scalar.
    #[inline(always)]
    pub fn pop_value(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    #[inline(always)]
    pub fn try_pop<T: TryFrom<MaybeAtom, Error = Error>>(
        &mut self,
        expected: usize,
        count: usize,
    ) -> Result<T, Error> {
        self.pop()
            .and_then(TryInto::try_into)
            .map_err(|err| map_arity(err, expected, count))
    }

    /// Pops and validates the operands declared by `function`, in signature order.
    #[inline(always)]
    pub(crate) fn extract(&mut self, function: Function) -> Result<Operands, Error> {
        let signature = function.signature();
        let mut operands = ArrayVec::new();

        for (found, expected) in signature.iter().copied().enumerate() {
            let atom = self.pop()?.0.ok_or_else(|| {
                Error::from(ArgumentError::Arity {
                    expected: signature.len(),
                    found,
                })
            })?;

            match (expected, atom) {
                (Token::Number, Atom::Number(_)) | (Token::Note, Atom::Note(_)) => {}
                (Token::Number, atom) => return Err(TypeError::Number(atom.into()).into()),
                (Token::Note, atom) => return Err(TypeError::Note(atom.into()).into()),
                _ => unreachable!("scalar and terminal signatures contain only typed operands"),
            }
            operands.push(atom);
        }

        Ok(Operands { inner: operands })
    }
}

impl<const N: usize> Default for Stack<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for MaybeAtom {
    type Target = Option<Atom>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MaybeAtom> for Atom {
    #[inline(always)]
    fn from(maybe_atom: MaybeAtom) -> Self {
        match maybe_atom.0 {
            Some(a) => a,
            None => Atom::Empty,
        }
    }
}

impl TryFrom<MaybeAtom> for NumericValue {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Note(n)) => Ok(Self::Note(n)),
            Some(Atom::Number(n)) => Ok(Self::Number(n)),
            Some(atom) => Err(TypeError::Numeric(atom.into()).into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for Function {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Function(f)) => Ok(f),
            _ => Err(ArgumentError::Expected.into()),
        }
    }
}

fn map_arity(err: Error, expected: usize, found: usize) -> Error {
    match err {
        Error::Argument(ArgumentError::Expected) => ArgumentError::Arity { expected, found }.into(),
        _ => err,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ArgumentError, Atom, Error, Function, Note, Sequence, SequenceError, Stack, TypeError,
        Value,
    };

    fn empty_stack() -> Stack<16> {
        Stack::new()
    }

    fn sequence() -> Sequence {
        Sequence::new([Atom::Number(0), Atom::Number(1)]).unwrap()
    }

    #[test]
    fn a_sequence_crosses_function_evaluation_intact() {
        // The point of the seam: what one Function pushes, the next pops
        // unchanged, without ever being encoded for the Source.
        let mut stack = empty_stack();
        stack.push(sequence());

        assert_eq!(stack.pop_value(), Some(Value::Sequence(sequence())));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn a_sequence_diagnoses_where_a_scalar_signature_requires_an_atom() {
        for function in [Function::Add, Function::Play] {
            let mut stack = empty_stack();
            stack.push(sequence());

            assert!(
                matches!(
                    stack.extract(function),
                    Err(Error::Sequence(SequenceError::ExpectedAtom(found))) if found == "0001"
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_sequence_diagnoses_where_a_numeric_conversion_requires_an_atom() {
        let mut stack = empty_stack();
        stack.push(sequence());

        let result: Result<crate::stack::NumericValue, Error> = stack.try_pop(1, 0);

        assert!(matches!(
            result,
            Err(Error::Sequence(SequenceError::ExpectedAtom(found))) if found == "0001"
        ));
    }

    #[test]
    fn scalar_operand_diagnostics_are_unchanged_by_the_sequence_seam() {
        let mut stack = empty_stack();
        stack.push(Atom::Number(1));
        stack.push(Atom::Note(Note::try_from(60).unwrap()));

        assert!(matches!(
            stack.extract(Function::Add),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));

        let mut stack = empty_stack();
        stack.push(Atom::Number(1));

        assert!(matches!(
            stack.extract(Function::Add),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            }))
        ));
    }

    #[test]
    fn an_empty_stack_still_pops_the_absence_marker() {
        let mut stack = empty_stack();

        assert_eq!(Atom::from(stack.pop().unwrap()), Atom::Empty);
        assert_eq!(stack.pop_value(), None);
    }
}
