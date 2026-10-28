//! The UDP `!u` and OSC `!o` bodies: each reads its Payload, the Cells the
//! Turn hands over as working Source, and states one send of it.

use crate::{
    Error, Interpretation, InterpretationError, NetworkSend, OscArgument, OscMessage,
    interpreter::Context,
};

/// The Payload the Turn read from working Source. A Payload with no Cells,
/// `!u!`, reads as an empty one.
fn payload<'a>(ctx: &Context<'a>) -> &'a str {
    ctx.inputs.portal_source().cells().unwrap_or("")
}

/// UDP: `!u` followed by its Payload.
///
/// One byte per Cell, a space for an empty one, and no terminator. An empty
/// Payload sends nothing and reports nothing.
#[inline(always)]
pub fn udp(ctx: &mut Context) -> Interpretation {
    let payload = payload(ctx);
    if payload.is_empty() {
        return Interpretation::EmptyPayload;
    }
    Interpretation::Send(Box::new(NetworkSend::Udp(payload.as_bytes().into())))
}

/// OSC: `!o` followed by its Payload of space-separated words.
///
/// The first word is the address. A next word starting with `,` is a type
/// string of `i`, `f` and `s`, one letter per argument. Without one, each
/// argument's type is guessed from its shape, as [`guess`] states.
///
/// # Errors
///
/// An address that does not start with `/`, a type letter other than `i`,
/// `f` or `s`, a type string whose length is not the argument count, or a
/// word that does not fit its type each diagnose, naming the word, and send
/// nothing.
pub fn osc(ctx: &mut Context) -> Result<Interpretation, Error> {
    let mut words = payload(ctx).split(' ').filter(|word| !word.is_empty());
    let address = words.next().unwrap_or("");
    if !address.starts_with('/') {
        return Err(InterpretationError::OscAddress(address.to_owned()).into());
    }
    let mut words = words.peekable();
    let types = words.next_if(|word| word.starts_with(','));
    let words: Vec<&str> = words.collect();
    let arguments = match types {
        None => words.iter().map(|word| guess(word)).collect(),
        Some(types) => {
            let letters = &types[1..];
            if let Some(letter) = letters.chars().find(|letter| !"ifs".contains(*letter)) {
                return Err(InterpretationError::OscTypeLetter(letter).into());
            }
            if letters.len() != words.len() {
                return Err(InterpretationError::OscArgumentCount {
                    types: letters.len(),
                    arguments: words.len(),
                }
                .into());
            }
            letters
                .chars()
                .zip(&words)
                .map(|(letter, word)| typed(letter, word))
                .collect::<Result<_, _>>()?
        }
    };
    Ok(Interpretation::Send(Box::new(NetworkSend::Osc(
        OscMessage {
            address: address.to_owned(),
            arguments,
        },
    ))))
}

/// The argument a word with no type string sends: an int32 for one to eight
/// uppercase hexadecimal digits, a float32 for a decimal with a point, and
/// otherwise the word as a string.
fn guess(word: &str) -> OscArgument {
    int(word)
        .or_else(|| float(word))
        .unwrap_or_else(|| OscArgument::String(word.to_owned()))
}

/// The argument `word` sends under the type string letter `letter`, which is
/// one of `i`, `f` and `s`.
fn typed(letter: char, word: &str) -> Result<OscArgument, Error> {
    let argument = match letter {
        'i' => int(word),
        'f' => float(word),
        _ => Some(OscArgument::String(word.to_owned())),
    };
    argument.ok_or_else(|| {
        InterpretationError::OscArgument {
            word: word.to_owned(),
            letter,
        }
        .into()
    })
}

/// One to eight uppercase hexadecimal digits, read as the bits of an int32,
/// as every other Number in the Grid is read as hexadecimal.
fn int(word: &str) -> Option<OscArgument> {
    let digits = (1..=8).contains(&word.len())
        && word
            .bytes()
            .all(|digit| digit.is_ascii_digit() || (b'A'..=b'F').contains(&digit));
    digits
        .then(|| u32::from_str_radix(word, 16).ok())
        .flatten()
        .map(|bits| OscArgument::Int(bits as i32))
}

/// A decimal matching `-?[0-9]+\.[0-9]+`, read as a float32.
fn float(word: &str) -> Option<OscArgument> {
    let unsigned = word.strip_prefix('-').unwrap_or(word);
    let (whole, fraction) = unsigned.split_once('.')?;
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|digit| digit.is_ascii_digit());
    (digits(whole) && digits(fraction))
        .then(|| word.parse().ok())
        .flatten()
        .map(OscArgument::Float)
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Error, Function, FunctionInputs, Interpretation, InterpretationError, Interpreter,
        NetworkSend, OscArgument, OscMessage, PortalSource, Tick, TickInputs,
    };

    /// `function`'s answer for a Turn that read `payload`.
    fn send(function: Function, payload: &str) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            function,
            [],
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                PortalSource::from_cells(Some(payload)),
            ),
        )
    }

    /// The two Functions that send a Payload.
    const FUNCTIONS: [Function; 2] = [Function::Osc, Function::Udp];

    fn osc(payload: &str) -> Result<Interpretation, Error> {
        send(Function::Osc, payload)
    }

    fn message(address: &str, arguments: Vec<OscArgument>) -> Interpretation {
        Interpretation::Send(Box::new(NetworkSend::Osc(OscMessage {
            address: address.to_owned(),
            arguments,
        })))
    }

    #[test]
    fn both_functions_take_a_payload_and_no_operand() {
        for function in FUNCTIONS {
            assert!(function.takes_payload(), "{function}");
            assert!(function.takes_no_operand(), "{function}");
            assert!(!function.answers_value(), "{function}");
        }
        assert!(
            Function::ALL
                .iter()
                .filter(|function| function.takes_payload())
                .eq(FUNCTIONS.iter())
        );
    }

    #[test]
    fn udp_sends_each_cell_as_one_byte_and_an_empty_payload_rests() {
        assert_eq!(
            send(Function::Udp, "hi there ").unwrap(),
            Interpretation::Send(Box::new(NetworkSend::Udp(b"hi there ".as_slice().into())))
        );
        assert_eq!(
            send(Function::Udp, "").unwrap(),
            Interpretation::EmptyPayload
        );
    }

    #[test]
    fn osc_guesses_each_argument_from_its_shape() {
        use OscArgument::{Float, Int, String};
        let string = |word: &str| String(word.to_owned());
        assert_eq!(
            osc("/synth/freq 40 7F").unwrap(),
            message("/synth/freq", vec![Int(64), Int(127)])
        );
        assert_eq!(
            osc("/a 1B8 440 C4 FFFFFFFF 0 FFFFFFFFF").unwrap(),
            message(
                "/a",
                vec![
                    Int(440),
                    Int(1088),
                    Int(196),
                    Int(-1),
                    Int(0),
                    string("FFFFFFFFF")
                ]
            )
        );
        assert_eq!(
            osc("/amp 0.5 -0.5 440.0").unwrap(),
            message("/amp", vec![Float(0.5), Float(-0.5), Float(440.0)])
        );
        assert_eq!(
            osc("/s ab .5 5. 1e3 -5 hi").unwrap(),
            message(
                "/s",
                ["ab", ".5", "5.", "1e3", "-5", "hi"].map(string).to_vec()
            )
        );
    }

    #[test]
    fn osc_splits_words_on_any_run_of_spaces() {
        assert_eq!(
            osc("  /x   40  ").unwrap(),
            message("/x", vec![OscArgument::Int(64)])
        );
        assert_eq!(osc("/x").unwrap(), message("/x", vec![]));
    }

    #[test]
    fn a_type_string_decides_each_argument() {
        use OscArgument::{Float, Int, String};
        assert_eq!(
            osc("/synth/freq ,fs 64.0 hi").unwrap(),
            message("/synth/freq", vec![Float(64.0), String("hi".to_owned())])
        );
        assert_eq!(
            osc("/n ,is 40 40").unwrap(),
            message("/n", vec![Int(64), String("40".to_owned())])
        );
        assert_eq!(osc("/n ,").unwrap(), message("/n", vec![]));
    }

    #[test]
    fn a_malformed_osc_payload_diagnoses_and_names_the_word() {
        let refused = |payload: &str| osc(payload).unwrap_err().to_string();
        assert!(matches!(
            osc("synth 40"),
            Err(Error::Interpretation(InterpretationError::OscAddress(word))) if word == "synth"
        ));
        assert!(matches!(
            osc(""),
            Err(Error::Interpretation(InterpretationError::OscAddress(word))) if word.is_empty()
        ));
        assert!(matches!(
            osc("/a ,ib 40 x"),
            Err(Error::Interpretation(InterpretationError::OscTypeLetter(
                'b'
            )))
        ));
        assert!(matches!(
            osc("/a ,ii 40"),
            Err(Error::Interpretation(
                InterpretationError::OscArgumentCount {
                    types: 2,
                    arguments: 1
                }
            ))
        ));
        assert!(matches!(
            osc("/a ,i ab"),
            Err(Error::Interpretation(InterpretationError::OscArgument { word, letter: 'i' }))
                if word == "ab"
        ));
        assert!(matches!(
            osc("/a ,f 5"),
            Err(Error::Interpretation(InterpretationError::OscArgument { word, letter: 'f' }))
                if word == "5"
        ));
        assert_eq!(refused("/a ,i ab"), "OSC argument \"ab\" is not an int32");
        assert_eq!(refused("/a ,f 5"), "OSC argument \"5\" is not a float32");
    }
}
