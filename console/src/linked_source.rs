//!
//! The Source a console opens from its own URL, and whether it opens one.
//!
//! The web console reads its URL once, at startup. `?example=<name>` opens a
//! shipped example by its link name, and `#source=<text>` opens percent-encoded
//! Source File text. A console opened either way is Linked: it neither restores
//! a Source from storage nor stores one, so the Source a reader keeps in the
//! full console is never read over or written over. A Linked console opens with
//! Playback stopped, as every console does, so a link cannot play by itself.
//!
//! `?small` opens the small console (ADR 0085): Linked, empty, without the
//! menu bar, and open to Source File text posted to it, which its [`Inbox`]
//! holds until the next frame opens it.
//!

#[cfg(any(target_arch = "wasm32", test))]
use orcvs::source::Source;

///
/// The most bytes of `#source=` text read, counted before decoding, so an
/// oversized link is refused without decoding it.
///
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const SOURCE_TEXT_LIMIT: usize = 16 * 1024;

///
/// Where a console's starting Source comes from.
///
pub(crate) enum Opening {
    /// The revision storage holds, or an empty Source when it holds none. The
    /// console saves to storage.
    Stored,
    /// The console's URL: the Source it names, or why it could not be opened.
    /// The console neither restores from nor saves to storage.
    #[cfg(any(target_arch = "wasm32", test))]
    Linked(Result<Source, String>),
    /// The small console: Linked, on an empty Source, opening the Source File
    /// text posted to it.
    #[cfg(any(target_arch = "wasm32", test))]
    Small,
}

///
/// The Source File text posted to a small console, oldest first, until a
/// frame opens it. The page's message listener fills it, and repaints.
///
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) type Inbox = std::rc::Rc<std::cell::RefCell<Vec<String>>>;

///
/// What a console at a URL with `search` and `fragment` opens. Each may be
/// empty or carry its leading `?` or `#`, as `Location` gives them.
///
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn opening(search: &str, fragment: &str) -> Opening {
    let search = search.strip_prefix('?').unwrap_or(search);
    if search
        .split('&')
        .any(|pair| pair == "small" || pair.starts_with("small="))
    {
        return Opening::Small;
    }
    let example = parameter(search, "example");
    let text = parameter(fragment.strip_prefix('#').unwrap_or(fragment), "source");
    match (example, text) {
        (None, None) => Opening::Stored,
        (Some(_), Some(_)) => Opening::Linked(Err(
            "the link names both an example and a Source; it can open only one".to_owned(),
        )),
        (Some(name), None) => Opening::Linked(example_named(name)),
        (None, Some(text)) => Opening::Linked(source_from(text)),
    }
}

/// The value of the first `key=value` pair in `pairs`, which `&` separates.
#[cfg(any(target_arch = "wasm32", test))]
fn parameter<'a>(pairs: &'a str, key: &str) -> Option<&'a str> {
    pairs
        .split('&')
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
}

#[cfg(any(target_arch = "wasm32", test))]
fn example_named(name: &str) -> Result<Source, String> {
    crate::examples::EXAMPLES
        .iter()
        .find(|example| example.link_name() == name)
        .map(crate::examples::Example::source)
        .ok_or_else(|| format!("the link names no shipped example called {name:?}"))
}

#[cfg(any(target_arch = "wasm32", test))]
fn source_from(text: &str) -> Result<Source, String> {
    if text.len() > SOURCE_TEXT_LIMIT {
        return Err(format!(
            "the link's Source is {} bytes, over the limit of {SOURCE_TEXT_LIMIT}",
            text.len()
        ));
    }
    let bytes = percent_decoded(text)
        .ok_or_else(|| "the link's Source is not percent-encoded correctly".to_owned())?;
    orcvs::source::file::read(&bytes)
        .map_err(|refusal| format!("the link's Source is not a Source File: {refusal}"))
}

///
/// The Source that Source File text posted to a small console opens, under
/// the limit a link's text has.
///
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn source_message(text: &str) -> Result<Source, String> {
    if text.len() > SOURCE_TEXT_LIMIT {
        return Err(format!(
            "the Source sent is {} bytes, over the limit of {SOURCE_TEXT_LIMIT}",
            text.len()
        ));
    }
    orcvs::source::file::read(text.as_bytes())
        .map_err(|refusal| format!("the Source sent is not a Source File: {refusal}"))
}

///
/// `text` with each `%` and the two hex digits after it read as one byte, or
/// `None` when a `%` is not followed by two hex digits. `+` stays `+`: it is
/// a Function's glyph, and only a query string spells a space with it.
///
#[cfg(any(target_arch = "wasm32", test))]
fn percent_decoded(text: &str) -> Option<Vec<u8>> {
    let hex = |byte: Option<u8>| char::from(byte?).to_digit(16);
    let mut bytes = text.bytes();
    let mut decoded = Vec::with_capacity(text.len());
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let digits = hex(bytes.next())? * 16 + hex(bytes.next())?;
            decoded.push(u8::try_from(digits).expect("two hex digits make a byte"));
        } else {
            decoded.push(byte);
        }
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::{Opening, SOURCE_TEXT_LIMIT, opening, source_message};
    use orcvs::source::file;

    fn linked(search: &str, fragment: &str) -> Result<String, String> {
        match opening(search, fragment) {
            Opening::Stored | Opening::Small => {
                panic!("{search:?} {fragment:?} opened no linked Source")
            }
            Opening::Linked(result) => result.map(|source| file::write(&source)),
        }
    }

    /// The `#source=` fragment for `text`, every byte percent-encoded.
    fn fragment_for(text: &str) -> String {
        let encoded: String = text.bytes().map(|byte| format!("%{byte:02X}")).collect();
        format!("#source={encoded}")
    }

    #[test]
    fn a_url_naming_nothing_opens_the_stored_source() {
        for (search, fragment) in [("", ""), ("?", "#"), ("?theme=dark", "#dev")] {
            assert!(
                matches!(opening(search, fragment), Opening::Stored),
                "{search:?} {fragment:?} did not open the stored Source"
            );
        }
    }

    #[test]
    fn an_example_name_opens_that_example() {
        for example in &crate::examples::EXAMPLES {
            let search = format!("?example={}", example.link_name());
            assert_eq!(
                linked(&search, ""),
                Ok(file::write(&example.source())),
                "{search}"
            );
        }
    }

    #[test]
    fn the_example_is_found_among_other_parameters() {
        let example = &crate::examples::EXAMPLES[0];
        let search = format!("?embed&example={}&x=1", example.link_name());
        assert_eq!(linked(&search, ""), Ok(file::write(&example.source())));
    }

    #[test]
    fn an_unknown_example_name_is_refused() {
        let refusal = linked("?example=no-such-example", "").expect_err("an unknown name");
        assert!(refusal.contains("no-such-example"), "{refusal}");
    }

    #[test]
    fn source_text_opens_as_a_source_file() {
        assert_eq!(
            linked("", "#source=.%2B0102%0A%20%200C"),
            Ok(".+0102\n  0C\n".to_owned())
        );
    }

    #[test]
    fn a_plus_is_a_plus_not_a_space() {
        assert_eq!(linked("", "#source=1+2"), Ok("1+2\n".to_owned()));
    }

    #[test]
    fn malformed_percent_encoding_is_refused() {
        for fragment in ["#source=%", "#source=1%2", "#source=%zz", "#source=%%41"] {
            assert!(linked("", fragment).is_err(), "{fragment} was accepted");
        }
    }

    #[test]
    fn text_over_the_limit_is_refused_before_decoding() {
        // Full rows, every byte encoded: three bytes of fragment per Cell.
        let rows = |count: usize| fragment_for(&format!("{}\n", ".".repeat(255)).repeat(count));
        let fits = rows(21);
        let over = rows(22);
        assert!(fits.len() - "#source=".len() <= SOURCE_TEXT_LIMIT);
        assert!(over.len() - "#source=".len() > SOURCE_TEXT_LIMIT);

        assert!(linked("", &fits).is_ok());
        let refusal = linked("", &over).expect_err("a Source over the limit");
        assert!(refusal.contains("limit"), "{refusal}");
    }

    #[test]
    fn text_that_is_not_a_source_file_is_refused() {
        for text in ["\t", "\u{e9}", &".".repeat(257)] {
            assert!(
                linked("", &fragment_for(text)).is_err(),
                "{text:?} was accepted"
            );
        }
    }

    #[test]
    fn small_opens_the_small_console() {
        for search in ["?small", "?x=1&small", "?small="] {
            assert!(
                matches!(opening(search, ""), Opening::Small),
                "{search:?} did not open the small console"
            );
        }
        assert!(matches!(opening("?smaller", ""), Opening::Stored));
    }

    #[test]
    fn a_message_opens_its_text_as_a_source_file() {
        let source = source_message(".+0102\n  0C\n").expect("a Source File");
        assert_eq!(file::write(&source), ".+0102\n  0C\n");
    }

    #[test]
    fn a_message_over_the_limit_or_not_a_source_file_is_refused() {
        let over = format!("{}\n", ".".repeat(255)).repeat(SOURCE_TEXT_LIMIT / 256 + 1);
        assert!(over.len() > SOURCE_TEXT_LIMIT);
        let refusal = source_message(&over)
            .map(|source| file::write(&source))
            .expect_err("a Source over the limit");
        assert!(refusal.contains("limit"), "{refusal}");
        assert!(source_message("\t").is_err());
    }

    #[test]
    fn an_example_and_source_text_together_are_refused() {
        let example = &crate::examples::EXAMPLES[0];
        let search = format!("?example={}", example.link_name());
        assert!(linked(&search, "#source=1").is_err());
    }

    ///
    /// Every byte, alone and in lines beside the others, opens the Source that
    /// reading it as a Source File gives, so percent-encoding loses nothing.
    /// Exhaustive over the byte alphabet: the tooling contract keeps proptest
    /// out of this crate.
    ///
    #[test]
    fn every_byte_round_trips_through_the_source_file_writer() {
        let every_byte: Vec<u8> = (0..=u8::MAX).collect();
        let texts = every_byte
            .iter()
            .map(|byte| vec![*byte])
            .chain(every_byte.chunks(40).map(<[u8]>::to_vec))
            .chain([every_byte.chunks(40).collect::<Vec<_>>().join(&b'\n')]);
        for text in texts {
            let fragment = format!(
                "#source={}",
                text.iter()
                    .map(|byte| format!("%{byte:02X}"))
                    .collect::<String>()
            );
            let expected = file::read(&text).map(|source| file::write(&source));
            assert_eq!(linked("", &fragment).ok(), expected.ok(), "{text:?}");
        }
    }
}
