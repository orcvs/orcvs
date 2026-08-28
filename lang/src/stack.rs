use crate::{ArgumentError, Atom, Error, Function, SyntaxError, TypeError, char_to_num};
use arrayvec::ArrayVec;
use std::ops::Deref;

pub struct MaybeAtom(pub Option<Atom>);

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

impl TryFrom<MaybeAtom> for u8 {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Number(n) | Atom::Note(n)) => Ok(n),
            Some(Atom::Char(c)) => char_to_num(c),
            Some(atom) => Err(TypeError::Number(atom.into()).into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for String {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(a) => Ok(a.into()),
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

impl TryFrom<&str> for Function {
    type Error = Error;

    #[inline(always)]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "++" => Ok(Function::Add),
            "//" => Ok(Function::Divide),
            "id" => Ok(Function::Id),
            "**" => Ok(Function::Multiply),
            ">>" => Ok(Function::Play),
            "--" => Ok(Function::Subtract),
            s => Err(SyntaxError::UnknownFunction(s.to_string()).into()),
        }
    }
}

fn map_arity(err: Error, expected: usize, found: usize) -> Error {
    match err {
        Error::Argument(ArgumentError::Expected) => ArgumentError::Arity { expected, found }.into(),
        _ => err,
    }
}
