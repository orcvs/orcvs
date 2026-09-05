use crate::{
    ArgumentError, Atom, Error, Function, InterpretationError, Note, Sequence, SequenceError,
    Token, TypeError, Value,
};
use arrayvec::ArrayVec;
use std::ops::Deref;

pub struct MaybeAtom(pub Option<Atom>);

pub(crate) enum NumericValue {
    Note(Note),
    Number(u8),
}

/// The operands one Function declares, named by the role each position plays.
///
/// `define_functions!` generates one implementation per Function from the same
/// table that declares its spelling, kind, pervasion, and operand types, so a
/// role, its position, and its type are declared together and once. A Function
/// body destructures the struct instead of indexing the operands it was handed,
/// which is what leaves the declaration as the only place an operand order
/// exists.
pub(crate) trait Operands: Sized {
    /// The Function whose signature these operands are extracted against.
    const FUNCTION: Function;

    /// Binds each declared role to its operand, in signature order.
    ///
    /// Only this module can produce the [`Extracted`] this takes, and it
    /// produces one only after checking every Atom of every element against
    /// `FUNCTION`'s signature. That is what keeps a mistyped bind unreachable
    /// rather than merely uncalled.
    ///
    /// It is fallible because a declared operand type may be narrower than the
    /// `Token` the signature checks: a MIDI channel is read as a Number and is
    /// a channel only once its domain conversion succeeds. Every arity, shape,
    /// and type diagnostic is already raised by the time this runs, so a domain
    /// diagnostic can never displace one.
    fn from_operands(operands: Extracted<'_>) -> Result<Self, Error>;
}

/// The operands of a Function that declares exactly one of them.
///
/// `define_functions!` implements this for a Function's operand struct only
/// where its declaration lists a single role, so the arity comes from the same
/// table row the roles and the types do. It exists because an evaluation seam
/// that reads one operand per element — the numeric conversions, whose type
/// layer ADR 0021 replaces rather than removes — would otherwise take the
/// Function it is for as an argument unrelated to the closure it is handed, and
/// a two-operand Function passed there would have to be caught at run time
/// inside a Tick. As a bound it is caught where it is written.
pub(crate) trait UnaryOperands: Operands {}

/// One element's operands, checked against a Function's signature.
///
/// The field is private to this module, so holding one is proof of having been
/// handed it by a checked broadcast. Nothing else in the crate can present a
/// short or mistyped slice to [`Operands::from_operands`].
pub(crate) struct Extracted<'a> {
    atoms: &'a [Atom],
}

impl Extracted<'_> {
    /// The checked operands, in signature order.
    #[inline(always)]
    pub(crate) fn atoms(&self) -> &[Atom] {
        self.atoms
    }
}

/// The one shape a whole operation runs at.
///
/// Decided once for every operand together rather than once per operand,
/// because ADR 0007's rule is a rule about the operation: a scalar repeats
/// across every element, two Sequences pair element-wise, and lengths that
/// cannot pair diagnose. A per-operand decision would have nowhere to notice
/// that two Sequence operands disagree, and would answer about the second one
/// as though the first had not been read.
#[derive(Clone, Copy)]
enum Shape {
    /// Every operand was one Atom, so the Function evaluates once and answers
    /// the ordinary Atom it answered before broadcasting existed.
    Scalar,
    /// At least one operand was a Sequence, and every Sequence operand has
    /// exactly this length. Zero is a width like any other: an empty Sequence
    /// operand makes an operation of no elements whose answer is the empty
    /// Sequence, rather than a shape to refuse.
    Sequence(usize),
}

/// The widest operand list any Function declares, and the capacity of every
/// per-operation buffer below.
///
/// Read off the Function table rather than written down beside it, so a
/// Function that declared a fifth operand would widen these buffers by being
/// declared rather than by someone remembering to. It is a bound of its own and
/// not `EXP_LEN` because the two count different things: `EXP_LEN` bounds the
/// Atoms one Expression may hold, while what bounds an operand list is the
/// signature the Function declares. Sizing an operand buffer at `EXP_LEN`
/// spends 32 `Value` slots — the better part of a kilobyte moved out of
/// [`Stack::broadcast`] on every operation — where the widest signature in the
/// table reads four.
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

/// One operation's popped operands and the single shape they decided.
///
/// This is deliberately not two mechanisms. The table-driven Functions and the
/// numeric conversions differ only in the type layer above this — a signature
/// check for the first, ADR 0021's `NumericValue` for the second — and share
/// the pop, the shape, the per-element operands, and the assembly.
///
/// It is not generic in the Operand Stack's capacity. What bounds an operand
/// list is the signature its Function declares, not how many values the stack
/// it was drained from can hold, and taking the stack's bound here would size
/// every operation's buffer for an operand list no Function can ask for.
struct Broadcast {
    operands: ArrayVec<Value, MAX_OPERANDS>,
    shape: Shape,
}

impl Broadcast {
    /// How many elements the operation evaluates.
    #[inline(always)]
    fn width(&self) -> usize {
        match self.shape {
            Shape::Scalar => 1,
            Shape::Sequence(width) => width,
        }
    }

    /// Whether every operand was one Atom, so the operation is the single
    /// element [`Shape::Scalar`] names.
    ///
    /// Both evaluation seams ask before they reserve anything. A scalar
    /// operation is not a second mechanism beside the widened one: it is the
    /// same pop, the same check, and the same bind, with only the Sequence
    /// assembly left out, because at width one there is no Sequence to assemble
    /// and no buffer to fill on the way to an answer that is one Atom. That
    /// path is the one every Expression a Source writes today takes, so what it
    /// leaves out is worth leaving out.
    #[inline(always)]
    fn is_scalar(&self) -> bool {
        matches!(self.shape, Shape::Scalar)
    }

    /// The operands for one element, in signature order.
    ///
    /// An Atom operand answers itself at every index, which is the repetition
    /// ADR 0007 describes; a Sequence operand answers its member at that index.
    /// The index is in bounds by construction: [`Stack::broadcast`] admits a
    /// Sequence operand only where its length is the width, and every caller
    /// walks `0..width`.
    #[inline(always)]
    fn element(&self, index: usize) -> ArrayVec<Atom, MAX_OPERANDS> {
        self.operands
            .iter()
            .map(|operand| match operand {
                Value::Atom(atom) => *atom,
                Value::Sequence(sequence) => sequence.atoms()[index],
            })
            .collect()
    }

    /// The first operand that widened the operation, in signature order.
    ///
    /// `None` is exactly the scalar shape, so a caller that binds one element
    /// can refuse a widened one without an impossible branch to describe.
    #[inline(always)]
    fn first_sequence(&self) -> Option<&Sequence> {
        self.operands.iter().find_map(|operand| match operand {
            Value::Sequence(sequence) => Some(sequence),
            Value::Atom(_) => None,
        })
    }

    /// Binds one element's operands to the roles `O` declares.
    #[inline(always)]
    fn bind<O: Operands>(&self, index: usize) -> Result<O, Error> {
        O::from_operands(Extracted {
            atoms: &self.element(index),
        })
    }

    /// Assembles a widened operation's element answers into its one Sequence.
    ///
    /// Nothing reaches here until every element has answered, which is what
    /// makes a fault at any element diagnose the complete operation instead of
    /// leaving a partial Sequence behind. The answer is built through
    /// [`Sequence::new`], so a broadcast result inherits the one membership
    /// rule every other Sequence is constructed under.
    ///
    /// Only a widened operation is assembled. A scalar operation answers the
    /// Atom its Function returned, and answering it as a singleton Sequence
    /// instead would both change what the Interpreter hands tick planning and
    /// hold that Atom to a membership rule an ordinary scalar answer has never
    /// been held to.
    #[inline(always)]
    fn assemble(results: Vec<Atom>) -> Result<Value, Error> {
        Ok(Sequence::new(results)?.into())
    }
}

/// The operand stack one Expression evaluates against.
///
/// It holds a [`Value`] rather than an `Atom` so a Sequence produced by one
/// Function can be consumed by another without becoming Source writes
/// prematurely. Whether a Sequence arriving at an operand position widens the
/// operation or is refused is not decided here: every Function declares its
/// pervasion in `define_functions!`, and the broadcast seam below asks.
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

    /// Pushes one value, diagnosing a stack with no slot left.
    ///
    /// The caller that sizes the stack is responsible for the bound — `Args`
    /// states why `EXP_LEN` slots suffice for every Expression the parser
    /// accepts — so this answer is unreachable from Source today. It is an
    /// answer rather than a panic because the Evaluator runs inside a Tick,
    /// under the Source write guard, and a panic there stops Playback on the
    /// native target and takes the editor with it on `wasm32`. A diagnostic
    /// costs one `?` and leaves the failure describable.
    #[inline(always)]
    pub fn push(&mut self, value: impl Into<Value>) -> Result<(), Error> {
        self.inner
            .try_push(value.into())
            .map_err(|_| InterpretationError::OperandStackExhausted { capacity: N }.into())
    }

    /// Pops one slot as the scalar Atom a caller outside Function evaluation
    /// asks for, answering the absence marker for an empty stack.
    ///
    /// Function evaluation does not come through here: it pops whole [`Value`]s
    /// at the broadcast seam, where the popping Function's declared pervasion
    /// decides whether a Sequence widens the operation. This raises the same
    /// `ExpectedAtom` diagnostic a Scalar Function's operand raises there,
    /// because the rule is the same one — a Sequence has no scalar reading —
    /// and answering with its first Atom would silently discard the rest.
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

    /// Pops the operands `function` declares and decides the one shape the
    /// whole operation runs at.
    ///
    /// Arity and shape are the only things settled here, and that is the whole
    /// ordering discipline in one place: a diagnostic about the operand list as
    /// a whole precedes every diagnostic about one of its elements. An operand
    /// that has not been popped yet cannot contribute to a shape, so a missing
    /// operand is reported before the shape of the operands that are present.
    ///
    /// A Sequence widens the operation only where the Function declares that it
    /// pervades. Every other Function refuses one wherever it stands, so the
    /// scalar exceptions ADR 0012 names are refused by their declaration rather
    /// than by an omission somewhere in a body.
    fn broadcast(&mut self, function: Function) -> Result<Broadcast, Error> {
        let signature = function.signature();
        // One Value per operand the signature declares, and `MAX_OPERANDS` is
        // the widest signature the table holds, so the push below is total: no
        // Function can declare an operand list this buffer cannot take. The
        // capacity is derived from the same declarations the loop reads, which
        // is what keeps the two from drifting apart.
        let mut operands: ArrayVec<Value, MAX_OPERANDS> = ArrayVec::new();

        for found in 0..signature.len() {
            operands.push(self.inner.pop().ok_or(ArgumentError::Arity {
                expected: signature.len(),
                found,
            })?);
        }

        let mut shape = Shape::Scalar;

        for operand in &operands {
            let Value::Sequence(sequence) = operand else {
                continue;
            };

            if !function.is_pervasive() {
                return Err(SequenceError::ExpectedAtom(sequence.to_string()).into());
            }

            match shape {
                Shape::Scalar => shape = Shape::Sequence(sequence.len()),
                Shape::Sequence(width) if width != sequence.len() => {
                    return Err(SequenceError::IncompatibleLengths {
                        left: width,
                        right: sequence.len(),
                    }
                    .into());
                }
                Shape::Sequence(_) => {}
            }
        }

        Ok(Broadcast { operands, shape })
    }

    /// Pops one operation's operands and checks every Atom of every operand
    /// against the signature `O` declares.
    ///
    /// Every Atom, before any of them binds a role: an element-3 type fault
    /// would otherwise be displaced by an element-0 domain fault, which would
    /// make the diagnostic a Source is shown depend on the order the elements
    /// happen to be walked in.
    ///
    /// The walk is over operands rather than over elements, and that is not an
    /// implementation preference. A scalar operand belongs to the operation's
    /// type even where the shape makes no elements out of it: at width zero
    /// there is no element for a scalar to be repeated into, so an element walk
    /// would let a Note stand in a Number position beside an empty Sequence and
    /// answer as though the operand had been read. Walking operands also fixes
    /// which of two faulty operands answers — the earlier one in signature
    /// order, and within it the earlier member — which is the only ordering a
    /// reader can follow, because the diagnostic carries the offending Atom and
    /// not its index.
    #[inline(always)]
    fn checked<O: Operands>(&mut self) -> Result<Broadcast, Error> {
        let broadcast = self.broadcast(O::FUNCTION)?;
        let signature = O::FUNCTION.signature().iter().copied();

        for (expected, operand) in signature.zip(&broadcast.operands) {
            match operand {
                Value::Atom(atom) => check_token(expected, *atom)?,
                Value::Sequence(sequence) => {
                    for atom in sequence {
                        check_token(expected, *atom)?;
                    }
                }
            }
        }

        Ok(broadcast)
    }

    /// Pops and validates the operands `O` declares, in signature order.
    ///
    /// The scalar path, for the Functions that declare they do not pervade:
    /// their operands are refused as Sequences in `broadcast`, so the one
    /// element this binds is already the whole operation and the answer below
    /// is unreachable through them. It is an answer rather than an assertion
    /// because binding element 0 of a widened operation would read one member
    /// of a Sequence and discard the rest, silently, inside a Tick — the exact
    /// failure `ExpectedAtom` exists to prevent, and not an invariant the types
    /// prove.
    #[inline(always)]
    pub(crate) fn extract<O: Operands>(&mut self) -> Result<O, Error> {
        let broadcast = self.checked::<O>()?;

        if let Some(sequence) = broadcast.first_sequence() {
            return Err(SequenceError::ExpectedAtom(sequence.to_string()).into());
        }

        broadcast.bind(0)
    }

    /// Evaluates one pervasive Function across the shape its operands decide.
    ///
    /// `element` states what the Function is for one element and nothing about
    /// Sequences, which is the point of putting the broadcast here: `math::add`
    /// says that addition wraps and says it once, whether it is answering about
    /// one pair of Numbers or two hundred.
    ///
    /// A scalar operation is answered where it is bound. It runs the same
    /// validation, the same bind, and the same closure the widened path runs —
    /// the ordering `checked` fixes is unchanged, because the check is over
    /// operands and happens before either path begins — and then answers the
    /// Atom the Function returned, without reserving a Sequence's worth of room
    /// for a single element on the way.
    #[inline(always)]
    pub(crate) fn apply<O, F>(&mut self, element: F) -> Result<Value, Error>
    where
        O: Operands,
        F: Fn(O) -> Result<Atom, Error>,
    {
        let broadcast = self.checked::<O>()?;

        if broadcast.is_scalar() {
            return Ok(element(broadcast.bind(0)?)?.into());
        }

        let mut results = Vec::with_capacity(broadcast.width());

        for index in 0..broadcast.width() {
            results.push(element(broadcast.bind(index)?)?);
        }

        Broadcast::assemble(results)
    }

    /// Evaluates one pervasive whole-value predicate across the shape its
    /// operands decide.
    ///
    /// ADR 0011 has Equality use ordinary broadcasting to find its comparison
    /// pairs and then answer one scalar about all of them, so it shares
    /// everything above with [`Stack::apply`] and differs only in what it does
    /// with the answers. It cannot be written as a map: a map would have to put
    /// something at a position where a pair was unequal, and the only Atom
    /// meaning nothing is the absence marker, which `Sequence::new` refuses
    /// precisely because it has no Source encoding. An operation of no pairs is
    /// vacuously true, which is what makes an empty Sequence operand answer one
    /// Bang rather than nothing.
    #[inline(always)]
    pub(crate) fn predicate<O, F>(&mut self, pair: F) -> Result<Value, Error>
    where
        O: Operands,
        F: Fn(O) -> bool,
    {
        let broadcast = self.checked::<O>()?;
        let mut all = true;

        // Every element is still bound once the answer is settled, because a
        // bind is where a declared domain is checked: stopping at the first
        // unequal pair would make whether a later element diagnoses depend on
        // which earlier pair happened to disagree.
        for index in 0..broadcast.width() {
            all &= pair(broadcast.bind(index)?);
        }

        Ok(if all { Atom::Bang } else { Atom::Empty }.into())
    }

    /// Evaluates one numeric conversion across the shape its operand decides.
    ///
    /// ADR 0021 excludes `.v` and `.^` from the signature check rather than
    /// from broadcasting. Their evaluation accepts an already-typed value of
    /// their own result type as an identity, so composition and broadcasting
    /// compose, and their operand is therefore read as a [`NumericValue`]
    /// instead of against the single `Token` their literal signature declares.
    /// That is one type layer replaced; the pop, the shape, the ordering, and
    /// the all-or-nothing assembly are the same ones every other pervasive
    /// Function runs on — including the scalar shape, which is answered as the
    /// one Atom it is rather than through the widened path's two buffers.
    #[inline(always)]
    pub(crate) fn convert<O, F>(&mut self, element: F) -> Result<Value, Error>
    where
        O: UnaryOperands,
        F: Fn(NumericValue) -> Result<Atom, Error>,
    {
        let broadcast = self.broadcast(O::FUNCTION)?;

        if broadcast.is_scalar() {
            // One declared operand at the scalar shape is one Atom, so reading
            // its type and converting it is the complete operation: the "every
            // element before any element" ordering below is satisfied here by
            // there being no second element to order against. The default is
            // unreachable — `UnaryOperands` declares the operand and
            // `broadcast` refuses to answer without it — and it is a default
            // rather than a panic because this runs inside a Tick under the
            // Source write guard, where ADR 0028 rules the panic out. The
            // absence marker is not numeric, so an impossible state costs a
            // type diagnostic rather than Playback.
            let atom = broadcast
                .element(0)
                .into_iter()
                .next()
                .unwrap_or(Atom::Empty);

            return Ok(element(NumericValue::try_from(atom)?)?.into());
        }

        // Every element's type before any element converts, for the reason
        // `checked` gives: a Sequence whose last member is not numeric must
        // diagnose as that rather than as whatever its first member fails to
        // convert to. There is no scalar operand to miss at width zero the way
        // a two-operand Function has one, because the single operand is what
        // the width was read from.
        let mut values = Vec::with_capacity(broadcast.width());
        for index in 0..broadcast.width() {
            // One Atom per element, because `UnaryOperands` is what the bound
            // above asks for, so this yields exactly `width` values.
            for atom in broadcast.element(index) {
                values.push(NumericValue::try_from(atom)?);
            }
        }

        let mut results = Vec::with_capacity(values.len());
        for value in values {
            results.push(element(value)?);
        }

        Broadcast::assemble(results)
    }
}

impl<const N: usize> Default for Stack<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks one operand Atom against the `Token` its declaration names.
///
/// The one place the signature is read, so the scalar path and every element
/// of a broadcast are held to the same rule.
#[inline(always)]
fn check_token(expected: Token, atom: Atom) -> Result<(), Error> {
    match (expected, atom) {
        (Token::Number, Atom::Number(_)) | (Token::Note, Atom::Note(_)) => Ok(()),
        (Token::Number, atom) => Err(TypeError::Number(atom.into()).into()),
        (Token::Note, atom) => Err(TypeError::Note(atom.into()).into()),
        _ => unreachable!("scalar and terminal signatures contain only typed operands"),
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

impl TryFrom<Atom> for NumericValue {
    type Error = Error;

    /// ADR 0021's evaluation-time reading of a conversion's operand: either
    /// numeric Atom is accepted, and the Function decides which of the two is
    /// its identity case.
    #[inline(always)]
    fn try_from(atom: Atom) -> Result<Self, Self::Error> {
        match atom {
            Atom::Note(value) => Ok(Self::Note(value)),
            Atom::Number(value) => Ok(Self::Number(value)),
            atom => Err(TypeError::Numeric(atom.into()).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ArgumentError, Atom, EXP_LEN, Error, Function, InterpretationError, Note, Sequence,
        SequenceError, Stack, TypeError, Value,
        atom::operands,
        stack::{MAX_OPERANDS, NumericValue},
    };

    fn empty_stack() -> Stack<16> {
        Stack::new()
    }

    fn sequence() -> Sequence {
        Sequence::new([Atom::Number(0), Atom::Number(1)]).unwrap()
    }

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    fn numbers(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(Atom::Number)).unwrap()
    }

    /// Subtraction, per element, as `math::subtract` states it.
    ///
    /// The broadcast tests below use an operation whose operands are not
    /// interchangeable, so a repeat or a pairing that lands on the wrong side
    /// changes the answer rather than only the shape.
    fn difference(stack: &mut Stack<16>) -> Result<Value, Error> {
        stack.apply(|operands::Subtract { left, right }: operands::Subtract| {
            Ok(Atom::Number(left.wrapping_sub(right)))
        })
    }

    /// Division, per element, as `math::divide` states it: the one arithmetic
    /// Function with an operand pair that has no answer, which is what makes
    /// an evaluation fault at a chosen element observable.
    fn quotient(stack: &mut Stack<16>) -> Result<Value, Error> {
        stack.apply(
            |operands::Divide { left, right }: operands::Divide| match right {
                0 => Err(InterpretationError::DivisionByZero.into()),
                right => Ok(Atom::Number(left / right)),
            },
        )
    }

    /// Equality, per pair, as `math::equality` states it: the one Function that
    /// answers once about every pair rather than once per pair.
    fn all_equal(stack: &mut Stack<16>) -> Result<Value, Error> {
        stack.predicate(|operands::Equality { left, right }: operands::Equality| left == right)
    }

    /// `.^`, per element, as `numeric_conversion::to_note` states it.
    fn to_note(stack: &mut Stack<16>) -> Result<Value, Error> {
        stack.convert::<operands::ConvertToNote, _>(|value| match value {
            NumericValue::Note(value) => Ok(Atom::Note(value)),
            NumericValue::Number(value) => Ok(Atom::Note(Note::try_from(value)?)),
        })
    }

    /// Pushes `operands` so extraction pops them in signature order.
    fn push_all(stack: &mut Stack<16>, operands: impl IntoIterator<Item = Value>) {
        let operands: Vec<Value> = operands.into_iter().collect();
        for operand in operands.into_iter().rev() {
            stack.push(operand).unwrap();
        }
    }

    #[test]
    fn no_function_declares_more_operands_than_one_broadcast_buffer_holds() {
        // A broadcast sizes its buffers to the widest signature rather than to
        // `EXP_LEN`, and `ArrayVec::push` panics on overflow — inside a Tick,
        // under the Source write guard ADR 0028 rules that out. The capacity is
        // derived from the same table the signatures come from, so this reads
        // that table a second way rather than restating a number: a Function
        // that declared a fifth operand would have to widen the buffer by being
        // declared, and this fails if it ever stops doing so.
        let widest = Function::ALL
            .iter()
            .map(|function| function.signature().len())
            .max()
            .expect("the Function table declares at least one Function");

        assert_eq!(
            MAX_OPERANDS, widest,
            "a declared operand list outgrows the buffer a broadcast pops it into"
        );
        assert!(
            widest < EXP_LEN,
            "the operand buffer is the parser's Expression bound under another name"
        );
    }

    #[test]
    fn a_sequence_crosses_function_evaluation_intact() {
        // The point of the seam: what one Function pushes, the next pops
        // unchanged, without ever being encoded for the Source.
        let mut stack = empty_stack();
        stack.push(sequence()).unwrap();

        assert_eq!(stack.pop_value(), Some(Value::Sequence(sequence())));
        assert_eq!(stack.pop_value(), None);
    }

    /// Pushes a complete, otherwise well-typed operand list for `O` with a
    /// Sequence standing at each position in turn, and requires the shape
    /// diagnostic every time.
    fn assert_a_sequence_is_refused_at_every_position<O: crate::stack::Operands>(base: &[Atom]) {
        for position in 0..base.len() {
            let mut stack = empty_stack();
            push_all(
                &mut stack,
                base.iter().enumerate().map(|(index, atom)| {
                    if index == position {
                        sequence().into()
                    } else {
                        (*atom).into()
                    }
                }),
            );

            assert!(
                matches!(
                    stack.extract::<O>(),
                    Err(Error::Sequence(SequenceError::ExpectedAtom(found))) if found == "0001"
                ),
                "a Sequence was accepted at operand {position}"
            );
        }
    }

    #[test]
    fn a_sequence_diagnoses_where_a_scalar_function_requires_an_atom() {
        // The Terminal Output Functions are the Scalar rows of the Function
        // table, so `ExpectedAtom` stays reachable for exactly the Functions
        // that declare they do not broadcast — at every operand position, not
        // only the one the pop loop happens to reach first.
        assert_a_sequence_is_refused_at_every_position::<operands::RawPlay>(&[
            Atom::Number(0),
            Atom::Number(0x7F),
            note(60),
        ]);
        assert_a_sequence_is_refused_at_every_position::<operands::TimedPlay>(&[
            Atom::Number(0),
            Atom::Number(0x7F),
            note(60),
            Atom::Number(0x04),
        ]);
    }

    #[test]
    fn an_operation_over_atoms_alone_evaluates_once_and_answers_an_ordinary_atom() {
        // Broadcasting must not change what every Expression written so far
        // answers: scalar operands leave an Atom on the stack, not a singleton
        // Sequence that would encode identically and compare differently.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x20).into(), Atom::Number(0x02).into()],
        );

        assert_eq!(
            difference(&mut stack).unwrap(),
            Value::Atom(Atom::Number(0x1E))
        );
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn an_atom_operand_repeats_across_every_element_of_a_sequence_operand() {
        // Both operand positions, because a repeat that lands on the wrong
        // side of a subtraction answers a Sequence of the right length and the
        // wrong members.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [numbers([0x10, 0x20, 0x30]).into(), Atom::Number(1).into()],
        );

        assert_eq!(
            difference(&mut stack).unwrap(),
            Value::Sequence(numbers([0x0F, 0x1F, 0x2F]))
        );

        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Atom::Number(0x40).into(),
                numbers([0x10, 0x20, 0x30]).into(),
            ],
        );

        assert_eq!(
            difference(&mut stack).unwrap(),
            Value::Sequence(numbers([0x30, 0x20, 0x10]))
        );
    }

    #[test]
    fn equal_length_sequence_operands_pair_element_wise_in_order() {
        // Distinct members in both operands, so a pairing that reversed one
        // side or transposed the two answers a different Sequence rather than
        // the same one.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                numbers([0x10, 0x20, 0x30]).into(),
                numbers([0x01, 0x02, 0x03]).into(),
            ],
        );

        assert_eq!(
            difference(&mut stack).unwrap(),
            Value::Sequence(numbers([0x0F, 0x1E, 0x2D]))
        );
    }

    #[test]
    fn a_sequence_result_keeps_each_element_atom_type() {
        // `.^` answers Notes, and a broadcast that rebuilt its result out of
        // Numbers would encode differently and re-parse as something else.
        let mut stack = empty_stack();
        stack.push(numbers([0x00, 0x3C, 0x7F])).unwrap();

        assert_eq!(
            to_note(&mut stack).unwrap(),
            Value::Sequence(Sequence::new([note(0x00), note(0x3C), note(0x7F)]).unwrap())
        );
    }

    #[test]
    fn two_non_scalar_operands_of_different_lengths_diagnose_and_build_no_sequence() {
        // The lengths are named in signature order, so the diagnostic says
        // which operand the Source wrote first.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [numbers([0x10, 0x20]).into(), numbers([1, 2, 3]).into()],
        );

        assert!(matches!(
            difference(&mut stack),
            Err(Error::Sequence(SequenceError::IncompatibleLengths {
                left: 2,
                right: 3
            }))
        ));

        // Empty against non-empty is incompatible like any other unequal pair:
        // the empty Sequence is a length, not a scalar that repeats.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Sequence::empty().into(), numbers([1, 2]).into()],
        );

        assert!(matches!(
            difference(&mut stack),
            Err(Error::Sequence(SequenceError::IncompatibleLengths {
                left: 0,
                right: 2
            }))
        ));
    }

    #[test]
    fn an_empty_sequence_operand_answers_the_empty_sequence() {
        // Width zero is a legitimate operation of no elements rather than a
        // shape to refuse, and the Function body never runs.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Sequence::empty().into(), Atom::Number(1).into()],
        );

        assert_eq!(
            difference(&mut stack).unwrap(),
            Value::Sequence(Sequence::empty())
        );

        // Including where the element that never runs would have diagnosed.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(1).into(), Sequence::empty().into()],
        );

        assert_eq!(
            quotient(&mut stack).unwrap(),
            Value::Sequence(Sequence::empty())
        );
    }

    #[test]
    fn a_mistyped_scalar_operand_diagnoses_where_the_shape_makes_no_elements() {
        // Width zero evaluates nothing, and a scalar operand is still part of
        // the operation's type: ADR 0011 has `.=` accept only Number operands,
        // and neither it nor an arithmetic Function may answer for a Note or a
        // Bang standing beside an empty Sequence. A check that walked elements
        // rather than operands cannot see this, because there is no element for
        // the scalar to be repeated into.
        for faulty in [note(60), Atom::Bang] {
            let rendering = faulty.to_string();

            for operands in [
                [Value::from(faulty), Sequence::empty().into()],
                [Sequence::empty().into(), faulty.into()],
            ] {
                let mut stack = empty_stack();
                push_all(&mut stack, operands.clone());

                assert!(
                    matches!(
                        difference(&mut stack),
                        Err(Error::Type(TypeError::Number(found))) if found == rendering
                    ),
                    "{operands:?} answered for an arithmetic Function"
                );

                let mut stack = empty_stack();
                push_all(&mut stack, operands.clone());

                assert!(
                    matches!(
                        all_equal(&mut stack),
                        Err(Error::Type(TypeError::Number(found))) if found == rendering
                    ),
                    "{operands:?} answered for a predicate"
                );
            }
        }
    }

    #[test]
    fn an_element_type_diagnostic_names_the_first_faulty_operand_in_signature_order() {
        // Both operands are faulty, at different indices: the left operand's
        // fault is at element 2 and the right operand's at element 0. The
        // diagnostic carries the offending Atom and not its index, so the only
        // ordering a reader can follow is the one the Source wrote — operands in
        // signature order, and members in order within an operand.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Sequence::new([Atom::Number(0), Atom::Number(0), note(60)])
                    .unwrap()
                    .into(),
                Sequence::new([Atom::Bang, Atom::Number(0), Atom::Number(0)])
                    .unwrap()
                    .into(),
            ],
        );

        assert!(matches!(
            difference(&mut stack),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
    }

    #[test]
    fn extract_diagnoses_a_widened_shape_rather_than_binding_its_first_element() {
        // `extract` binds one element, so a Sequence operand that widened the
        // operation would leave its remaining members unread. A Scalar Function
        // never reaches this — `broadcast` refuses its Sequence first — and it
        // answers rather than truncating because a silent truncation inside a
        // Tick is the exact failure `ExpectedAtom` exists to prevent.
        let mut stack = empty_stack();
        push_all(&mut stack, [sequence().into(), Atom::Number(1).into()]);

        assert!(matches!(
            stack.extract::<operands::Add>(),
            Err(Error::Sequence(SequenceError::ExpectedAtom(found))) if found == "0001"
        ));
    }

    #[test]
    fn a_type_fault_at_any_element_diagnoses_the_complete_operation() {
        // The mistyped member is the last one, so an implementation that
        // checked only the first element — or that checked each element as it
        // evaluated it — would answer a partial Sequence of three Numbers
        // instead of a diagnostic.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Sequence::new([Atom::Number(0), Atom::Number(0), Atom::Number(0), note(60)])
                    .unwrap()
                    .into(),
                Atom::Number(1).into(),
            ],
        );

        assert!(matches!(
            difference(&mut stack),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
    }

    #[test]
    fn an_evaluation_fault_at_any_element_diagnoses_the_complete_operation() {
        // The divisor that has no quotient is the third of four, so the two
        // elements that already answered are discarded rather than assembled.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x10).into(), numbers([1, 1, 0, 1]).into()],
        );

        assert!(matches!(
            quotient(&mut stack),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn a_scalar_operation_type_checks_its_operands_before_it_evaluates_the_element() {
        // The scalar analogue of the element walk below: `./ C4 00` is mistyped
        // in its left operand and has no quotient in its right, so a path that
        // bound and evaluated the one element before checking the operand list
        // would answer `DivisionByZero`. Validation strictly precedes
        // evaluation at width one for the same reason it does at width four —
        // which diagnostic the Source is shown must not depend on how many
        // elements the operands happened to make.
        let mut stack = empty_stack();
        push_all(&mut stack, [note(60).into(), Atom::Number(0).into()]);

        assert!(
            matches!(
                quotient(&mut stack),
                Err(Error::Type(TypeError::Number(found))) if found == "C4"
            ),
            "a scalar type fault was displaced by an evaluation fault"
        );
    }

    #[test]
    fn an_evaluation_fault_in_a_scalar_operation_answers_the_fault_the_function_raised() {
        // Every other evaluation-fault case here is a widened one. A scalar
        // operation assembles nothing, so the Function's own diagnostic must
        // reach the Source unwrapped rather than as something about a Sequence
        // that was never built.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [Atom::Number(0x10).into(), Atom::Number(0x00).into()],
        );

        assert!(
            matches!(
                quotient(&mut stack),
                Err(Error::Interpretation(InterpretationError::DivisionByZero))
            ),
            "a scalar evaluation fault answered as something other than itself"
        );
    }

    #[test]
    fn every_element_is_type_checked_before_any_element_is_evaluated() {
        // Element 0 divides by zero and element 3 is mistyped. Only checking
        // every element of every operand before evaluating any of them lets
        // the later type fault win, which is what stops a diagnostic from
        // depending on which element the walk reached first.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Atom::Number(0x10).into(),
                Sequence::new([Atom::Number(0), Atom::Number(1), Atom::Number(1), note(60)])
                    .unwrap()
                    .into(),
            ],
        );

        assert!(
            matches!(
                quotient(&mut stack),
                Err(Error::Type(TypeError::Number(found))) if found == "C4"
            ),
            "an element type fault was displaced by an element evaluation fault"
        );
    }

    #[test]
    fn a_shape_diagnostic_precedes_every_element_type_diagnostic() {
        // Incompatible lengths and a mistyped member at once: the shape is a
        // fault of the operation, so it is reported ahead of anything about
        // one of its elements.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Sequence::new([note(60), Atom::Number(0)]).unwrap().into(),
                numbers([1, 2, 3]).into(),
            ],
        );

        assert!(
            matches!(
                difference(&mut stack),
                Err(Error::Sequence(SequenceError::IncompatibleLengths {
                    left: 2,
                    right: 3
                }))
            ),
            "a shape fault was displaced by an element type fault"
        );
    }

    #[test]
    fn an_arity_diagnostic_precedes_the_shape_decision() {
        // One operand short, and the operand present is a Sequence: the
        // missing operand is what the Source is told about. An implementation
        // that decided the shape while popping would answer about the shape of
        // an operand list it has not finished reading.
        let mut stack = empty_stack();
        stack.push(numbers([1, 2, 3])).unwrap();

        assert!(matches!(
            difference(&mut stack),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            }))
        ));

        // And for a Scalar Function, ahead of the shape diagnostic that would
        // otherwise refuse the same Sequence.
        let mut stack = empty_stack();
        stack.push(sequence()).unwrap();

        assert!(matches!(
            stack.extract::<operands::RawPlay>(),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 3,
                found: 1
            }))
        ));
    }

    #[test]
    fn a_numeric_conversion_shares_the_shape_decision_with_every_other_broadcast() {
        // ADR 0021 excludes `.v` and `.^` from the signature check, not from
        // broadcasting: they read a `NumericValue` where the table-driven
        // Functions read a declared `Token`, and everything below that — the
        // arity diagnostic, the shape, and the all-or-nothing assembly — is
        // the one seam the arithmetic Functions use. A width of zero also
        // leaves nothing unchecked here the way it would for a Function of two
        // operands: with one declared operand, the only way the width can be
        // zero is for that operand to be the empty Sequence itself, so there is
        // no scalar beside it for an unwalked element to hide.
        let mut stack = empty_stack();

        assert!(matches!(
            to_note(&mut stack),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 1,
                found: 0
            }))
        ));

        let mut stack = empty_stack();
        stack.push(Sequence::empty()).unwrap();

        assert_eq!(
            to_note(&mut stack).unwrap(),
            Value::Sequence(Sequence::empty())
        );
    }

    #[test]
    fn a_conversion_over_one_atom_evaluates_once_and_answers_an_ordinary_atom() {
        // The scalar shape of a conversion, at the seam rather than at the
        // Function: `.^ 3C` answered a Note before broadcasting existed and
        // must answer one still. A singleton Sequence would encode identically
        // and reach tick planning through the other arm.
        let mut stack = empty_stack();
        stack.push(Atom::Number(0x3C)).unwrap();

        assert_eq!(to_note(&mut stack).unwrap(), Value::Atom(note(0x3C)));
        assert_eq!(stack.pop_value(), None);
    }

    #[test]
    fn an_evaluation_fault_in_a_scalar_conversion_answers_the_fault_the_conversion_raised() {
        // `80` names no MIDI Note, and with one element there is nothing to
        // assemble, so the conversion's own diagnostic is what the Source is
        // told about.
        let mut stack = empty_stack();
        stack.push(Atom::Number(0x80)).unwrap();

        assert!(
            matches!(
                to_note(&mut stack),
                Err(Error::Interpretation(InterpretationError::NoteConversion(
                    0x80
                )))
            ),
            "a scalar conversion fault answered as something other than itself"
        );
    }

    #[test]
    fn a_numeric_conversion_type_checks_every_element_before_converting_any() {
        // Element 0 is outside the Note range and element 1 is not numeric at
        // all. The conversion's own type layer runs over every element first,
        // so the evaluation fault cannot displace the type fault.
        let mut stack = empty_stack();
        stack
            .push(Sequence::new([Atom::Number(0x80), Atom::Bang]).unwrap())
            .unwrap();

        assert!(matches!(
            to_note(&mut stack),
            Err(Error::Type(TypeError::Numeric(found))) if found == "**"
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
    fn scalar_operand_diagnostics_are_unchanged_by_the_sequence_seam() {
        let mut stack = empty_stack();
        push_all(&mut stack, [Atom::Number(1).into(), note(60).into()]);

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
        // `extract` decides arity, then shape, then every operand's type,
        // before binding any operand, so a declared domain can only ever be
        // the last thing to fail. Each case below supplies an operand that is
        // out of its domain *and* a second fault the earlier stage sees; the
        // earlier stage's diagnostic must win.

        // Too few operands, with the one supplied outside the channel domain.
        let mut stack = empty_stack();
        stack.push(Atom::Number(0xFF)).unwrap();

        assert!(
            matches!(
                stack.extract::<operands::RawPlay>(),
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
            [
                Atom::Number(0xFF).into(),
                Atom::Number(0xFF).into(),
                Atom::Number(60).into(),
            ],
        );

        assert!(
            matches!(
                stack.extract::<operands::RawPlay>(),
                Err(Error::Type(TypeError::Note(found))) if found == "3C"
            ),
            "a type fault was displaced by a domain fault"
        );

        // A Sequence where a Scalar Function's operand belongs, ahead of the
        // same two out-of-domain Numbers.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                sequence().into(),
                Atom::Number(0xFF).into(),
                note(60).into(),
            ],
        );

        assert!(
            matches!(
                stack.extract::<operands::RawPlay>(),
                Err(Error::Sequence(SequenceError::ExpectedAtom(found))) if found == "0001"
            ),
            "a shape fault was displaced by a domain fault"
        );

        // With nothing left for the earlier stages to answer, the domain fault
        // is reached, which is what makes the three cases above meaningful.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Atom::Number(0xFF).into(),
                Atom::Number(0xFF).into(),
                note(60).into(),
            ],
        );

        assert!(matches!(
            stack.extract::<operands::RawPlay>(),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0xFF
            )))
        ));
    }

    #[test]
    fn a_domain_diagnostic_names_the_first_operand_in_signature_order() {
        // Two operands out of their domains at once: the earlier role in the
        // declaration is the one the Source is told about, matching the pop
        // loop above it.
        let mut stack = empty_stack();
        push_all(
            &mut stack,
            [
                Atom::Number(0x10).into(),
                Atom::Number(0x80).into(),
                note(60).into(),
            ],
        );

        assert!(matches!(
            stack.extract::<operands::RawPlay>(),
            Err(Error::Interpretation(InterpretationError::MidiChannel(
                0x10
            )))
        ));
    }

    #[test]
    fn an_exhausted_operand_stack_diagnoses_rather_than_panicking() {
        // `Args` sizes the Operand Stack so no Expression the parser accepts
        // can reach this, and the answer exists anyway: the Evaluator runs
        // inside a Tick under the Source write guard, where a panic costs
        // Playback rather than the Expression. A two-slot stack states the
        // behaviour without depending on the size `Args` chooses.
        let mut stack: Stack<2> = Stack::new();
        stack.push(Atom::Number(0)).unwrap();
        stack.push(Atom::Number(1)).unwrap();

        assert!(matches!(
            stack.push(Atom::Number(2)),
            Err(Error::Interpretation(
                InterpretationError::OperandStackExhausted { capacity: 2 }
            ))
        ));

        // The refused value displaced nothing already on the stack.
        assert_eq!(Atom::from(stack.pop().unwrap()), Atom::Number(1));
        assert_eq!(Atom::from(stack.pop().unwrap()), Atom::Number(0));
    }

    #[test]
    fn an_empty_stack_still_pops_the_absence_marker() {
        let mut stack = empty_stack();

        assert_eq!(Atom::from(stack.pop().unwrap()), Atom::Empty);
        assert_eq!(stack.pop_value(), None);
    }
}
