//!
//! The Source File the native console has open, if any, and whether the
//! running Source has changed since that file was opened or saved
//! (`.scratch/menu-structure/issues/06`).
//!
//! Native only: the web opens and saves no file (`.scratch/menu-structure/
//! spec.md`, "Web file I/O is deferred"), so it tracks nothing and shows no
//! marker.
//!

use std::io::Read as _;
use std::path::{Path, PathBuf};

use orcvs::grid::{COL_COUNT, ROW_COUNT};
use orcvs::source::{RevisionId, Source, SourceCommander, file};

/// A Source File's extension, without its dot.
pub(crate) const EXTENSION: &str = "orcvs";

///
/// The most bytes a Source File the console can open holds: every line of
/// the one Grid full and ended by CRLF. A longer file is refused whatever it
/// holds, so no more than one byte past this is read — a file picked by
/// mistake is not read whole to learn that.
///
const MAX_SOURCE_FILE_BYTES: usize = ROW_COUNT * (COL_COUNT + 2);

///
/// The Source the Source File at `path` holds, or why it cannot be opened:
/// the file could not be read, or `orcvs::source::file::read` refused it,
/// naming the line and column.
///
/// Reads at most one byte past [`MAX_SOURCE_FILE_BYTES`]. A text that long
/// already holds a refusal — a 257th line or a line past 256 characters —
/// and `file::read` finds the first one in reading order, so the prefix is
/// refused where the whole file would be.
///
pub(crate) fn read_source_file(path: &Path) -> Result<Source, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|opened| {
            opened
                .take(MAX_SOURCE_FILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
        })
        .map_err(|error| format!("{} could not be read: {error}", path.display()))?;
    file::read(&bytes).map_err(|refusal| {
        format!(
            "{} is not a Source File, so it was not opened: {refusal}",
            path.display()
        )
    })
}

///
/// The open Source File: its path, if the Source came from or went to one,
/// and the Cells it held when it was last opened or saved.
///
/// **Unsaved** means the running Source's Cells differ from those. It is a
/// comparison of Cells, not a count of writes, because the console has no
/// Undo to walk back with: typing a character and deleting it again, or a
/// Tick writing a Cell back to what the file holds, leaves nothing to save,
/// and a counter would say otherwise. Playback's Ticks write the Source, so a
/// running program can leave its file unsaved without a key being pressed —
/// that is the truth of what Save would write.
///
/// **Cheap per frame.** The comparison copies the Source once, so it is asked
/// only when the Source is at a revision it has not answered for: the
/// revision identity ([`SourceCommander::revision`]) is read every frame
/// under the lock without copying a Cell, and the answer for it is kept.
/// While nothing writes, every frame reuses it; while Playback runs, it is
/// answered once per Tick.
///
pub(crate) struct OpenSourceFile {
    path: Option<PathBuf>,
    /// The Source's Cells, one byte each in Grid order, as they stood when
    /// they were last opened or saved.
    saved: String,
    /// The last revision the comparison answered for, and its answer.
    answered: Option<(RevisionId, bool)>,
}

impl OpenSourceFile {
    ///
    /// No file, over a Source whose Cells are `saved`: what the console
    /// starts on, and what a New or the Function reference opens.
    ///
    pub(crate) fn untitled(saved: String) -> Self {
        Self {
            path: None,
            saved,
            answered: None,
        }
    }

    ///
    /// Names the file the Source was just opened from.
    ///
    pub(crate) fn name(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// Where the open file is, if the Source came from or went to one.
    #[cfg(test)]
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    ///
    /// Whether `source` holds Cells other than the ones last opened or
    /// saved: what Save would change, and what discarding it would lose.
    ///
    pub(crate) fn unsaved(&mut self, source: &SourceCommander) -> bool {
        let revision = source.revision();
        if let Some((answered, unsaved)) = self.answered
            && answered == revision
        {
            return unsaved;
        }
        // The revision and the Cells are read under one lock, so the answer
        // is kept for the revision it was reached on even when a Tick writes
        // between the check above and this.
        let mut answer = (revision, true);
        source.read_source(|source| {
            answer = (source.revision(), source.snapshot() != self.saved);
        });
        self.answered = Some(answer);
        answer.1
    }

    ///
    /// The window title: the file's name, or `Untitled`, marked while the
    /// Source is unsaved.
    ///
    pub(crate) fn title(&self, unsaved: bool) -> String {
        let name = self
            .path
            .as_deref()
            .and_then(Path::file_name)
            .map_or_else(|| UNTITLED.into(), |name| name.to_string_lossy());
        let marker = if unsaved { UNSAVED_MARKER } else { "" };
        format!("{marker}{name} — Orcvs")
    }
}

/// The name a Source with no file goes by.
const UNTITLED: &str = "Untitled";
/// What precedes the name in the title while the Source is unsaved.
const UNSAVED_MARKER: &str = "• ";

#[cfg(test)]
mod tests {
    use orcvs::grid::Grid;
    use orcvs::source::SourceCommander;

    use super::{MAX_SOURCE_FILE_BYTES, OpenSourceFile, read_source_file};
    use crate::theme_registry::tests_support::TempDir;

    #[test]
    fn a_source_file_reads_as_its_source() {
        let dir = TempDir::new();
        let path = dir.write("loop.orcvs", "  1\r\n\n*\n");
        let source = read_source_file(&path).expect("a Source File");
        let snapshot = source.snapshot();
        assert_eq!(&snapshot[..3], "  1");
        assert_eq!(&snapshot[512..513], "*");
    }

    #[test]
    fn a_refused_or_unreadable_file_says_why_and_where() {
        let dir = TempDir::new();
        let path = dir.write("tabbed.orcvs", "..\n.\t");
        let refused = read_source_file(&path).err().expect("a tab is not a Cell");
        assert!(refused.contains("tabbed.orcvs"), "{refused}");
        assert!(refused.contains("line 2, column 2"), "{refused}");

        let missing = read_source_file(&dir.path().join("missing.orcvs"))
            .err()
            .expect("a missing file cannot be read");
        assert!(
            missing.contains("missing.orcvs could not be read"),
            "{missing}"
        );
    }

    #[test]
    fn a_file_longer_than_any_source_file_is_refused_without_reading_it_whole() {
        let dir = TempDir::new();
        let full = format!("{}\r\n", ".".repeat(256)).repeat(256);
        assert_eq!(full.len(), MAX_SOURCE_FILE_BYTES);
        let path = dir.write("full.orcvs", &full);
        assert!(
            read_source_file(&path).is_ok(),
            "the longest Source File was refused"
        );

        let path = dir.write("long.orcvs", &format!("{full}{}", "x".repeat(1 << 20)));
        let refused = read_source_file(&path).err().expect("a 257th line");
        assert!(refused.contains("line 257, column 1"), "{refused}");
    }

    fn written(source: &SourceCommander, index: usize, content: &str) {
        let cell = source.grid().cell_index(index).expect("inside the Grid");
        source
            .set(cell, content)
            .expect("a Cell the Source accepts");
    }

    #[test]
    fn a_source_is_unsaved_while_its_cells_differ_from_the_ones_opened() {
        let source = SourceCommander::new(Grid::with_shape(4, 2));
        let mut file = OpenSourceFile::untitled(source.snapshot());
        assert!(!file.unsaved(&source), "an untouched Source is unsaved");

        written(&source, 1, "a");
        assert!(file.unsaved(&source), "a written Cell is not unsaved");

        // No Undo: deleting what was typed is what brings it back.
        written(&source, 1, " ");
        assert!(
            !file.unsaved(&source),
            "a Source written back to its opened Cells is still unsaved"
        );
    }

    #[test]
    fn the_answer_is_kept_until_the_source_is_at_a_new_revision() {
        let source = SourceCommander::new(Grid::with_shape(4, 2));
        let mut file = OpenSourceFile::untitled(source.snapshot());
        assert!(!file.unsaved(&source));
        let answered = file.answered;
        assert!(!file.unsaved(&source));
        assert_eq!(
            file.answered, answered,
            "an unwritten Source was compared again"
        );

        written(&source, 0, "x");
        assert!(file.unsaved(&source));
        assert_ne!(file.answered, answered, "a written Source was not compared");
    }

    #[test]
    fn the_title_names_the_file_or_untitled_and_marks_unsaved() {
        let file = OpenSourceFile::untitled(String::new());
        assert_eq!(file.title(false), "Untitled — Orcvs");
        assert_eq!(file.title(true), "• Untitled — Orcvs");

        let file = OpenSourceFile {
            path: Some("/music/loop.orcvs".into()),
            saved: String::new(),
            answered: None,
        };
        assert_eq!(file.title(false), "loop.orcvs — Orcvs");
        assert_eq!(file.title(true), "• loop.orcvs — Orcvs");
        assert_eq!(file.path(), Some(std::path::Path::new("/music/loop.orcvs")));
    }
}
