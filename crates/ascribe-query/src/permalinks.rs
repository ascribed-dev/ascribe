//! A project's permalinks against its list, `ascribe.permalinks` (SPEC
//! §7.5): every permalink the content declares, and each name
//! `ascribe.addresses` moves or retires, and whether the list has it
//! ([`list`]), what bringing the list
//! up to date changes ([`sync`]), whether names a product links to are in a
//! list ([`verify`]), which needs the list alone, and which names retiring
//! a page or a folder ends ([`retiring`]).

use ascribe_core::{FileId, Issue};
use ascribe_model::{ADDRESSES_FILE, Current, End, PERMALINKS_FILE, PermalinkList};
use ascribe_resolve::{DeclaredPermalink, Project};
use serde::Serialize;

use crate::lines::{Lines, project_path};
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// The version of [`PermalinksAnswer`]'s schema: 2 since a retired
/// permalink has no `to`, and a name moved to a page is under `moved`
/// (decision #944a).
pub const PERMALINKS_LIST_SCHEMA_VERSION: u32 = 2;

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
    /// The names `ascribe.addresses` moves to another permalink, by name.
    pub aliases: Vec<AliasEntry>,
    /// The names `ascribe.addresses` moves to a page, by name.
    pub moved: Vec<MovedName>,
    /// The names `ascribe.addresses` retires, or moves to one that's
    /// retired, by name.
    pub retired: Vec<RetiredEntry>,
    /// The names the list has that the project no longer has, by name.
    pub removed: Vec<RemovedEntry>,
}

/// A name `ascribe.addresses` moves to another permalink: an alias.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AliasEntry {
    /// Its name.
    pub name: String,
    /// The name it was moved to.
    pub target: String,
    /// Whether the list has it, leading there: `listed`, or `not_listed`.
    pub state: ListState,
}

/// A name `ascribe.addresses` moves to a page.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct MovedName {
    /// Its name.
    pub name: String,
    /// Where its address leads now, as written: a page, and `#` and a
    /// heading's id.
    pub to: String,
    /// Whether the list has it, leading there: `listed`, or `not_listed`.
    pub state: ListState,
}

/// A name `ascribe.addresses` retires.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RetiredEntry {
    /// Its name.
    pub name: String,
    /// Why it was retired.
    pub reason: String,
    /// Whether the list has it as retired, with this reason: `listed`, or
    /// `not_listed`.
    pub state: ListState,
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

/// A name the list has that the project doesn't.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RemovedEntry {
    /// Its name.
    pub name: String,
    /// What the list has it as.
    pub was: ListedAs,
    /// Where the list says a permalink was declared; for an alias, the name
    /// it led to; for a moved one, where; for a retired one, its reason.
    pub place: String,
}

/// Which of the list's tables has a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ListedAs {
    /// `[permalinks]`: a permalink the content declared.
    Permalink,
    /// `[aliases]`: a name moved to another permalink.
    Alias,
    /// `[moved]`: a name moved to a page.
    Moved,
    /// `[retired]`: a retired permalink.
    Retired,
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
    let recorded = project.recorded_permalinks();
    let current = Current::new(places, &recorded);
    let aliases = current
        .aliases
        .iter()
        .map(|(name, target)| AliasEntry {
            name: name.clone(),
            target: target.clone(),
            state: listed(
                list.and_then(|l| l.aliases.get(name))
                    .is_some_and(|e| e.target == *target),
            ),
        })
        .collect();
    let moved = current
        .moved
        .iter()
        .map(|(name, to)| MovedName {
            name: name.clone(),
            to: to.clone(),
            state: listed(
                list.and_then(|l| l.moved.get(name))
                    .is_some_and(|e| e.place == *to),
            ),
        })
        .collect();
    let retired = current
        .retired
        .iter()
        .map(|(name, reason)| RetiredEntry {
            name: name.clone(),
            reason: reason.clone(),
            state: listed(
                list.and_then(|l| l.retired.get(name))
                    .is_some_and(|e| e.reason == *reason),
            ),
        })
        .collect();
    let removed = list.map(|l| removed(l, &current)).unwrap_or_default();
    Ok(PermalinksAnswer {
        schema_version: PERMALINKS_LIST_SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        has_list: list.is_some(),
        permalinks,
        aliases,
        moved,
        retired,
        removed,
    })
}

/// `listed` or `not_listed`.
fn listed(yes: bool) -> ListState {
    if yes {
        ListState::Listed
    } else {
        ListState::NotListed
    }
}

/// The names the list has that the project doesn't, by name.
fn removed(list: &PermalinkList, current: &Current) -> Vec<RemovedEntry> {
    let gone = |name: &&String| !current.has(name);
    let mut out: Vec<RemovedEntry> = list
        .permalinks
        .iter()
        .filter(|(n, _)| gone(n))
        .map(|(name, e)| (name, ListedAs::Permalink, &e.place))
        .chain(
            list.aliases
                .iter()
                .filter(|(n, _)| gone(n))
                .map(|(name, e)| (name, ListedAs::Alias, &e.target)),
        )
        .chain(
            list.moved
                .iter()
                .filter(|(n, _)| gone(n))
                .map(|(name, e)| (name, ListedAs::Moved, &e.place)),
        )
        .chain(
            list.retired
                .iter()
                .filter(|(n, _)| gone(n))
                .map(|(name, e)| (name, ListedAs::Retired, &e.reason)),
        )
        .map(|(name, was, place)| RemovedEntry {
            name: name.clone(),
            was,
            place: place.clone(),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
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
    /// Whether the list's file was written: it changes, and nothing said to
    /// write nothing (`--check`, `--dry-run`).
    pub written: bool,
    /// Whether the project had no list, so this makes one.
    pub created: bool,
    /// The names added, by name.
    pub added: Vec<String>,
    /// The names whose place changed, by name.
    pub moved: Vec<MovedEntry>,
    /// The names moved to another permalink recorded, new or leading
    /// somewhere new, by name.
    pub aliased: Vec<String>,
    /// The names moved to a page recorded, new or leading somewhere new, by
    /// name.
    pub forwarded: Vec<String>,
    /// The retired permalinks recorded, new or with a new reason, by name.
    pub retired: Vec<String>,
    /// The names the list keeps that the project no longer has, by name.
    /// `ascribe check` warns about each.
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

/// The list brought up to date with what the project has: a new name
/// added, a moved one's place changed, each name `ascribe.addresses` moves
/// or retires recorded as it has it, and a name the project doesn't have
/// kept.
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
    let current = Current::new(
        DeclaredPermalink::places(&project.declared_permalinks()),
        &project.recorded_permalinks(),
    );
    let synced = before.cloned().unwrap_or_default().synced(&current);
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
        written: false,
        created: before.is_none(),
        added: synced.added,
        moved,
        aliased: synced.aliased,
        forwarded: synced.moved_to,
        retired: synced.retired,
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
    /// The names the list has, as permalinks, or moved to another permalink
    /// or a page, in the order given, each once.
    pub found: Vec<String>,
    /// The names the list has as retired, in the order given, each once.
    pub retired: Vec<RetiredName>,
    /// The names it doesn't have at all, in the order given, each once.
    pub missing: Vec<String>,
}

/// A name the list has as retired.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RetiredName {
    /// The name.
    pub name: String,
    /// Why it was retired, as the docs' authors wrote it.
    pub reason: String,
}

/// Which of `names` the list `text` has, as a permalink, or moved, and
/// which it has as retired. Only the list is read, so a product's
/// repository can check its links against a copy of it.
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
        retired: Vec::new(),
        missing: Vec::new(),
    };
    for name in names {
        if answer.found.contains(name)
            || answer.missing.contains(name)
            || answer.retired.iter().any(|r| r.name == *name)
        {
            continue;
        }
        if list.permalinks.contains_key(name)
            || list.aliases.contains_key(name)
            || list.moved.contains_key(name)
        {
            answer.found.push(name.clone());
        } else if let Some(entry) = list.retired.get(name) {
            answer.retired.push(RetiredName {
                name: name.clone(),
                reason: entry.reason.clone(),
            });
        } else {
            answer.missing.push(name.clone());
        }
    }
    Ok(answer)
}

/// What to retire: names, and every permalink a page or the pages under a
/// folder hold.
#[derive(Debug, Default)]
pub struct Retire<'a> {
    /// Names gone from the content that the list has.
    pub names: &'a [String],
    /// A page, by its content path, whose permalinks are retired.
    pub page: Option<&'a str>,
    /// A folder, by its content path, whose pages' permalinks are retired.
    pub under: Option<&'a str>,
    /// The page or folder as the caller was given it, which an error names;
    /// its content path when `None`.
    pub named: Option<&'a str>,
}

/// The names [`Retire`] asks for, in order, each once: each name given,
/// which must be gone from the content and in the list; and each name
/// declared in the page or under the folder, or that the list has there,
/// which must not be declared anywhere else. It writes nothing.
///
/// # Errors
///
/// The list can't be read; a name is still declared elsewhere, already
/// moved or retired in `ascribe.addresses`, or not a permalink at all; or
/// the page or folder holds none.
pub fn retiring(
    project: &Project,
    list: Option<&Result<PermalinkList, Vec<Issue>>>,
    retire: &Retire<'_>,
) -> Result<Vec<String>, QueryError> {
    let list = readable(list)?;
    let declared = project.declared_permalinks();
    let recorded = project.recorded_permalinks();
    let refuse = |name: &str, why: String| QueryError::NotRetirable {
        name: name.to_owned(),
        why,
    };
    let retired = |name: &str| {
        recorded
            .get(name)
            .is_some_and(|r| matches!(r.end, End::Retired(_)))
    };
    let already = |name: &str| -> Result<(), QueryError> {
        match recorded.get(name) {
            None => Ok(()),
            Some(entry) if matches!(entry.end, End::Retired(_)) => {
                Err(refuse(name, format!("{ADDRESSES_FILE} retires it already")))
            }
            Some(entry) => Err(refuse(
                name,
                format!(
                    "{ADDRESSES_FILE} moves it to `{}` already; remove that entry first",
                    entry.value
                ),
            )),
        }
    };
    let mut out: Vec<String> = Vec::new();
    for name in retire.names {
        if out.contains(name) {
            continue;
        }
        already(name)?;
        if let Some(at) = declared.iter().find(|p| p.name == *name) {
            return Err(refuse(
                name,
                format!(
                    "`{}` still declares it; remove it from the content first, or retire what that page holds with --page",
                    at.place()
                ),
            ));
        }
        match list {
            Some(list) if list.permalinks.contains_key(name) => out.push(name.clone()),
            Some(_) => {
                return Err(refuse(
                    name,
                    format!(
                        "it isn't a permalink: nothing declares it, and {PERMALINKS_FILE} doesn't have it"
                    ),
                ));
            }
            None => {
                return Err(refuse(
                    name,
                    format!(
                        "nothing declares it, and there's no {PERMALINKS_FILE} to say it was a permalink"
                    ),
                ));
            }
        }
    }
    let (held, shown) = match (retire.page, retire.under) {
        (Some(page), _) => (
            page.trim_start_matches('/').to_owned(),
            retire.named.unwrap_or(page),
        ),
        (None, Some(folder)) => {
            let folder = folder.trim_start_matches('/').trim_end_matches('/');
            (
                format!("{folder}/"),
                retire.named.or(retire.under).unwrap_or_default(),
            )
        }
        (None, None) => return Ok(out),
    };
    let holds = |file: &str| {
        if retire.page.is_some() {
            file == held
        } else {
            held == "/" || file.starts_with(&held)
        }
    };
    let file_of = |place: &str| place.split('#').next().unwrap_or_default().to_owned();
    let mut found: Vec<String> = declared
        .iter()
        .filter(|p| holds(p.file.as_str()))
        .map(|p| p.name.clone())
        .collect();
    if let Some(list) = list {
        found.extend(
            list.permalinks
                .iter()
                .filter(|(_, e)| holds(&file_of(&e.place)))
                .map(|(name, _)| name.clone()),
        );
    }
    if found.is_empty() {
        return Err(refuse(
            shown,
            "it declares no permalinks, and the list has none there".to_owned(),
        ));
    }
    for name in found {
        if out.contains(&name) || retired(&name) {
            continue;
        }
        already(&name)?;
        if let Some(at) = declared
            .iter()
            .find(|p| p.name == name && !holds(p.file.as_str()))
        {
            return Err(refuse(
                &name,
                format!(
                    "`{}` declares it too, and it isn't being retired",
                    at.place()
                ),
            ));
        }
        out.push(name);
    }
    Ok(out)
}

/// The version of [`RetireAnswer`]'s schema: 2 since a name retired with
/// `to` is moved, and has no `reason` (decision #944a).
pub const PERMALINKS_RETIRE_SCHEMA_VERSION: u32 = 2;

/// What `ascribe permalinks retire` did.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RetireAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The names retired, each written in `ascribe.addresses` as
    /// `go:<name>`: in `[retired]`, or, with `to`, in `[moved]`, in order.
    pub retired: Vec<String>,
    /// The reason each was given; none when each leads to `to`.
    pub reason: Option<String>,
    /// Where each address leads now, as written in `ascribe.addresses`;
    /// none when it shows the reason.
    pub to: Option<String>,
    /// Whether `ascribe.addresses` was made, since the project had none.
    pub created: bool,
    /// Whether `ascribe.permalinks` was changed to record them. Only a name
    /// the content no longer declares is recorded as retired.
    pub list_changed: bool,
    /// Whether the files were written: false with `--dry-run`, which says
    /// what would change.
    pub written: bool,
}

impl RetireAnswer {
    /// The answer for these names.
    pub fn new(
        retired: Vec<String>,
        reason: Option<&str>,
        to: Option<&str>,
        created: bool,
        list_changed: bool,
        written: bool,
    ) -> Self {
        RetireAnswer {
            schema_version: PERMALINKS_RETIRE_SCHEMA_VERSION,
            ascribe_version: ASCRIBE_VERSION,
            retired,
            reason: reason.map(str::to_owned),
            to: to.map(str::to_owned),
            created,
            list_changed,
            written,
        }
    }
}

/// `list`'s text with each of `names` that `project` no longer declares
/// recorded as retired for `reason`, or as moved to `to`, and nothing else
/// changed; `None` when that changes nothing.
pub fn record_retired(
    project: &Project,
    list: &PermalinkList,
    names: &[String],
    reason: &str,
    to: Option<&str>,
) -> Option<String> {
    let declared = DeclaredPermalink::places(&project.declared_permalinks());
    let mut after = list.clone();
    let span = ascribe_core::Span::empty(0);
    for name in names.iter().filter(|n| !declared.contains_key(*n)) {
        after.permalinks.remove(name);
        after.aliases.remove(name);
        after.moved.remove(name);
        after.retired.remove(name);
        match to {
            Some(place) => {
                after.moved.insert(
                    name.clone(),
                    ascribe_model::ListedMoved {
                        place: place.to_owned(),
                        span,
                    },
                );
            }
            None => {
                after.retired.insert(
                    name.clone(),
                    ascribe_model::ListedRetired {
                        reason: reason.to_owned(),
                        span,
                    },
                );
            }
        }
    }
    let text = after.to_toml();
    (text != list.to_toml()).then_some(text)
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
