//! What a Function makes of the operand slots it was given.
//!
//! The Parser reads characters and records two blank Cells in an operand slot
//! as unwritten, whatever Function owns the slot. Whether that leaves the
//! Function anything to do is the Function's own decision, made here and
//! nowhere else: the Language Map asks while the Source is typed, and the Tick
//! asks the same question of the Cells it reads at a Turn.

use crate::Function;

/// What one operand slot holds, as far as admitting its Function goes.
///
/// No value is carried: admission decides from which slots are written, so
/// the Language Map, which has no Tick inputs, gets the answer a Turn gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// Its Cells spell a value of the slot's type.
    Written,
    /// Its two Cells are blank.
    Unwritten,
    /// A nested Function stands in it, whose Return is known only at its
    /// Turn. It never decides admission.
    Nested,
    /// Its Cells are written but spell no value of the slot's type, or the
    /// row ends inside it.
    Malformed,
}

/// What a Function makes of its operand slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    /// It runs, and its operands' types and domains are checked as it does.
    Runs,
    /// It sends nothing and reports nothing.
    SendsNothing,
    /// The operand at `slot` spells no value. It is refused ahead of any
    /// unwritten operand, so a half-written pair is always diagnosed.
    Malformed { slot: usize },
    /// The operand at `slot` is unwritten, and this Function refuses that.
    Unwritten { slot: usize },
}

/// What a Function does with an unwritten operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OnUnwritten {
    /// Per ADR 0069, an unwritten operand is invalid input.
    Refuse,
    /// Per ADR 0106, the Turn sends nothing and reports nothing.
    SendNothing,
}

impl Function {
    /// What this Function makes of `slots`, its operand slots in signature
    /// order.
    pub fn admit(self, slots: impl IntoIterator<Item = Slot>) -> Admission {
        let mut unwritten = None;
        for (slot, state) in slots.into_iter().enumerate() {
            match state {
                Slot::Malformed => return Admission::Malformed { slot },
                Slot::Unwritten => {
                    unwritten.get_or_insert(slot);
                }
                Slot::Written | Slot::Nested => {}
            }
        }
        match (unwritten, self.on_unwritten()) {
            (None, _) => Admission::Runs,
            (Some(_), OnUnwritten::SendNothing) => Admission::SendsNothing,
            (Some(slot), OnUnwritten::Refuse) => Admission::Unwritten { slot },
        }
    }

    /// Each Function's own answer. No arm is a wildcard, so a new Function
    /// does not compile until it says what an unwritten operand means.
    const fn on_unwritten(self) -> OnUnwritten {
        use OnUnwritten::{Refuse, SendNothing};
        match self {
            // The MIDI outputs, as Orca's `:` returns early on an empty port.
            Self::RawPlay
            | Self::TimedPlay
            | Self::MonophonicPlay
            | Self::RawChordPlay
            | Self::TimedChordPlay
            | Self::ControlChange
            | Self::PitchBend => SendNothing,
            // UDP and OSC take a Payload, not operands.
            Self::Udp | Self::Osc => Refuse,
            Self::AbsoluteCopy
            | Self::AbsoluteDifference
            | Self::AbsoluteRead
            | Self::AbsoluteWrite
            | Self::Add
            | Self::Bang
            | Self::Clock
            | Self::ConvertToNote
            | Self::ConvertToNumber
            | Self::CopyEast
            | Self::CopyNorth
            | Self::CopySouth
            | Self::CopyWest
            | Self::Delay
            | Self::DirectionalBangEast
            | Self::DirectionalBangNorth
            | Self::DirectionalBangSouth
            | Self::DirectionalBangWest
            | Self::Divide
            | Self::Down
            | Self::Equality
            | Self::Euclidean
            | Self::Halt
            | Self::Increment
            | Self::Interpolation
            | Self::Maximum
            | Self::Minimum
            | Self::Modulo
            | Self::Multiply
            | Self::Pick
            | Self::Push
            | Self::Random
            | Self::ReadEast
            | Self::ReadNorth
            | Self::ReadSouth
            | Self::ReadWest
            | Self::SelfBangingEast
            | Self::SelfBangingNorth
            | Self::SelfBangingSouth
            | Self::SelfBangingWest
            | Self::Stack
            | Self::Subtract
            | Self::Track
            | Self::Up
            | Self::WriteEast
            | Self::WriteNorth
            | Self::WriteSouth
            | Self::WriteWest => Refuse,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Admission, Slot};
    use crate::Function;

    /// The MIDI outputs, which per ADR 0106 send nothing and report nothing
    /// when an operand is unwritten, as Orca's `:` returns early on a `.`.
    const SENDS_NOTHING: [&str; 7] = ["!>", "!~", "!%", "!:", "!#", "!c", "!b"];

    fn arity(function: Function) -> usize {
        function.signature().len()
    }

    fn sends_nothing(function: Function) -> bool {
        SENDS_NOTHING.contains(&function.cells().to_string().as_str())
    }

    fn with(function: Function, slot: usize, state: Slot) -> Vec<Slot> {
        let mut slots = vec![Slot::Written; arity(function)];
        slots[slot] = state;
        slots
    }

    #[test]
    fn every_function_runs_when_every_operand_is_written() {
        for &function in Function::ALL {
            assert_eq!(
                function.admit(vec![Slot::Written; arity(function)]),
                Admission::Runs,
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_midi_output_sends_nothing_on_any_unwritten_operand() {
        for &function in Function::ALL.iter().filter(|f| sends_nothing(**f)) {
            for slot in 0..arity(function) {
                assert_eq!(
                    function.admit(with(function, slot, Slot::Unwritten)),
                    Admission::SendsNothing,
                    "{function:?} slot {slot}"
                );
            }
        }
    }

    #[test]
    fn every_other_function_refuses_its_first_unwritten_operand() {
        for &function in Function::ALL.iter().filter(|f| !sends_nothing(**f)) {
            for slot in 0..arity(function) {
                let mut slots = with(function, slot, Slot::Unwritten);
                slots.iter_mut().skip(slot + 1).for_each(|s| *s = Slot::Unwritten);
                assert_eq!(
                    function.admit(slots),
                    Admission::Unwritten { slot },
                    "{function:?} slot {slot}"
                );
            }
        }
    }

    #[test]
    fn a_malformed_operand_is_refused_before_any_unwritten_one() {
        for &function in Function::ALL.iter().filter(|f| arity(**f) >= 2) {
            let mut slots = with(function, 1, Slot::Malformed);
            slots[0] = Slot::Unwritten;
            assert_eq!(
                function.admit(slots),
                Admission::Malformed { slot: 1 },
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_nested_operand_never_decides() {
        for &function in Function::ALL {
            for slot in 0..arity(function) {
                assert_eq!(
                    function.admit(with(function, slot, Slot::Nested)),
                    Admission::Runs,
                    "{function:?} slot {slot}"
                );
            }
        }
    }

    #[test]
    fn a_midi_output_sends_nothing_whatever_its_nested_operands_answer_later() {
        let slots = [Slot::Nested, Slot::Written, Slot::Unwritten];
        assert_eq!(Function::RawPlay.admit(slots), Admission::SendsNothing);
    }
}
