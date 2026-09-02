use crate::{ArgumentError, Atom, EXP_LEN, Error, Function, Note, Token, TypeError};
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

#[derive(Debug)]
pub struct Stack<const N: usize> {
    inner: ArrayVec<Atom, N>,
}

impl<const N: usize> Stack<N> {
    pub fn new() -> Self {
        Self {
            inner: ArrayVec::new(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, atom: Atom) {
        self.inner.push(atom);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> MaybeAtom {
        MaybeAtom(self.inner.pop())
    }

    #[inline(always)]
    pub fn try_pop<T: TryFrom<MaybeAtom, Error = Error>>(
        &mut self,
        expected: usize,
        count: usize,
    ) -> Result<T, Error> {
        self.pop()
            .try_into()
            .map_err(|err| map_arity(err, expected, count))
    }

    /// Pops and validates the operands declared by `function`, in signature order.
    #[inline(always)]
    pub(crate) fn extract(&mut self, function: Function) -> Result<Operands, Error> {
        let signature = function.signature();
        let mut operands = ArrayVec::new();

        for (found, expected) in signature.iter().copied().enumerate() {
            let atom = self.pop().0.ok_or_else(|| {
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
