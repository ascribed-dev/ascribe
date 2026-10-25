//! The addresses a base revision published, for `ascribe check` to compare
//! with the working tree's (`ascribe_check::addresses`), which can't read
//! git itself, and the moves git sees since. The base is chosen as `ascribe
//! diff` chooses it.

use std::collections::BTreeMap;
use std::path::Path;

use ascribe_check::addresses::Base as Published;
use ascribe_core::{FileId, RelPath};
use ascribe_model::{ADDRESSES_FILE, AddressFile};

use crate::git::FileChange;
use crate::{Base, DiffError, Repository, Revision};

/// The addresses a base revision published.
#[derive(Clone, Debug)]
pub struct BaseAddresses {
    /// The base.
    pub base: Base,
    /// What it published, its own `ascribe.addresses`, its pages' text, and
    /// the pages git sees renamed since; `None` when there's no project at
    /// the base.
    pub published: Option<Published>,
}

impl BaseAddresses {
    /// The base as a message names it: the revision asked for, and its
    /// commit when that's something else.
    pub fn named(&self) -> String {
        named(&self.base)
    }
}

/// `base` as a message names it.
fn named(base: &Base) -> String {
    let short = &base.commit[..base.commit.len().min(12)];
    if base.requested.starts_with(short) {
        format!("`{}`", base.requested)
    } else {
        format!("`{}` ({short})", base.requested)
    }
}

/// The addresses the project at `root` published at the base: `requested`,
/// or the repository's default branch; its merge base with `HEAD` unless
/// `exact`. Listed with the library, with no output written. `content_root`
/// is the working tree's, which the renames git sees are given from.
///
/// # Errors
///
/// The base can't be read: no repository, no git, a revision that isn't
/// there, or no merge base.
pub fn base_addresses(
    root: &Path,
    content_root: &RelPath,
    requested: Option<&str>,
    exact: bool,
) -> Result<BaseAddresses, DiffError> {
    let repo = Repository::discover(root)?;
    let base = repo.base(requested, exact)?;
    let published = match Revision::read(&repo, base.compared())? {
        Some(revision) => {
            let mut published = Published::of(&revision.project());
            published.renames = renames(&repo, base.compared(), content_root)?;
            Some(published)
        }
        None => None,
    };
    Ok(BaseAddresses { base, published })
}

/// The source files git sees renamed between `commit` and the working tree,
/// each old content path with its new one, both under `content_root`: a
/// rename git sees below its similarity threshold isn't one, and neither is
/// a copy. Only files git tracks are seen, so a move made with `git mv` is,
/// and one whose new file isn't added yet isn't.
///
/// # Errors
///
/// [`DiffError::Git`] when the files can't be listed.
pub fn renames(
    repo: &Repository,
    commit: &str,
    content_root: &RelPath,
) -> Result<BTreeMap<String, String>, DiffError> {
    let content_dir = repo
        .project_dir()
        .join(content_root.as_str())
        .map_err(|e| DiffError::Path(e.to_string()))?;
    let in_content = |path: &RelPath| path.relative_to(&content_dir).filter(|p| !p.is_root());
    Ok(repo
        .changed_files(commit)?
        .into_iter()
        .filter_map(|(now, change)| match change {
            FileChange::Renamed(was) => {
                Some((in_content(&was)?.to_string(), in_content(&now)?.to_string()))
            }
            FileChange::Added | FileChange::Modified => None,
        })
        .filter(|(was, now)| was.ends_with(".md") && now.ends_with(".md"))
        .collect())
}

/// The number of entries of the `ascribe.addresses` the project at `root`
/// had at the base, chosen as [`base_addresses`] chooses it: `None` when
/// the base had none that reads, or can't be read. For a project that has
/// no file now, to say what deleting it dropped; it reads the one file, not
/// the project.
pub fn base_file_entries(root: &Path, requested: Option<&str>, exact: bool) -> Option<usize> {
    let repo = Repository::discover(root).ok()?;
    let base = repo.base(requested, exact).ok()?;
    let path = repo.project_dir().join(ADDRESSES_FILE).ok()?;
    let tree = repo
        .tree_in(base.compared(), std::slice::from_ref(&path))
        .ok()?;
    let entry = tree.files.get(&path).filter(|e| !e.symlink)?;
    let bytes = repo.read_blobs(&[entry.object.as_str()]).ok()?.pop()?;
    let file = AddressFile::parse(&String::from_utf8(bytes).ok()?, FileId::new(0)).ok()?;
    Some(file.moved.len() + file.retired.len())
}
