use crate::{Atom, Atoms, EXP_LEN, Function, SyntaxError};
use arrayvec::ArrayVec;
use std::mem;

const DEFAULT_TOKEN_LEN: usize = 2;
const DEFAULT_CHAR_TOKEN_LEN: usize = 1;

pub type Tokens = ArrayVec<Token, EXP_LEN>;

#[derive(Debug, Clone)]
pub struct Expression {
    pub tokens: Tokens,
    pub atoms: Atoms,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Activation,
    Bang,
    Function,
    Note,
    Number,
    NumberN(usize),
    Char,
}

pub type T = Token;

impl Expression {
    pub fn new() -> Self {
        let tokens = ArrayVec::new();
        let atoms = ArrayVec::new();

        Self { tokens, atoms }
    }

    ///
    /// Adds a token and atom to the expression
    /// Should always be added together to keep collections in sync
    ///
    pub fn add(&mut self, t: Token, a: Atom) -> Result<(), SyntaxError> {
        self.tokens
            .try_push(t)
            .map_err(|_| SyntaxError::ExpressionTooLong { capacity: EXP_LEN })?;
        self.atoms
            .try_push(a)
            .map_err(|_| SyntaxError::ExpressionTooLong { capacity: EXP_LEN })?;
        Ok(())
    }

    pub fn take_atoms(&mut self) -> Atoms {
        let atoms = mem::take(&mut self.atoms);
        atoms.into_iter().collect()
    }

    pub fn take_tokens(&mut self) -> Vec<Token> {
        let tokens = mem::take(&mut self.tokens);
        tokens.into_iter().collect()
    }

    pub fn len(&self) -> usize {
        self.tokens.len().max(self.atoms.len())
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.atoms.is_empty()
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self::new()
    }
}

impl Token {
    pub fn len(&self) -> usize {
        match self {
            Token::NumberN(n) => *n,
            Token::Char => DEFAULT_CHAR_TOKEN_LEN,
            _ => DEFAULT_TOKEN_LEN,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<&Function> for Tokens {
    #[inline(always)]
    fn from(f: &Function) -> Self {
        let tokens = match f {
            Function::Add => vec![T::Number, T::Number],
            Function::ConvertToNote => vec![T::Number],
            Function::ConvertToNumber => vec![T::Note],
            Function::Divide => vec![T::Number, T::Number],
            Function::Play => vec![T::NumberN(1), T::Number, T::Note],
            Function::Multiply => vec![T::Number, T::Number],
            Function::Subtract => vec![T::Number, T::Number],
            _ => vec![],
        };

        tokens.into_iter().collect()
    }
}
