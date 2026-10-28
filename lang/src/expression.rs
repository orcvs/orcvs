use crate::{Atom, Atoms, Function};

// A provisional balance for short and nested Expressions; revisit with usage data.
const INLINE_RECORD_CAPACITY: usize = 8;

/// The Cells one Language Unit other than a Comment or a Payload occupies.
pub(crate) const DEFAULT_TOKEN_LEN: usize = 2;
const DEFAULT_CHAR_TOKEN_LEN: usize = 1;

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone)]
pub struct Expression {
    records: RecordStore,
    comment: Option<CommentText>,
    /// Every Payload's text, in entry order. Empty, and so unallocated, for
    /// an Expression holding no UDP or OSC Function.
    payloads: Vec<PayloadText>,
}

///
/// The Cells between a Comment's delimiters: after its `||` introducer, and
/// before the `||` that closes it or the end of the Source. Empty for `||||`.
///
/// The Parser decides it with the claim, so whether a Comment is closed is
/// one fact every reader asks the Expression for rather than reads off its
/// Cells. It is in the address space supplied to the Parser, as every
/// [`PositionedEntry`]'s Cells are.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommentText {
    start: usize,
    end: usize,
}

impl CommentText {
    /// The text the Parser decided for the Comment it is claiming, in the
    /// Parser's address space.
    pub(crate) fn new(text: std::ops::Range<usize>) -> Self {
        Self {
            start: text.start,
            end: text.end,
        }
    }

    /// The text's Cells, in the address space supplied to the Parser.
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

///
/// The Cells of a Payload after the UDP or OSC Function at entry `function`:
/// after its spelling, and before the `!` that closes it or the end of the
/// Source. Empty for `!u!`.
///
/// The Parser decides it with the claim, as it decides a [`CommentText`], so
/// whether a Payload is closed is asked of the Expression rather than read off
/// its Cells. It is in the address space supplied to the Parser.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PayloadText {
    function: usize,
    start: usize,
    end: usize,
}

impl PayloadText {
    /// The text's Cells, in the address space supplied to the Parser.
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
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
/// Source shows at a Position. `define_functions!` also reads one per declared
/// operand from its token's [`crate::operand::TokenKind::TOKEN`], so a `Token` is
/// equally what a signature requires at that position. The two readings
/// coincide for the literal
/// operands — a Number position holds two hexadecimal Cells and an entry
/// holding them is labelled `Number` — and come apart for the rest: `Comment`
/// and `Function` are labels the Parser applies to Cells no signature declares.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    /// The Cells from a `||` introducer through the next `||`, or through the
    /// end of the Source where none follows, and never decoded. A Comment
    /// records this and no Atom, which is what lets it carry text:
    /// an Atom is a fixed-width value and cannot hold a run of text.
    Comment,
    /// The Cells a UDP or OSC Function sends: from the end of its spelling
    /// through the next `!`, or through the end of the Source where none
    /// follows, and never decoded. It records no Atom, as a Comment does, and
    /// its parent is the Function that sends it.
    Payload,
    Function,
    Note,
    Number,
    /// The one slot that takes no type: a Write's `value`. It holds any
    /// two-Cell Language Unit spelling, a Bang among them, and carries it
    /// unchanged; whatever later reads those Cells decodes them by its own
    /// type.
    Untyped,
    /// One Cell of leftover content: written, and claimed by no Language
    /// Unit. The Parser never labels an entry with it; `orcvs` answers it for
    /// such a Cell so its presentation can tell written Cells from blank ones.
    Char,
}

impl Expression {
    pub fn new() -> Self {
        Self {
            records: RecordStore::new(),
            comment: None,
            payloads: Vec::new(),
        }
    }

    /// Records one entry other than a Comment.
    ///
    /// A Comment enters only through [`Self::add_comment`], which records its
    /// text with it; one recorded here would be a `Token::Comment` entry that
    /// [`Self::comment`] does not answer.
    pub(crate) fn add_positioned(
        &mut self,
        token: Token,
        atom: Option<Atom>,
        cells: std::ops::Range<usize>,
        parent: Option<usize>,
    ) {
        assert!(
            token != Token::Comment,
            "a Comment enters an Expression only with its text"
        );
        assert!(
            token != Token::Payload,
            "a Payload enters an Expression only with its text"
        );
        self.push(token, atom, cells, parent);
    }

    ///
    /// Records a Comment claiming `cells`, whose text is `text`.
    ///
    /// The Token and the text are recorded together, and this is the only way
    /// a `Token::Comment` entry enters an Expression, so an Expression holds
    /// one exactly when [`Self::comment`] answers. The Parser reaches a
    /// Comment only at the root slot, so an Expression holds at most one, and
    /// a second is refused rather than allowed to replace the first's text.
    ///
    pub(crate) fn add_comment(
        &mut self,
        cells: std::ops::Range<usize>,
        text: CommentText,
        parent: Option<usize>,
    ) {
        assert!(
            self.comment.is_none(),
            "an Expression holds at most one Comment"
        );
        self.comment = Some(text);
        self.push(Token::Comment, None, cells, parent);
    }

    ///
    /// Records the Payload of the Function at entry `function`, claiming
    /// `cells`, whose text is `text`.
    ///
    /// As [`Self::add_comment`] is for a Comment, this is the only way a
    /// `Token::Payload` entry enters an Expression, so each one has its text.
    ///
    pub(crate) fn add_payload(
        &mut self,
        cells: std::ops::Range<usize>,
        text: std::ops::Range<usize>,
        function: usize,
    ) {
        self.payloads.push(PayloadText {
            function,
            start: text.start,
            end: text.end,
        });
        self.push(Token::Payload, None, cells, Some(function));
    }

    fn push(
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

    /// The text of the Comment this Expression is, when it is one. Its range
    /// is in the address space supplied to the Parser, as every
    /// [`PositionedEntry`]'s Cells are.
    ///
    /// It answers exactly when the Expression holds a `Token::Comment` entry.
    pub fn comment(&self) -> Option<CommentText> {
        self.comment
    }

    /// The text of the Payload the Function at entry `function` sends, when
    /// it is a UDP or OSC Function.
    pub fn payload(&self, function: usize) -> Option<PayloadText> {
        self.payloads
            .iter()
            .find(|payload| payload.function == function)
            .copied()
    }

    /// The text of the Payload `entry` is, when it is a Payload entry.
    pub fn payload_of(&self, entry: &PositionedEntry) -> Option<PayloadText> {
        (entry.token == Token::Payload)
            .then(|| entry.parent.and_then(|function| self.payload(function)))
            .flatten()
    }

    pub fn positioned(&self) -> impl DoubleEndedIterator<Item = &PositionedEntry> {
        self.records.iter()
    }

    /// Complete evaluable entries, with their syntax and runtime value paired.
    pub fn entries(&self) -> impl Iterator<Item = (Token, Atom)> + '_ {
        self.records.iter().filter_map(|record| record.entry())
    }

    /// Every entry's Atom, or `None` when an entry bound none. A Payload is
    /// part of its Function's claim rather than a value, so it is skipped.
    pub fn atoms(&self) -> Option<Atoms> {
        self.records
            .iter()
            .filter(|record| record.token != Token::Payload)
            .map(Record::atom)
            .collect()
    }

    pub fn take_atoms(self) -> Option<Atoms> {
        self.records
            .into_iter()
            .filter(|record| record.token != Token::Payload)
            .map(|record| record.atom)
            .collect()
    }

    pub fn tokens(&self) -> impl DoubleEndedIterator<Item = Token> + '_ {
        self.records.iter().map(Record::token)
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
            Self::Untyped => crate::atom::untyped_atom(spelling),
            // The Parser fills this position structurally rather than by
            // decoding a literal against a signature: it reads two Cells,
            // recognises a Function spelling, and labels the entry with what
            // it found. Nothing asks it to decode.
            Self::Function => Err(crate::SyntaxError::ExpectedToken.into()),
            // A Comment records no Atom at all: its claim is a run of
            // text rather than a fixed-width value,
            // and nothing asks it to decode one.
            Self::Comment => Err(crate::SyntaxError::ExpectedToken.into()),
            // A Payload is a run of text for the same reason.
            Self::Payload => Err(crate::SyntaxError::ExpectedToken.into()),
            // Leftover content is a Cell no Language Unit claims, so it has
            // no Atom to decode to.
            Self::Char => Err(crate::SyntaxError::ExpectedToken.into()),
        }
    }

    /// The Cells this Token's spelling occupies where a slot declares it.
    ///
    /// This is an operand width: `take_token` reads exactly this many Cells
    /// for the operand a signature names, and every rendered Atom spells
    /// exactly this many back. `Token::Comment` names no operand — no
    /// signature declares one and a Comment never binds — so what it answers
    /// here is the width of its `||` introducer rather than the extent of its
    /// claim, which runs to its closer and is not a Token width at all.
    ///
    /// Exhaustive for the same reason [`Token::decode`] is: a width is a
    /// decision each `Token` makes, not one it inherits.
    pub fn len(&self) -> usize {
        match self {
            // A Payload names no operand either, and its closer is one Cell.
            Token::Char | Token::Payload => DEFAULT_CHAR_TOKEN_LEN,
            // Two Cells is what an operand position occupies whatever fills it.
            // Every Atom spelling is two Cells wide, and a nested Function —
            // the only other thing that can stand at an operand position — is
            // exactly two by the compile-time assertion `define_functions!`
            // holds every spelling to.
            Token::Comment | Token::Function | Token::Note | Token::Number | Token::Untyped => {
                DEFAULT_TOKEN_LEN
            }
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

#[cfg(test)]
mod test {
    use super::{CommentText, Expression, Token};

    #[test]
    #[should_panic(expected = "a Comment enters an Expression only with its text")]
    fn a_comment_entry_without_its_text_is_refused() {
        Expression::new().add_positioned(Token::Comment, None, 0..2, None);
    }

    #[test]
    #[should_panic(expected = "an Expression holds at most one Comment")]
    fn a_second_comment_is_refused() {
        let mut expression = Expression::new();
        expression.add_comment(0..4, CommentText::new(2..4), None);
        expression.add_comment(4..8, CommentText::new(6..8), None);
    }
}
