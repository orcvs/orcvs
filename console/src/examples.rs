//! The example Source Files shipped in `examples/`, compiled into the console
//! so `Help → Examples` opens them on native and on the web alike.

use orcvs::source::{Source, file};

///
/// A shipped example: the name `Help → Examples` lists it under, and its
/// Source File text.
///
pub(crate) struct Example {
    pub(crate) name: &'static str,
    text: &'static [u8],
}

///
/// Knobs: an acid bassline on the Built-in Synth's 303-style voice, with knobs
/// sweeping the voice and its channel's effects.
///
pub(crate) const KNOBS: Example = Example {
    name: "Knobs",
    text: include_bytes!("../../examples/knobs.orcvs"),
};

/// The example the web console opens when nothing is stored.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const FIRST_VISIT: &Example = &KNOBS;

/// Every shipped example, in the order `Help → Examples` lists them.
pub(crate) const EXAMPLES: [Example; 5] = [
    Example {
        name: "Beat",
        text: include_bytes!("../../examples/beat.orcvs"),
    },
    Example {
        name: "Melody",
        text: include_bytes!("../../examples/melody.orcvs"),
    },
    Example {
        name: "Tracker",
        text: include_bytes!("../../examples/tracker.orcvs"),
    },
    Example {
        name: "Sequencer",
        text: include_bytes!("../../examples/sequencer.orcvs"),
    },
    KNOBS,
];

impl Example {
    ///
    /// The name a link gives for this example: its name in lowercase, with a
    /// hyphen for each space, so `Melody` is `melody`.
    ///
    pub(crate) fn link_name(&self) -> String {
        self.name.to_ascii_lowercase().replace(' ', "-")
    }

    ///
    /// The example's Source, read fresh each call.
    ///
    /// # Panics
    ///
    /// If the text is refused: it is compiled in, so a refusal is a defect in
    /// the shipped file, which `every_example_is_a_source_file` catches.
    ///
    pub(crate) fn source(&self) -> Source {
        file::read(self.text)
            .unwrap_or_else(|refusal| panic!("{} is not a Source File: {refusal}", self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::{EXAMPLES, FIRST_VISIT};

    #[test]
    fn a_first_visit_opens_knobs_with_nothing_diagnosed() {
        assert_eq!(FIRST_VISIT.link_name(), "knobs");
        let source = FIRST_VISIT.source();
        assert_eq!(source.language_map().diagnostics().count(), 0);
    }

    #[test]
    fn every_example_is_a_source_file() {
        for example in &EXAMPLES {
            assert!(
                orcvs::source::file::read(example.text).is_ok(),
                "{} is not a Source File",
                example.name
            );
        }
    }

    #[test]
    fn each_example_links_by_its_name() {
        let links: Vec<String> = EXAMPLES.iter().map(super::Example::link_name).collect();
        assert_eq!(links, ["beat", "melody", "tracker", "sequencer", "knobs"]);
    }

    #[test]
    fn every_example_leaves_its_first_row_empty() {
        for example in &EXAMPLES {
            let first = example.text.split(|&byte| byte == b'\n').next();
            assert!(
                first.is_some_and(|row| row.iter().all(|&byte| byte == b' ')),
                "{} starts on its first row",
                example.name
            );
        }
    }
}
