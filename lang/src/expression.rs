use crate::{Atom, Atoms, Function};

// A provisional balance for short and nested Expressions; revisit with usage data.
const INLINE_RECORD_CAPACITY: usize = 8;

const DEFAULT_TOKEN_LEN: usize = 2;
const DEFAULT_CHAR_TOKEN_LEN: usize = 1;

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone)]
pub struct Expression {
    records: RecordStore,
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

/// What stands at one entry of an Expression, in the two places a `Token` is
/// read.
///
/// The Parser labels every entry it produces with one, so a `Token` is what the
/// Source shows at a Position. `define_functions!` also mints one per declared
/// operand through `operand_token!`, so a `Token` is equally what a signature
/// requires at that position. The two readings coincide for the literal
/// operands — a Number position holds two hexadecimal Cells and an entry
/// holding them is labelled `Number` — and they come apart at both ends.
/// `Activation`, `Bang`, `Comment`, and `Function` are labels the Parser
/// applies to Cells no signature declares, and `Atom` and `Sequence` below are
/// declarations no Cells spell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Activation,
    Bang,
    /// The rest of the Source, claimed by the `||` introducer and never
    /// decoded — a Grid row, wherever one supplied the Source. A Comment
    /// records this and no Atom (ADR 0035), which is what lets it carry text:
    /// an Atom is a fixed-width value and cannot hold a row.
    Comment,
    Function,
    Note,
    Number,
    Char,
    /// An operand a Function declares over every Atom rather than over one
    /// type: the replacement of ADR 0007's Replace, which "may have a
    /// different Atom type" from the member it displaces.
    Atom,
    /// An operand a Function consumes as one whole Sequence rather than
    /// extending across element by element: the Sequence operand of ADR 0007's
    /// Reverse, Select, and Replace.
    Sequence,
}

impl Expression {
    pub fn new() -> Self {
        Self {
            records: RecordStore::new(),
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
    ///
    /// Exhaustive rather than closed with a wildcard, so a `Token` added later
    /// states whether it has a literal reading here instead of inheriting a
    /// refusal by default.
    pub fn decode(self, spelling: &str) -> Result<Atom, crate::Error> {
        match self {
            Self::Number => crate::to_atom_num(spelling),
            Self::Note => crate::to_atom_note(spelling),
            Self::Char => crate::atom::to_atom_char(spelling),
            // A generic Atom operand has no literal reading, and the refusal is
            // the decision rather than a gap left for later. CONTEXT.md defines
            // an Operand Literal as two Cells "interpreted as an Atom according
            // to the typed operand position of the Function that consumes
            // them", and a position declared over every Atom supplies no type
            // for them to be interpreted against: `07` would have to read as a
            // Number here and as something else there with nothing in the
            // declaration to say which. Choosing one would mint exactly the
            // intrinsic type that entry denies. No width would serve the choice
            // in any case — `Atom::Char` spells in one Cell and every other Atom
            // in two, so `len` below could answer for no decode ranging over all
            // of them, and the Parser's `atom.to_string().len() == token.len()`
            // property would be the first thing to break.
            //
            // No Function declares this operand today, so the refusal also has
            // no caller. Whether Replace's replacement earns a spelling rule of
            // its own is issue 03's decision, made against its own tests; until
            // then the only thing that can stand at the position is a nested
            // Function's typed answer, exactly as for a Sequence below.
            Self::Atom => Err(crate::SyntaxError::ExpectedToken.into()),
            // A Sequence has no literal spelling at all, and this refusal is
            // settled rather than deferred. ADR 0007 encodes a Sequence result
            // into Source as ordinary Atoms "without a privileged
            // literal-Sequence interpretation", so nothing reads Cells back as
            // one: a Sequence "exists only between Functions", and a Sequence
            // operand can only ever be a nested Function's answer. A reading
            // invented here would be the privileged interpretation that ADR
            // rules out.
            Self::Sequence => Err(crate::SyntaxError::ExpectedToken.into()),
            // The Parser fills these three positions structurally rather than by
            // decoding a literal against a signature: it reads two Cells,
            // recognises `**`, an Activation spelling, or a Function spelling,
            // and labels the entry with what it found. Nothing asks them to
            // decode, and the refusal they have always answered with is
            // unchanged.
            Self::Activation | Self::Bang | Self::Function => {
                Err(crate::SyntaxError::ExpectedToken.into())
            }
            // A Comment records no Atom at all (ADR 0035): its claim is the
            // rest of the Source, a Grid row rather than a fixed-width value,
            // and nothing asks it to decode one.
            Self::Comment => Err(crate::SyntaxError::ExpectedToken.into()),
        }
    }

    /// The Cells this Token's spelling occupies where a slot declares it.
    ///
    /// This is an operand width: `take_token` reads exactly this many Cells
    /// for the operand a signature names, and every rendered Atom spells
    /// exactly this many back. `Token::Comment` names no operand — no
    /// signature declares one and a Comment never binds — so what it answers
    /// here is the width of its `||` introducer rather than the extent of its
    /// claim, which is the rest of the Source and is not a Token width at all.
    ///
    /// Exhaustive for the same reason [`Token::decode`] is: a width is a
    /// decision each `Token` makes, not one it inherits.
    pub fn len(&self) -> usize {
        match self {
            Token::Char => DEFAULT_CHAR_TOKEN_LEN,
            // Two Cells is what an operand position occupies whatever fills it.
            // Every Atom spelling but `Atom::Char` is two Cells wide, and a
            // nested Function — the only other thing that can stand at an
            // operand position — is exactly two by the compile-time assertion
            // `define_functions!` holds every spelling to. That settles both new
            // declarations. `Atom` and `Sequence` refuse their literal decode
            // above, so this width fixes only how far a refused operand advances
            // and how wide the Span its diagnostic covers is, and two Cells is
            // the operand width every existing signature already reserves.
            //
            // Zero is the tempting reading for `Sequence` — a value that is
            // never spelled occupies no Source — and it is wrong twice. The
            // Cells are occupied, by the nested Function that is the only thing
            // able to fill the position; and `Token::is_empty` is `len() == 0`,
            // so a zero-width operand would claim an operand position that holds
            // nothing and hand the Parser a slot it advances no Cells past.
            Token::Activation
            | Token::Bang
            | Token::Comment
            | Token::Function
            | Token::Note
            | Token::Number
            | Token::Atom
            | Token::Sequence => DEFAULT_TOKEN_LEN,
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

/// Append-only storage: overflow follows the full inline prefix in entry order.
/// Eight is an allocation optimization, never a limit on accepted Expressions.
#[derive(Debug, Clone)]
struct RecordStore {
    inline: arrayvec::ArrayVec<Record, INLINE_RECORD_CAPACITY>,
    overflow: Vec<Record>,
}

impl RecordStore {
    fn new() -> Self {
        Self {
            inline: arrayvec::ArrayVec::new(),
            overflow: Vec::new(),
        }
    }

    fn push(&mut self, record: Record) {
        if self.inline.is_full() {
            self.overflow.push(record);
        } else {
            self.inline.push(record);
        }
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Record> {
        self.inline.iter().chain(self.overflow.iter())
    }

    fn len(&self) -> usize {
        self.inline.len() + self.overflow.len()
    }

    fn is_empty(&self) -> bool {
        self.inline.is_empty() && self.overflow.is_empty()
    }
}

impl IntoIterator for RecordStore {
    type Item = Record;
    type IntoIter = std::iter::Chain<
        arrayvec::IntoIter<Record, INLINE_RECORD_CAPACITY>,
        std::vec::IntoIter<Record>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.inline.into_iter().chain(self.overflow)
    }
}
