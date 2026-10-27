//! A `FileSystem` that counts what's read through it: how many times the
//! sources are listed, and how many times each source is read. It wraps
//! another, which answers.

use std::collections::BTreeMap;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use ascribe_core::RelPath;
use ascribe_resolve::{FileSystem, Probe, Sources};

/// The file system `inner`, with each listing of the sources and each read
/// of one counted.
pub struct Counting {
    inner: Arc<dyn FileSystem + Send + Sync>,
    listed: AtomicUsize,
    reads: Mutex<BTreeMap<RelPath, usize>>,
}

impl Counting {
    /// `inner`, counted.
    pub fn new(inner: Arc<dyn FileSystem + Send + Sync>) -> Arc<Counting> {
        Arc::new(Counting {
            inner,
            listed: AtomicUsize::new(0),
            reads: Mutex::default(),
        })
    }

    /// How many times the sources have been listed.
    pub fn listed(&self) -> usize {
        self.listed.load(Ordering::SeqCst)
    }

    /// How many times each source has been read, by content path.
    pub fn reads(&self) -> BTreeMap<RelPath, usize> {
        self.reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl FileSystem for Counting {
    fn sources(&self) -> Sources {
        self.listed.fetch_add(1, Ordering::SeqCst);
        self.inner.sources()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        *self
            .reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(path.clone())
            .or_default() += 1;
        self.inner.read(path)
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.inner.probe(project_path)
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.inner.read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.inner.real_path(project_path)
    }

    fn size(&self, project_path: &RelPath) -> Option<u64> {
        self.inner.size(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.inner.files_in(project_dir)
    }
}
