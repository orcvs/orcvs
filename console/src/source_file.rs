//!
//! The Source File the native console has open, if any, and whether the
//! running Source has changed since that file was opened or saved.
//!
//! Native only: the web opens and saves no file.
//!

use std::io::Read as _;
use std::path::{Path, PathBuf};

use orcvs::grid::{COL_COUNT, ROW_COUNT};
use orcvs::source::{RevisionId, Source, SourceCommander, file};

/// A Source File's extension, without its dot.
pub(crate) const EXTENSION: &str = "orcvs";

///
/// The most bytes a Source File the console can open holds: every line of
/// the one Grid full and ended by CRLF. A longer file is refused unread.
///
const MAX_SOURCE_FILE_BYTES: usize = ROW_COUNT * (COL_COUNT + 2);

///
/// The Source the Source File at `path` holds, or why it cannot be opened:
/// the file could not be read, or `orcvs::source::file::read` refused it,
/// naming the line and column.
///
/// Reads at most one byte past [`MAX_SOURCE_FILE_BYTES`]; a prefix that long
/// already holds the refusal the whole file would get.
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
/// **Unsaved** means the running Source's Cells differ from those, so
/// typing a character and deleting it again leaves nothing unsaved, and a
/// Tick's write can make the Source unsaved.
///
/// The comparison copies the Source, so it runs only when the Source's
/// [`SourceCommander::revision`] differs from the one last answered for.
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
    /// No file, over a Source whose Cells are `saved`.
    ///
    pub(crate) fn untitled(saved: String) -> Self {
        Self {
            path: None,
            saved,
            answered: None,
        }
    }

    ///
    /// Records that `saved` was just written to `path`, which becomes the open
    /// file.
    ///
    pub(crate) fn saved(&mut self, path: PathBuf, saved: String) {
        self.path = Some(path);
        self.saved = saved;
        self.answered = None;
    }

    ///
    /// Names the file the Source was just opened from.
    ///
    pub(crate) fn name(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// Where the open file is, if the Source came from or went to one.
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    ///
    /// Whether `source` holds Cells other than the ones last opened or saved.
    ///
    pub(crate) fn unsaved(&mut self, source: &SourceCommander) -> bool {
        let revision = source.revision();
        if let Some((answered, unsaved)) = self.answered
            && answered == revision
        {
            return unsaved;
        }
        // One lock for the revision and the Cells, so the answer is kept for
        // the revision it was reached on.
        let mut answer = (revision, true);
        source.read_source(|source| {
            answer = (source.revision(), source.cells() != self.saved.as_bytes());
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

///
/// `path`, ending in the Source File extension. The extension is appended,
/// not substituted, so `loop-v1.2` keeps its dot; a name already ending
/// `.orcvs`, in any case, is kept as it is.
///
pub(crate) fn with_source_file_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION))
    {
        return path;
    }
    let mut named = path.into_os_string();
    named.push(".");
    named.push(EXTENSION);
    PathBuf::from(named)
}

///
/// Writes `text` to `path` without ever leaving a truncated file there: the
/// text goes to a new hidden file beside it, is flushed, and is renamed over
/// `path`. A failure at any step removes that file and leaves `path` as it
/// was.
///
/// Beside rather than in the temporary directory, because a rename across
/// file systems is a copy. A `path` that exists is written where its links
/// lead, so a linked Source File stays linked, and the new file takes its
/// permissions; a new file gets default permissions.
///
pub(crate) fn write_beside_then_rename(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let existing = std::fs::metadata(path).ok();
    let resolved = match existing {
        Some(_) => std::fs::canonicalize(path)?,
        None => path.to_path_buf(),
    };
    let path = resolved.as_path();
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("the path names no file"))?;
    let beside = path.with_file_name(format!(
        ".{}.{}-{}.saving",
        name.to_string_lossy(),
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            // After the write, so a read-only original cannot refuse it.
            match &existing {
                Some(existing) => file.set_permissions(existing.permissions()),
                None => Ok(()),
            }
        })
        .and_then(|()| std::fs::rename(&beside, path));
    if written.is_err() {
        // Nothing to do if it is already gone; the write's own error is the
        // one to report.
        let _ = std::fs::remove_file(&beside);
    }
    written
}

/// The name a Source with no file goes by.
const UNTITLED: &str = "Untitled";
/// What precedes the name in the title while the Source is unsaved.
const UNSAVED_MARKER: &str = "• ";

#[cfg(test)]
mod tests {
    use orcvs::grid::Grid;
    use orcvs::source::SourceCommander;

    use super::{
        MAX_SOURCE_FILE_BYTES, OpenSourceFile, read_source_file, with_source_file_extension,
        write_beside_then_rename,
    };
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

    ///
    /// The names in `dir`, sorted: what a write left there.
    ///
    fn entries(dir: &TempDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir.path())
            .expect("the test directory")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_write_replaces_the_file_whole_and_leaves_nothing_beside_it() {
        let dir = TempDir::new();
        let path = dir.write("loop.orcvs", &"x".repeat(4096));
        write_beside_then_rename(&path, "1\n").expect("a write");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "1\n");
        write_beside_then_rename(&dir.path().join("new.orcvs"), "2\n").expect("a write");
        assert_eq!(entries(&dir), ["loop.orcvs", "new.orcvs"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_write_through_a_link_replaces_the_file_it_names_and_keeps_the_link() {
        let dir = TempDir::new();
        let target = dir.write("target.orcvs", "old\n");
        let link = dir.path().join("link.orcvs");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        write_beside_then_rename(&link, "1\n").expect("a write");

        assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "1\n");
        assert_eq!(entries(&dir), ["link.orcvs", "target.orcvs"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_write_keeps_the_permissions_of_the_file_it_replaces() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = TempDir::new();
        let path = dir.write("private.orcvs", "old\n");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();

        write_beside_then_rename(&path, "1\n").expect("a write");

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn a_failed_write_leaves_the_target_and_nothing_beside_it() {
        let dir = TempDir::new();
        // A directory holding a file: no rename can replace it.
        let target = dir.path().join("taken.orcvs");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("kept"), "kept").unwrap();

        assert!(write_beside_then_rename(&target, "1\n").is_err());
        assert_eq!(entries(&dir), ["taken.orcvs"], "the file beside was left");
        assert_eq!(
            std::fs::read_to_string(target.join("kept")).unwrap(),
            "kept"
        );

        let missing = dir.path().join("no-such-directory").join("loop.orcvs");
        assert!(write_beside_then_rename(&missing, "1\n").is_err());
    }

    #[test]
    fn a_bare_name_takes_the_source_file_extension() {
        assert_eq!(
            with_source_file_extension("/music/loop".into()),
            std::path::PathBuf::from("/music/loop.orcvs")
        );
        assert_eq!(
            with_source_file_extension("/music/loop.orcvs".into()),
            std::path::PathBuf::from("/music/loop.orcvs")
        );
        assert_eq!(
            with_source_file_extension("/music/loop.ORCVS".into()),
            std::path::PathBuf::from("/music/loop.ORCVS")
        );
        assert_eq!(
            with_source_file_extension("/music/loop-v1.2".into()),
            std::path::PathBuf::from("/music/loop-v1.2.orcvs")
        );
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
