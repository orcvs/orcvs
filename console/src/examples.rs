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

/// Every shipped example, in the order `Help → Examples` lists them.
pub(crate) const EXAMPLES: [Example; 4] = [
    Example {
        name: "Cell Tracker",
        text: include_bytes!("../../examples/cell-tracker.orcvs"),
    },
    Example {
        name: "Orca Tracker",
        text: include_bytes!("../../examples/tracker.orcvs"),
    },
    Example {
        name: "Orca Sequencer",
        text: include_bytes!("../../examples/sequencer.orcvs"),
    },
    Example {
        name: "Orca Knobs",
        text: include_bytes!("../../examples/knobs.orcvs"),
    },
];

impl Example {
    ///
    /// The name a link gives for this example: its name in lowercase, with a
    /// hyphen for each space, so `Cell Tracker` is `cell-tracker`.
    ///
    #[cfg(any(target_arch = "wasm32", test))]
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
    use super::EXAMPLES;

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
}
