//! A project's permalinks against its list, `ascribe.permalinks` (SPEC
//! §7.5): every permalink the content declares and whether the list has it
//! ([`list`]), what bringing the list up to date changes ([`sync`]), and
//! whether names a product links to are in a list ([`verify`]), which needs
//! the list alone.

use ascribe_core::{FileId, Issue};
use ascribe_model::{PERMALINKS_FILE, PermalinkList};
use ascribe_resolve::{DeclaredPermalink, Project};
use serde::Serialize;

use crate::lines::{Lines, project_path};
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe permalinks list` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PermalinksAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// Whether the project has `ascribe.permalinks`. Without it, every
    /// permalink is `not_listed`, and nothing is `removed`.
    pub has_list: bool,
    /// Every permalink the content declares, in path order and then in
    /// document order, a page's own first. A name declared in several
    /// places is here once for each.
    pub permalinks: Vec<DeclaredEntry>,
    /// The names the list has that the content no longer declares, by name.
    pub removed: Vec<RemovedEntry>,
}

/// A permalink the content declares.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct DeclaredEntry {
    /// Its name.
    pub name: String,
    /// Where it's declared, as the list writes it: the file's path from the
    /// content root, and `#` and the id for a heading.
    pub place: String,
    /// The file it's declared in, from the project root, as `ascribe check`
    /// reports it.
    pub file: String,
    /// The line of its `@id` or its frontmatter `permalink`, from 1.
    pub line: u32,
    /// Whether the list has it: `listed` at one of the places its name is
    /// declared, `moved` when the list has it somewhere else, or
    /// `not_listed`.
    pub state: ListState,
    /// The place the list has for it, when that's somewhere else.
    pub listed_at: Option<String>,
}

/// Whether the list has a permalink where it's declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ListState {
    /// At one of the places its name is declared.
    Listed,
    /// At a place that doesn't declare it any more.
    Moved,
    /// Not at all.
    NotListed,
}

/// A name the list has that nothing declares.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RemovedEntry {
    /// Its name.
    pub name: String,
    /// Where the list says it was.
    pub place: String,
}

/// The project's permalinks, against its list when it has one.
///
/// # Errors
///
/// The project's `ascribe.permalinks` can't be read.
pub fn list(
    project: &Project,
    list: Option<&Result<PermalinkList, Vec<Issue>>>,
) -> Result<PermalinksAnswer, QueryError> {
    let list = readable(list)?;
    let declared = project.declared_permalinks();
    let places = DeclaredPermalink::places(&declared);
    let mut lines = Lines::new(project);
    let permalinks = declared
        .iter()
        .map(|permalink| {
            let listed = list.and_then(|l| l.permalinks.get(&permalink.name));
            let (state, listed_at) = match listed {
                None => (ListState::NotListed, None),
                Some(entry) if places[&permalink.name].contains(&entry.place) => {
                    (ListState::Listed, None)
                }
                Some(entry) => (ListState::Moved, Some(entry.place.clone())),
            };
            DeclaredEntry {
                name: permalink.name.clone(),
                place: permalink.place(),
                file: project_path(project, &permalink.file),
                line: lines.at(&permalink.file, permalink.at.span.start()).0,
                state,
                listed_at,
            }
        })
        .collect();
    let removed = list
        .map(|l| {
            l.permalinks
                .iter()
                .filter(|(name, _)| !places.contains_key(*name))
                .map(|(name, entry)| RemovedEntry {
                    name: name.clone(),
                    place: entry.place.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(PermalinksAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        has_list: list.is_some(),
        permalinks,
        removed,
    })
}

/// What `ascribe permalinks sync` does to the list.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SyncAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// Whether the list's file changes: it's new, or its text differs.
    pub changed: bool,
    /// Whether the project had no list, so this makes one.
    pub created: bool,
    /// The names added, by name.
    pub added: Vec<String>,
    /// The names whose place changed, by name.
    pub moved: Vec<MovedEntry>,
    /// The names the list keeps that the content no longer declares, by
    /// name. `ascribe check` warns about each.
    pub removed: Vec<String>,
    /// The list's text after the sync.
    #[serde(skip)]
    pub text: String,
}

/// A permalink whose place changed.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct MovedEntry {
    /// Its name.
    pub name: String,
    /// Where the list had it.
    pub from: String,
    /// Where it's declared now.
    pub to: String,
}

/// The list brought up to date with what the content declares: a new name
/// added, a moved one's place changed, and a name nothing declares kept.
/// It writes nothing; [`SyncAnswer::text`] is what the file should hold.
///
/// # Errors
///
/// The project's `ascribe.permalinks` can't be read.
pub fn sync(
    project: &Project,
    list: Option<&Result<PermalinkList, Vec<Issue>>>,
    text: Option<&str>,
) -> Result<SyncAnswer, QueryError> {
    let before = readable(list)?;
    let places = DeclaredPermalink::places(&project.declared_permalinks());
    let synced = before.cloned().unwrap_or_default().synced(&places);
    let new_text = synced.list.to_toml();
    let moved = synced
        .moved
        .into_iter()
        .map(|(name, from)| {
            let to = synced.list.permalinks[&name].place.clone();
            MovedEntry { name, from, to }
        })
        .collect();
    Ok(SyncAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        changed: text != Some(new_text.as_str()),
        created: before.is_none(),
        added: synced.added,
        moved,
        removed: synced.removed,
        text: new_text,
    })
}

/// What `ascribe permalinks verify` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct VerifyAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The list checked against, as given.
    pub list: String,
    /// The names the list has, in the order given, each once.
    pub found: Vec<String>,
    /// The names it doesn't, in the order given, each once.
    pub missing: Vec<String>,
}

/// Which of `names` the list `text` has. Only the list is read, so a
/// product's repository can check its links against a copy of it.
///
/// # Errors
///
/// `text` isn't a list this version of Ascribe reads.
pub fn verify(text: &str, shown_as: &str, names: &[String]) -> Result<VerifyAnswer, QueryError> {
    let list =
        PermalinkList::parse(text, FileId::new(0)).map_err(|issues| invalid(shown_as, &issues))?;
    let mut answer = VerifyAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        list: shown_as.to_owned(),
        found: Vec::new(),
        missing: Vec::new(),
    };
    for name in names {
        if answer.found.contains(name) || answer.missing.contains(name) {
            continue;
        }
        if list.permalinks.contains_key(name) {
            answer.found.push(name.clone());
        } else {
            answer.missing.push(name.clone());
        }
    }
    Ok(answer)
}

/// The list, when there's one that can be read.
fn readable(
    list: Option<&Result<PermalinkList, Vec<Issue>>>,
) -> Result<Option<&PermalinkList>, QueryError> {
    match list {
        None => Ok(None),
        Some(Ok(list)) => Ok(Some(list)),
        Some(Err(issues)) => Err(invalid(PERMALINKS_FILE, issues)),
    }
}

/// The error for a list, shown as `file`, that can't be read: its first
/// problem.
fn invalid(file: &str, issues: &[Issue]) -> QueryError {
    let detail = issues
        .first()
        .and_then(|issue| issue.args.iter().find(|a| a.name == "detail"))
        .map_or_else(String::new, |a| a.value.clone());
    QueryError::PermalinksListInvalid {
        file: file.to_owned(),
        detail,
    }
}
