//! The addresses the docs have published, between checks (SPEC §7.10).
//!
//! When the project has `ascribe.addresses`, a thread of its own
//! (`server::addresses_worker`) compares the addresses the project publishes
//! with the base's, as `ascribe check` does, and tells the client what it
//! found with [`NOTIFICATION`]: the problems `ascribe check` reports about
//! the addresses, at `ascribe.addresses`. They aren't a page's diagnostics,
//! and they aren't published as the author types (README decision 8): the
//! editor shows them with Check Every Build's problems, which a check of
//! every build replaces when it runs. The agents' hook runs the server
//! without this thread, so it never reports them.
//!
//! - **When.** The comparison runs once the project loads, and again after a
//!   save, a change on disk, or an edit to `ascribe.addresses` itself, a
//!   moment after the last of a burst ([`SETTLE`]): a move in the Explorer
//!   is a deletion and a creation, and an edit to the file the move's own.
//!   Not as a page is typed.
//! - **The base** is chosen as `ascribe check` chooses it: the merge base of
//!   `HEAD` and the repository's default branch. Resolving it is a couple of
//!   `git` processes, done each time; listing its addresses resolves every
//!   build of the project there, so it's done once for each base commit and
//!   version of Ascribe, and kept ([`Kept`]). The pages git sees renamed
//!   since are read again each time, since the working tree moves.
//! - **The working tree** is the editor's snapshot, unsaved pages included,
//!   with `ascribe.addresses` as the editor has it: its open buffer, so a
//!   move's entry counts before it's saved, and an undo takes it back.
//! - **What's sent** is `ascribe check`'s: the lost addresses and ids, a
//!   build only the base has, the file's own checks, and a base that can't
//!   be read, with the content model's levels and the baseline applied.
//!
//! The same round keeps each page's redirects, from `redirects.json` as
//! each build would write it ([`ascribe_emit::site::redirects`]), for This
//! page's "Also answers at" ([`Redirects::of`]).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ascribe_check::addresses::{Base as Published, Here};
use ascribe_check::{ADDRESSES_FILE_ID, Diagnostic};
use ascribe_core::{LineIndex, RelPath, diagnostics};
use ascribe_diff::{BaseInfo, Repository, Revision};
use ascribe_model::addresses::split_place;
use ascribe_model::{ADDRESSES_FILE, AddressFile, ContentModel};
use ascribe_resolve::addresses::BuildAddresses;
use ascribe_resolve::{AstroRouter, FileSystem, Snapshot};
use lsp_types::Location;
use serde::Serialize;

use crate::compute::to_lsp;
use crate::fsx::LayerFs;
use crate::position::Encoding;
use crate::uri::path_to_uri;

/// The notification that tells the client what the comparison found.
pub const NOTIFICATION: &str = "ascribe/addresses";

/// How long the comparison waits after a change for the next, so a burst
/// of them, as a move or a folder's, is compared once.
pub(crate) const SETTLE: Duration = Duration::from_millis(250);

/// The parameters of `ascribe/addresses`: what the last comparison found.
/// Sent once the project loads and after each change that compares again,
/// when it, or a page's old addresses (This page's "Also answers at"),
/// differ from what was sent last.
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AddressesParams {
    /// `ascribe.addresses`' `file:` URI; `null` when the project has no
    /// file, and nothing is compared.
    pub uri: Option<String>,
    /// The base compared with; `null` when there's no file, it doesn't
    /// read, or the base couldn't be read.
    pub base: Option<BaseInfo>,
    /// How many `address-lost` warnings there are: one for each page whose
    /// addresses are lost.
    pub lost: usize,
    /// The problems, all at `ascribe.addresses`, as `ascribe check`
    /// reports them: the protocol's diagnostics, each with the data a
    /// published diagnostic has, its fixes among it.
    #[cfg_attr(feature = "json-schema", schemars(with = "Vec<serde_json::Value>"))]
    pub diagnostics: Vec<lsp_types::Diagnostic>,
}

/// An old address that leads to a page: This page's "Also answers at".
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageOldAddress {
    /// The address, from the host's root, as `redirects.json`'s `from`.
    pub address: String,
    /// The `ascribe.addresses` key it comes from, as `redirects.json`'s
    /// `entry`.
    pub entry: String,
}

/// The base's addresses, listed once for a base commit and a version of
/// Ascribe.
pub(crate) struct Kept {
    /// The commit compared, and the version that listed it.
    key: (String, String),
    /// What the base published: `None` when there's no project there.
    published: Option<Published>,
}

/// Each page's old addresses, by its content path, from the last
/// comparison.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Redirects(BTreeMap<String, Vec<PageOldAddress>>);

impl Redirects {
    /// The old addresses that lead to the page at `path`, in address order.
    pub(crate) fn of(&self, path: &RelPath) -> Vec<PageOldAddress> {
        self.0.get(path.as_str()).cloned().unwrap_or_default()
    }
}

/// The comparison's state, under the server's lock.
#[derive(Default)]
pub(crate) struct State {
    /// Whether the server compares at all: an editor's, not the agents'
    /// hook's.
    pub enabled: bool,
    /// A change since the last comparison: compare again.
    pub dirty: bool,
    /// When the last change was.
    pub changed_at: Option<Instant>,
    /// Whether a comparison is under way.
    pub running: bool,
    /// The base listed last.
    kept: Option<Arc<Kept>>,
    /// The redirects the last comparison found.
    pub redirects: Arc<Redirects>,
    /// What was sent last, so the same isn't sent again.
    sent: Option<AddressesParams>,
}

impl State {
    /// Something changed that the comparison reads.
    pub(crate) fn changed(&mut self) {
        if self.enabled {
            self.dirty = true;
            self.changed_at = Some(Instant::now());
        }
    }

    /// Whether a comparison is waiting or under way.
    pub(crate) fn has_work(&self) -> bool {
        self.dirty || self.running
    }

    /// Forgets what the comparison found, for a project that's no longer
    /// served: the base it listed is another project's.
    pub(crate) fn forget(&mut self) {
        self.kept = None;
        self.redirects = Arc::default();
    }
}

/// What a comparison reads, taken under the lock.
pub(crate) struct Job {
    /// The project's folder.
    pub root: PathBuf,
    /// The content root.
    pub content_root: RelPath,
    pub snapshot: Snapshot,
    pub model: Arc<ContentModel>,
    /// The project's files as the editor has them, `ascribe.addresses`'
    /// buffer included.
    pub fs: Arc<LayerFs>,
    pub baseline: Arc<crate::baseline::Now>,
    pub encoding: Encoding,
    /// Ascribe's version, which keys the base listed.
    pub version: String,
    kept: Option<Arc<Kept>>,
}

impl Job {
    /// The job for the project loaded, when there is one.
    pub(crate) fn of(core: &crate::core::Core) -> Option<Job> {
        let loaded = core.loaded.as_ref()?;
        Some(Job {
            root: loaded.root.clone(),
            content_root: loaded.layout.content_root.clone(),
            snapshot: loaded.inc.snapshot(),
            model: loaded.model.clone(),
            fs: loaded.fs.clone(),
            baseline: loaded.baseline.clone(),
            encoding: core.encoding,
            version: core.version.clone(),
            kept: core.addresses.kept.clone(),
        })
    }
}

/// What a comparison found.
pub(crate) struct Outcome {
    pub root: PathBuf,
    pub params: AddressesParams,
    pub redirects: Redirects,
    /// The base it listed or kept, when it read one.
    kept: Option<Arc<Kept>>,
}

impl State {
    /// Keeps what `outcome` found, and the notification to send when it
    /// differs from the last: its problems, or the redirects, which This
    /// page shows, so the client asks for the page again.
    pub(crate) fn finish(&mut self, outcome: Outcome) -> Option<AddressesParams> {
        if outcome.kept.is_some() {
            self.kept = outcome.kept;
        }
        let redirected = *self.redirects != outcome.redirects;
        self.redirects = Arc::new(outcome.redirects);
        if !redirected && self.sent.as_ref() == Some(&outcome.params) {
            return None;
        }
        self.sent = Some(outcome.params.clone());
        Some(outcome.params)
    }
}

/// Compares the project of `job` with its base. `listed` is called with
/// the commit each time the base's addresses are listed.
pub(crate) fn compare(job: &Job, listed: &dyn Fn(&str)) -> Outcome {
    let none = |redirects| Outcome {
        root: job.root.clone(),
        params: AddressesParams::default(),
        redirects,
        kept: None,
    };
    let Some(text) = RelPath::parse(ADDRESSES_FILE)
        .ok()
        .and_then(|path| job.fs.read_file(&path).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
    else {
        return none(Redirects::default());
    };
    let Some(uri) = path_to_uri(&job.root.join(ADDRESSES_FILE)) else {
        return none(Redirects::default());
    };
    let index = LineIndex::new(&text);
    let to_lsp_all = |found: Vec<Diagnostic>| -> Vec<lsp_types::Diagnostic> {
        let related = |r: &ascribe_check::RelatedInfo| {
            (r.location.file == ADDRESSES_FILE_ID).then(|| Location {
                uri: uri.clone(),
                range: job.encoding.range(&index, r.location.span),
            })
        };
        found
            .iter()
            .map(|d| to_lsp(d, &index, job.encoding, &related))
            .collect()
    };
    let file = match AddressFile::parse(&text, ADDRESSES_FILE_ID) {
        Ok(file) => file,
        Err(issues) => {
            // `ascribe check` compares nothing then, and says why.
            let found = issues.iter().map(Diagnostic::from_issue).collect();
            let diagnostics = to_lsp_all(levels(job, &text, found));
            return Outcome {
                root: job.root.clone(),
                params: AddressesParams {
                    uri: Some(uri.as_str().to_owned()),
                    base: None,
                    lost: 0,
                    diagnostics,
                },
                redirects: Redirects::default(),
                kept: None,
            };
        }
    };
    let index_now = job.snapshot.project();
    let (now, redirects) = every_build(index_now, &file);
    let here = Here {
        file: &file,
        text: &text,
        index: index_now,
        builds: &now,
    };
    let (found, base, kept) = match base(job, listed) {
        // Nothing can have been published yet, so nothing is compared, as
        // `ascribe check` does (decision #1016).
        Err(_) if ascribe_diff::before_history(&job.root) => {
            (ascribe_check::addresses::entries(&here, None), None, None)
        }
        Err(problem) => {
            let mut out = vec![ascribe_check::addresses::base_unreadable(&problem)];
            out.extend(ascribe_check::addresses::entries(&here, None));
            (out, None, None)
        }
        Ok((info, named, kept, renames)) => match &kept.published {
            None => {
                let mut out = vec![ascribe_check::addresses::base_has_no_project(&named)];
                out.extend(ascribe_check::addresses::entries(&here, None));
                (out, Some(info), Some(kept))
            }
            Some(published) => {
                let mut published = published.clone();
                published.renames = renames;
                let mut out = ascribe_check::addresses::compare(&here, &published);
                out.extend(ascribe_check::addresses::entries(&here, Some(&published)));
                (out, Some(info), Some(kept))
            }
        },
    };
    let found = levels(job, &text, found);
    let lost = found
        .iter()
        .filter(|d| d.slug == diagnostics::ADDRESS_LOST)
        .count();
    Outcome {
        root: job.root.clone(),
        params: AddressesParams {
            uri: Some(uri.as_str().to_owned()),
            base,
            lost,
            diagnostics: to_lsp_all(found),
        },
        redirects,
        kept,
    }
}

/// `found` with the content model's levels, and without the advice
/// `ascribe.baseline` lists, as `ascribe check` reports them.
fn levels(job: &Job, text: &str, found: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let found = ascribe_check::apply_levels(&job.model.checks, found);
    crate::baseline::hold_in(&job.baseline, ADDRESSES_FILE, text, found)
        .diagnostics
        .into_iter()
        // The baseline's own entries are checked against every finding,
        // which these aren't.
        .filter(|d| d.slug != diagnostics::BASELINE_ENTRY_UNMATCHED)
        .collect()
}

/// Each build's addresses now, and each page's redirects in them, from
/// `file`: every build resolved once for both.
fn every_build(
    index: &ascribe_resolve::Project,
    file: &AddressFile,
) -> (Vec<BuildAddresses>, Redirects) {
    let mut addresses = Vec::new();
    let mut redirects: BTreeMap<String, Vec<PageOldAddress>> = BTreeMap::new();
    for build in &index.model().builds {
        let settings = ascribe_resolve::addresses::Settings::of(index.model(), build);
        let index = index.for_build(build);
        let router = AstroRouter::from_consumer(&index.model().consumer);
        let resolved = index.resolve_build(build, &router);
        addresses.push(ascribe_resolve::addresses::of_resolved(
            &index, build, &resolved, settings,
        ));
        let written = ascribe_emit::site::redirects::redirects(
            index.model(),
            &index,
            build,
            &resolved,
            Some(file),
        );
        for redirect in written.redirects {
            // The page the entry leads to in the end.
            let Some(page) = file
                .moved
                .get(&redirect.entry)
                .and_then(|entry| file.follow(&entry.value))
                .and_then(|target| split_place(&target).map(|(path, _)| path.to_owned()))
            else {
                continue;
            };
            let old = PageOldAddress {
                address: redirect.from,
                entry: redirect.entry,
            };
            let list = redirects.entry(page).or_default();
            if !list.contains(&old) {
                list.push(old);
            }
        }
    }
    for list in redirects.values_mut() {
        list.sort();
    }
    (addresses, Redirects(redirects))
}

/// The base read: what it is, as a message names it, what it published, and
/// the pages git sees renamed since.
type ReadBase = (BaseInfo, String, Arc<Kept>, BTreeMap<String, String>);

/// The base, or why it can't be read. Its addresses are listed again only
/// for a commit, or a version of Ascribe, other than the kept one's.
fn base(job: &Job, listed: &dyn Fn(&str)) -> Result<ReadBase, String> {
    let repo = Repository::discover(&job.root).map_err(|e| e.to_string())?;
    let base = repo.base(None, false).map_err(|e| e.to_string())?;
    let key = (base.compared().to_owned(), job.version.clone());
    let kept = match &job.kept {
        Some(kept) if kept.key == key => kept.clone(),
        _ => {
            listed(base.compared());
            let published = Revision::read(&repo, base.compared())
                .map_err(|e| e.to_string())?
                .map(|revision| Published::of(&revision.project()));
            Arc::new(Kept { key, published })
        }
    };
    let renames = ascribe_diff::renames(&repo, base.compared(), &job.content_root)
        .map_err(|e| e.to_string())?;
    let named = base.named();
    let info = BaseInfo {
        requested: base.requested,
        commit: base.commit,
        merge_base: base.merge_base,
    };
    Ok((info, named, kept, renames))
}
