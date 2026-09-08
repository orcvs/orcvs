use crate::{Atom, Atoms, Function};

const DEFAULT_TOKEN_LEN: usize = 2;
const DEFAULT_CHAR_TOKEN_LEN: usize = 1;

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone)]
pub struct Expression {
    records: Vec<Record>,
}

/// One parser-owned entry. Cells use the address space supplied to the Parser;
/// an empty range records an input missing at the Source boundary. `parent`
/// identifies the directly owning Function in this Expression's entry order.
#[derive(Debug, Clone)]
pub struct PositionedEntry {
    pub cells: std::ops::Range<usize>,
    pub parent: Option<usize>,
    pub token: Token,
    pub atom: Option<Atom>,
}

type Record = PositionedEntry;

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
            records: Vec::new(),
        }
    }

    pub(crate) fn add_positioned(
        &mut self,
        token: Token,
        atom: Option<Atom>,
        cells: std::ops::Range<usize>,
        parent: Option<usize>,
    ) {
        self.records.push(PositionedEntry {
            cells,
            parent,
            token,
            atom,
        });
    }

    pub fn positioned(&self) -> impl Iterator<Item = &PositionedEntry> {
        self.records.iter()
    }

    /// Complete evaluable entries, with their syntax and runtime value paired.
    pub fn entries(&self) -> impl Iterator<Item = (Token, Atom)> + '_ {
        self.records.iter().filter_map(|record| record.entry())
    }

    pub fn atoms(&self) -> Option<Atoms> {
        self.records.iter().map(Record::atom).collect()
    }

    pub fn take_atoms(self) -> Option<Atoms> {
        self.records.into_iter().map(|record| record.atom).collect()
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

impl PositionedEntry {
    fn entry(&self) -> Option<(Token, Atom)> {
        self.atom.map(|atom| (self.token, atom))
    }

    fn atom(&self) -> Option<Atom> {
        self.atom
    }
    fn token(&self) -> Token {
        self.token
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self::new()
    }
}

impl Token {
    /// Decodes one literal encoding using the receiving operand's signature.
    pub fn decode(self, spelling: &str) -> Result<Atom, crate::Error> {
        match self {
            Self::Number => crate::to_atom_num(spelling),
            Self::Note => crate::to_atom_note(spelling),
            Self::Char => crate::atom::to_atom_char(spelling),
            _ => Err(crate::SyntaxError::ExpectedToken.into()),
        }
    }

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
        f.signature().to_vec()
    }
}
