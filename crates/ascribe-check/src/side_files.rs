//! Every side file a project has: the files beside `ascribe.toml` that aren't
//! sources ([`ascribe_resolve::side_file`]). [`Project`](crate::Project)
//! reads each one listed here, the language server follows each through its
//! [`Stale`](ascribe_resolve::side_file::Stale), and a report names each.

use ascribe_core::FileId;
use ascribe_resolve::side_file::{Addresses, Entry, Lock, Nav, Permalinks, SideFile, Stale};

use crate::baseline::{self, BASELINE_FILE, BASELINE_FILE_ID, BaselineError};

/// The side files, in the order a report lists them, after the content
/// model and before the sources.
pub const SIDE_FILES: &[Entry] = &[
    Entry::of::<Lock>(),
    Entry::of::<Permalinks>(),
    Entry::of::<Nav>(),
    Entry::of::<Addresses>(),
    Entry::of::<Baseline>(),
];

/// The side file at `path`, from the project root.
pub fn side_file(path: &str) -> Option<&'static Entry> {
    SIDE_FILES.iter().find(|entry| entry.path == path)
}

/// The side file whose locations have the id `id`.
pub fn side_file_by_id(id: FileId) -> Option<&'static Entry> {
    SIDE_FILES.iter().find(|entry| entry.id == id)
}

/// `ascribe.baseline`: the problems a project holds back
/// ([`crate::baseline`]).
pub struct Baseline;

impl SideFile for Baseline {
    const PATH: &'static str = BASELINE_FILE;
    const ID: FileId = BASELINE_FILE_ID;
    const STALE: Stale = Stale {
        index: false,
        across: false,
        every_file: true,
        buffered: true,
    };
    type Read = Result<baseline::Baseline, BaselineError>;
    fn parse(text: &str) -> Self::Read {
        baseline::Baseline::parse(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_ids_are_distinct() {
        for (i, a) in SIDE_FILES.iter().enumerate() {
            for b in &SIDE_FILES[i + 1..] {
                assert_ne!(a.path, b.path);
                assert_ne!(a.id, b.id);
            }
        }
    }

    /// `Indexed` reads the files whose `Stale::index` is set, and no others.
    #[test]
    fn the_index_reads_the_files_that_say_so() {
        let indexed: Vec<&str> = SIDE_FILES
            .iter()
            .filter(|entry| entry.stale.index)
            .map(|entry| entry.path)
            .collect();
        assert_eq!(indexed, [Nav::PATH, Addresses::PATH]);
    }
}
