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
pub(crate) const EXAMPLES: [Example; 2] = [
    Example {
        name: "Cell Tracker",
        text: include_bytes!("../../examples/cell-tracker.orcvs"),
    },
    Example {
        name: "Orca Tracker",
        text: include_bytes!("../../examples/tracker.orcvs"),
    },
];

impl Example {
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
