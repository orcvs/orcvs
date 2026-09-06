use crate::{Atom, Atoms, EXP_LEN, Function, SyntaxError};
use arrayvec::ArrayVec;

const DEFAULT_TOKEN_LEN: usize = 2;
const DEFAULT_CHAR_TOKEN_LEN: usize = 1;

pub type Tokens = ArrayVec<Token, EXP_LEN>;

#[derive(Debug, Clone)]
pub struct Expression {
    records: ArrayVec<Record, EXP_LEN>,
}

#[derive(Debug, Clone, Copy)]
enum Record {
    Evaluable { token: Token, atom: Atom },
    Incomplete { expected: Token },
    Invalid { expected: Token },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Activation,
    Bang,
    Function,
    Note,
    Number,
    Char,
}

impl Expression {
    pub fn new() -> Self {
        Self {
            records: ArrayVec::new(),
        }
    }

    /// Adds one complete syntax-and-value entry to the Expression.
    pub fn add(&mut self, token: Token, atom: Atom) -> Result<(), SyntaxError> {
        self.records
            .try_push(Record::Evaluable { token, atom })
            .map_err(|_| SyntaxError::ExpressionTooLong { capacity: EXP_LEN })?;
        Ok(())
    }

    pub(crate) fn add_incomplete(&mut self, expected: Token) -> Result<(), SyntaxError> {
        self.records
            .try_push(Record::Incomplete { expected })
            .map_err(|_| SyntaxError::ExpressionTooLong { capacity: EXP_LEN })
    }

    pub(crate) fn add_invalid(&mut self, expected: Token) -> Result<(), SyntaxError> {
        self.records
            .try_push(Record::Invalid { expected })
            .map_err(|_| SyntaxError::ExpressionTooLong { capacity: EXP_LEN })
    }

    /// Complete evaluable entries, with their syntax and runtime value paired.
    pub fn entries(&self) -> impl Iterator<Item = (Token, Atom)> + '_ {
        self.records.iter().filter_map(|record| record.entry())
    }

    /// Every parser-owned slot as its source-relative Cell offset, expected
    /// syntax, and optional value. Missing and invalid operands retain their
    /// width, so later entries never slide into an earlier operand's position.
    /// Missing tail slots can extend beyond the supplied Source fragment.
    pub fn layout(&self) -> impl Iterator<Item = (usize, Token, Option<Atom>)> + '_ {
        self.records.iter().scan(0, |offset, record| {
            let token = record.token();
            let entry = (*offset, token, record.atom());
            *offset += token.len();
            Some(entry)
        })
    }

    /// Reads current operand Cells through this Expression's original layout.
    /// Function and standalone control spellings must retain their identities;
    /// newly written Function spellings cannot change an operand's syntax.
    /// The caller supplies Source from the original anchor, within one row.
    /// Trailing Cells are outside this layout and are not interpreted.
    pub fn bind_source(&self, source: &str) -> Result<Atoms, crate::Error> {
        self.layout()
            .map(|(offset, token, original)| {
                let spelling = source
                    .get(offset..offset + token.len())
                    .ok_or(SyntaxError::ExpectedToken)?;
                match token {
                    Token::Number => crate::to_atom_num(spelling),
                    Token::Note => crate::to_atom_note(spelling),
                    Token::Char => crate::atom::to_atom_char(spelling),
                    Token::Function | Token::Bang | Token::Activation => match original {
                        Some(atom) if atom.to_string() == spelling => Ok(atom),
                        _ => Err(crate::TypeError::Function(spelling.to_owned()).into()),
                    },
                }
            })
            .collect()
    }

    pub fn atoms(&self) -> Option<Atoms> {
        self.records.iter().copied().map(Record::atom).collect()
    }

    pub fn take_atoms(self) -> Option<Atoms> {
        self.records.into_iter().map(Record::atom).collect()
    }

    pub fn tokens(&self) -> impl DoubleEndedIterator<Item = Token> + '_ {
        self.records.iter().map(Record::token)
    }

    pub fn take_tokens(self) -> Vec<Token> {
        self.records
            .into_iter()
            .map(|record| record.token())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Record {
    fn entry(&self) -> Option<(Token, Atom)> {
        match self {
            Self::Evaluable { token, atom } => Some((*token, *atom)),
            Self::Incomplete { .. } | Self::Invalid { .. } => None,
        }
    }

    fn atom(self) -> Option<Atom> {
        self.entry().map(|(_, atom)| atom)
    }

    fn token(&self) -> Token {
        match self {
            Self::Evaluable { token, .. } => *token,
            Self::Incomplete { expected } => *expected,
            Self::Invalid { expected } => *expected,
        }
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
        f.signature().iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Expression, Token};
    use crate::{Atom, EXP_LEN, SyntaxError};

    #[test]
    fn bounded_entries_add_syntax_and_value_atomically() {
        let mut expression = Expression::new();
        for _ in 0..EXP_LEN {
            expression.add(Token::Number, Atom::Number(1)).unwrap();
        }

        assert!(matches!(
            expression.add(Token::Note, Atom::Note(crate::Note::try_from(60).unwrap())),
            Err(SyntaxError::ExpressionTooLong { capacity: EXP_LEN })
        ));
        assert_eq!(expression.len(), EXP_LEN);
        assert_eq!(expression.tokens().last(), Some(Token::Number));
        assert_eq!(expression.atoms().unwrap().last(), Some(&Atom::Number(1)));
        assert_eq!(
            expression.entries().last(),
            Some((Token::Number, Atom::Number(1)))
        );
    }
}
