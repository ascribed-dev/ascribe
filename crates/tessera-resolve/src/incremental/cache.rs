//! Caches: parsed files by content, and resolved pages by build.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use tessera_core::{FileId, RelPath, Router};
use tessera_model::Build;
use tessera_syntax::ParsedDocument;

use super::Affected;
use super::signature::hash_of;
use crate::build::ResolvedPage;
use crate::project::Project;

struct ParseEntry {
    hash: u64,
    /// What the parser read of the model when it made this.
    model: u64,
    source: Arc<str>,
    document: Arc<ParsedDocument>,
}

/// One parse per file id, valid while the file's text (by content hash, and
/// then by comparison, so a collision can't return a wrong tree) and the
/// model's directive keywords and note types are what they were.
#[derive(Default)]
pub(crate) struct ParseCache {
    entries: HashMap<FileId, ParseEntry>,
}

impl ParseCache {
    /// The cached parse of a file, if `source` is the text it was made from
    /// and the model's parse-relevant part (`model`) is the same.
    pub(crate) fn get(&self, id: FileId, source: &str, model: u64) -> Option<Arc<ParsedDocument>> {
        let entry = self.entries.get(&id)?;
        (entry.model == model && entry.hash == hash_of(source) && *entry.source == *source)
            .then(|| entry.document.clone())
    }

    pub(crate) fn put(
        &mut self,
        id: FileId,
        source: &Arc<str>,
        model: u64,
        document: &Arc<ParsedDocument>,
    ) {
        self.entries.insert(
            id,
            ParseEntry {
                hash: hash_of(&**source),
                model,
                source: source.clone(),
                document: document.clone(),
            },
        );
    }

    pub(crate) fn forget(&mut self, id: FileId) {
        self.entries.remove(&id);
    }
}

/// Resolved pages kept between updates.
///
/// A [`Project::resolve_page`] call builds a fresh [`BuildResolver`]
/// (`crate::BuildResolver`) each time and forgets what it worked out. This
/// keeps each page's result, per build, until an update says it may have
/// changed: hand it each update's [`Affected`] with [`ResolvedCache::apply`].
///
/// The cache assumes one [`Router`] for its whole life (a router is part of
/// what a page resolves to, and isn't compared); make another cache to use a
/// different one.
#[derive(Default)]
pub struct ResolvedCache {
    /// By build name, then page. `None` records a page the build doesn't
    /// publish.
    pages: BTreeMap<String, BTreeMap<RelPath, Option<Arc<ResolvedPage>>>>,
    computed: u64,
}

impl ResolvedCache {
    /// An empty cache.
    pub fn new() -> ResolvedCache {
        ResolvedCache::default()
    }

    /// Forgets the pages an update says may have changed. Every update since
    /// the cache was last used must be applied, in order; if any was missed,
    /// use [`ResolvedCache::clear`].
    pub fn apply(&mut self, affected: &Affected) {
        for pages in self.pages.values_mut() {
            if affected
                .model
                .is_some_and(|m| m > super::ModelImpact::Warnings)
            {
                pages.clear();
                continue;
            }
            for path in affected.re_resolve.iter().chain(&affected.removed) {
                pages.remove(path);
            }
        }
    }

    /// Forgets everything.
    pub fn clear(&mut self) {
        self.pages.clear();
    }

    /// The page as `build` publishes it, from the cache when it's still good.
    /// `project` must be the snapshot the last update produced.
    pub fn resolve_page(
        &mut self,
        project: &Project,
        page: &RelPath,
        build: &Build,
        router: &dyn Router,
    ) -> Option<Arc<ResolvedPage>> {
        let pages = self.pages.entry(build.name.clone()).or_default();
        if let Some(hit) = pages.get(page) {
            return hit.clone();
        }
        let resolved = project.resolve_page(page, build, router).map(Arc::new);
        self.computed += 1;
        pages.insert(page.clone(), resolved.clone());
        resolved
    }

    /// How many pages have been resolved (cache misses) so far.
    pub fn computed(&self) -> u64 {
        self.computed
    }
}
