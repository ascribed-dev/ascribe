//! The files beside `ascribe.toml` that aren't sources: `ascribe.nav`,
//! `ascribe.addresses`, `ascribe.lock`, `ascribe.permalinks`, and
//! `ascribe_check`'s `ascribe.baseline`.
//!
//! Each is a [`SideFile`]: where it is, the id its locations carry, how it
//! reads, and what a change to it makes stale. `ascribe_check::side_files`
//! lists them, and the project loader, the language server, and the source
//! index go through that list instead of naming each file. ARCHITECTURE.md's
//! "Side files" says what adding one takes.

use std::any::Any;
use std::sync::Arc;

use ascribe_core::{FileId, Issue, RelPath};
use ascribe_model::{
    ADDRESSES_FILE, AddressFile, LOCK_FILE, NAV_FILE, NavFile, PERMALINKS_FILE, PermalinkList,
};

use crate::FileSystem;

/// A file beside `ascribe.toml` that isn't a source.
pub trait SideFile: 'static {
    /// Its path from the project root.
    const PATH: &'static str;
    /// The id of the file, for locations in it.
    const ID: FileId;
    /// What a change to it makes stale.
    const STALE: Stale;
    /// What it reads as: the file, or its problems.
    type Read: Send + Sync + 'static;
    /// Reads its text, locating its problems in [`SideFile::ID`].
    fn parse(text: &str) -> Self::Read;
}

/// What a change to a side file makes stale, beyond the file's own
/// problems: what the language server does when the editor or the disk
/// changes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stale {
    /// The source index reads it ([`Indexed`]), so the index is given it
    /// again ([`crate::incremental::Change::SideFiles`]).
    pub index: bool,
    /// The checks across the project compare it with the content, and
    /// report on it, so they run again.
    pub across: bool,
    /// It holds back problems in every file, so every file is checked again.
    pub every_file: bool,
    /// The editor's open buffer of it wins over the file on disk.
    pub buffered: bool,
}

/// `ascribe.lock` (SPEC §7.4): the pins of the sources in other
/// repositories.
pub struct Lock;

impl SideFile for Lock {
    const PATH: &'static str = LOCK_FILE;
    /// Past any source file's, and before the code files'.
    const ID: FileId = FileId::new(0x7FFF_FFFF);
    /// Nothing beyond the file-level checks, which read it each round.
    const STALE: Stale = Stale {
        index: false,
        across: false,
        every_file: false,
        buffered: false,
    };
    type Read = Result<ascribe_model::Lock, Vec<Issue>>;
    fn parse(text: &str) -> Self::Read {
        ascribe_model::Lock::parse(text, Self::ID)
    }
}

/// `ascribe.permalinks` (SPEC §7.5): the published list of permalinks.
pub struct Permalinks;

impl SideFile for Permalinks {
    const PATH: &'static str = PERMALINKS_FILE;
    /// Just before the lock's.
    const ID: FileId = FileId::new(0x7FFF_FFFE);
    const STALE: Stale = Stale {
        index: false,
        across: true,
        every_file: false,
        buffered: false,
    };
    type Read = Result<PermalinkList, Vec<Issue>>;
    fn parse(text: &str) -> Self::Read {
        PermalinkList::parse(text, Self::ID)
    }
}

/// `ascribe.nav` (SPEC §7.8): the order of the navigation.
pub struct Nav;

impl SideFile for Nav {
    const PATH: &'static str = NAV_FILE;
    /// Just before the permalinks list's.
    const ID: FileId = FileId::new(0x7FFF_FFFD);
    const STALE: Stale = Stale {
        index: true,
        across: true,
        every_file: false,
        buffered: true,
    };
    type Read = Result<NavFile, Vec<Issue>>;
    fn parse(text: &str) -> Self::Read {
        NavFile::parse(text, Self::ID)
    }
}

/// `ascribe.addresses` (SPEC §7.10): what happened to each address and
/// permalink the content no longer has.
pub struct Addresses;

impl SideFile for Addresses {
    const PATH: &'static str = ADDRESSES_FILE;
    /// Just before the baseline's.
    const ID: FileId = FileId::new(0x7FFF_FFFB);
    const STALE: Stale = Stale {
        index: true,
        across: true,
        every_file: false,
        buffered: true,
    };
    type Read = Result<AddressFile, Vec<Issue>>;
    fn parse(text: &str) -> Self::Read {
        AddressFile::parse(text, Self::ID)
    }
}

/// A side file, without its type: what a list of them holds.
#[derive(Clone, Copy)]
pub struct Entry {
    /// [`SideFile::PATH`].
    pub path: &'static str,
    /// [`SideFile::ID`].
    pub id: FileId,
    /// [`SideFile::STALE`].
    pub stale: Stale,
    parse: fn(&str) -> Arc<dyn Any + Send + Sync>,
}

impl Entry {
    /// The entry of `F`.
    pub const fn of<F: SideFile>() -> Entry {
        Entry {
            path: F::PATH,
            id: F::ID,
            stale: F::STALE,
            parse: parse_any::<F>,
        }
    }

    /// Whether a change to the file on disk is followed apart from any other
    /// file's: whether anything is stale beyond what the file-level checks
    /// read each round.
    pub fn watched(&self) -> bool {
        self.stale.index || self.stale.across || self.stale.every_file
    }

    /// Reads the file through `fs`: its text, lossily when it isn't UTF-8,
    /// and what it reads as ([`SideFile::Read`], taken back out with
    /// [`Held::get`]). `None` when there's no file.
    pub fn read(&self, fs: &dyn FileSystem) -> Option<Held> {
        let text = text(fs, self.path)?;
        let read = (self.parse)(&text);
        Some(Held {
            path: self.path,
            text,
            read,
        })
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry")
            .field("path", &self.path)
            .field("id", &self.id)
            .field("stale", &self.stale)
            .finish_non_exhaustive()
    }
}

fn parse_any<F: SideFile>(text: &str) -> Arc<dyn Any + Send + Sync> {
    Arc::new(F::parse(text))
}

/// A side file as [`Entry::read`] read it.
#[derive(Clone)]
pub struct Held {
    path: &'static str,
    text: String,
    read: Arc<dyn Any + Send + Sync>,
}

impl Held {
    /// The file's path from the project root.
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Its text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// What it read as, when it's `F`.
    pub fn get<F: SideFile>(&self) -> Option<&F::Read> {
        (self.path == F::PATH)
            .then(|| self.read.downcast_ref::<F::Read>())
            .flatten()
    }
}

impl std::fmt::Debug for Held {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Held")
            .field("path", &self.path)
            .field("text", &self.text)
            .finish_non_exhaustive()
    }
}

/// A side file's text, read through `fs`, lossily when it isn't UTF-8.
fn text(fs: &dyn FileSystem, path: &str) -> Option<String> {
    let bytes = fs.read_file(&RelPath::parse(path).ok()?).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// The side files the source index reads, as it reads them: each when it's
/// there, is UTF-8, and reads; a file that doesn't is its check's to report.
/// Their entries are the ones whose [`Stale::index`] is set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Indexed {
    /// `ascribe.nav`, for the project's navigation.
    pub nav: Option<Arc<NavFile>>,
    /// `ascribe.addresses`, for what it records of permalinks.
    pub addresses: Option<Arc<AddressFile>>,
}

impl Indexed {
    /// The side files the index reads, read through `fs`.
    pub fn read(fs: &dyn FileSystem) -> Indexed {
        Indexed {
            nav: indexed::<Nav, _>(fs),
            addresses: indexed::<Addresses, _>(fs),
        }
    }
}

fn indexed<F, T>(fs: &dyn FileSystem) -> Option<Arc<T>>
where
    F: SideFile<Read = Result<T, Vec<Issue>>>,
{
    let bytes = fs.read_file(&RelPath::parse(F::PATH).ok()?).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    F::parse(&text).ok().map(Arc::new)
}
