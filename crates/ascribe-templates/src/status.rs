//! What's changed in the templates since a project's copy: `ascribe
//! templates status`.
//!
//! It compares three things: the set the binary carries ([`Upstream`]),
//! what `templates.lock` remembers of each file as eject wrote it, and the
//! project's files as they are. The lock's hash of a file is what tells
//! "this changed upstream" (the set's text isn't what was written) from
//! "the project changed it" (the file isn't what was written). Without an
//! entry, it says what it can from the files alone: whether each is the
//! set's text. It reads the project through its [`FileSystem`] and nothing
//! else: no network, no git.

use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_model::{ContentModel, file_hash};
use ascribe_resolve::{FileSystem, Probe};
use serde::Serialize;
use toml_edit::Item;

use crate::eject::{ASCRIBE_VERSION, LOCK_FILE, TEMPLATES_DIR, parse_lock};
use crate::{FILES, NOTES, Set, set};

/// The version of [`StatusAnswer`]'s schema.
pub const STATUS_SCHEMA_VERSION: u32 = 1;

/// A set of templates as a binary carries it: its list, and the text of each
/// of its files and of Ascribe's notes, by name. [`Upstream::carried`] is
/// this binary's; a test can make a newer one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Upstream {
    /// The set, as `set.toml` lists it.
    pub set: Set,
    /// Each file's text, by its name in `set/` or `notes/`.
    pub texts: BTreeMap<String, String>,
}

impl Upstream {
    /// The set this binary carries.
    pub fn carried() -> Upstream {
        Upstream {
            set: set(),
            texts: FILES
                .iter()
                .chain(NOTES)
                .map(|f| (f.name.to_owned(), f.text.to_owned()))
                .collect(),
        }
    }

    /// The text the set has for one of a template's files, when it has the
    /// template.
    fn text(&self, name: &str, kind: FileKind) -> Option<&str> {
        let template = self.set.templates.iter().find(|t| t.name == name)?;
        let file = match kind {
            FileKind::Template => template.template.clone(),
            FileKind::Guide => template.guide.clone(),
            FileKind::Notes => format!("{name}.ascribe.md"),
        };
        self.texts.get(&file).map(String::as_str)
    }
}

/// What `ascribe templates status` found: for each template the project
/// took, whether the set and the project have changed it since the copy, and
/// each template of the set it doesn't have.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct StatusAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The set this `ascribe` carries, which the copies are compared with.
    pub set: SetRelease,
    /// Whether the set has changed a template the project took since the
    /// copy, as `templates.lock` remembers it, or no longer has one. What
    /// `--exit-code` exits with 1 for.
    pub upstream_changed: bool,
    /// Each template the project took, in the set's order, then any the set
    /// no longer has, by name.
    pub templates: Vec<TemplateStatus>,
    /// Each template of the set the project doesn't have, in the set's
    /// order.
    pub not_taken: Vec<NotTaken>,
}

/// Which release of theirs a set was converted from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SetRelease {
    /// Their repository.
    pub repository: String,
    /// Their release's tag.
    pub tag: String,
    /// The tag's commit.
    pub commit: String,
}

/// A template the project took: its lock entry, its type, or both.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct TemplateStatus {
    /// Its name, which is its type's.
    pub name: String,
    /// What `templates.lock` remembers of it.
    pub record: Record,
    /// The lock its entry is in, or the one looked in when there's no entry,
    /// from the folder `ascribe.toml` is in.
    pub lock: String,
    /// Why the lock can't be read, when `record` is `invalid_lock`; otherwise
    /// null.
    pub lock_problem: Option<String>,
    /// Their release the copy was made from, as the lock records it; null
    /// without an entry.
    pub copied_from: Option<String>,
    /// Whether the set has changed it since the copy: `changed` when any of
    /// its files has.
    pub upstream: UpstreamChange,
    /// Whether the project has changed its copy: `changed` when any of its
    /// files has, or is gone.
    pub project: Change,
    /// Its files: the template, their guide, and Ascribe's notes.
    pub files: Vec<FileStatus>,
}

/// What `templates.lock` remembers of a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Record {
    /// An entry, with a hash of each file as written.
    Entry,
    /// There's no `templates.lock` where it was looked for.
    NoLock,
    /// The lock is there, and has no entry for it.
    NoEntry,
    /// The lock is there, and isn't one eject wrote.
    InvalidLock,
}

/// Whether the set has changed a template, or a file of one, since the copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum UpstreamChange {
    /// The set's text is what was written.
    Unchanged,
    /// The set's text isn't what was written.
    Changed,
    /// Nothing is remembered of what was written, so it can't be told.
    Unknown,
    /// The set no longer has it.
    Removed,
}

impl UpstreamChange {
    /// Whether it's a change in the set since the copy: `changed`, or
    /// `removed`.
    pub fn is_change(self) -> bool {
        matches!(self, UpstreamChange::Changed | UpstreamChange::Removed)
    }
}

/// Whether the project has changed a template, or a file of one, since the
/// copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// The file is what was written.
    Unchanged,
    /// The file isn't what was written.
    Changed,
    /// The file isn't there.
    Deleted,
    /// Nothing is remembered of what was written, so it can't be told.
    Unknown,
}

/// A file of a template the project took.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct FileStatus {
    /// The file, from the folder `ascribe.toml` is in.
    pub path: String,
    /// Which of the template's files it is.
    pub kind: FileKind,
    /// Whether the set's text has changed since the copy.
    pub upstream: UpstreamChange,
    /// Whether the project has changed the file since the copy.
    pub project: Change,
    /// Whether the file is the set's text now; null when it isn't there, or
    /// the set doesn't have it.
    pub matches_set: Option<bool>,
    /// What's different in the set by section, when the set changed it and
    /// the project didn't, so the file is what was written; otherwise null.
    pub sections: Option<Vec<SectionChange>>,
}

/// Which of a template's files a file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    /// The template, which a page is copied from.
    Template,
    /// Their guide.
    Guide,
    /// Ascribe's notes on the type.
    Notes,
}

/// A section that's different in the set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SectionChange {
    /// The section's heading, as written; null for what comes before the
    /// first heading, frontmatter included.
    pub heading: Option<String>,
    /// How it's different.
    pub change: SectionChangeKind,
}

/// How a section is different in the set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SectionChangeKind {
    /// The set has it, and the copy doesn't.
    Added,
    /// The copy has it, and the set doesn't.
    Removed,
    /// Both have it, with different text.
    Changed,
}

/// A template of the set the project doesn't have.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct NotTaken {
    /// Its name.
    pub name: String,
    /// Their index's summary of it.
    pub summary: String,
}

/// What status compares.
#[derive(Clone, Debug)]
pub struct Status<'a> {
    /// The project's content model, whose types name the templates' files.
    /// It needn't have been checked against the files: a file it names may
    /// be gone.
    pub model: &'a ContentModel,
    /// The set to compare with: [`Upstream::carried`], but for a test.
    pub upstream: &'a Upstream,
}

/// A lock, as read.
enum Lock {
    Missing,
    Invalid(String),
    Read(Vec<Entry>),
}

/// An entry of a lock.
struct Entry {
    name: String,
    tag: Option<String>,
    /// Each file, its kind, and the hash it was written with.
    files: Vec<(String, FileKind, String)>,
}

/// What's changed in the templates since the project's copy, comparing the
/// project's files, read through `fs`, and its `templates.lock` with
/// `request.upstream`. It reads nothing else.
pub fn status(request: &Status<'_>, fs: &dyn FileSystem) -> StatusAnswer {
    let upstream = request.upstream;
    let locks = read_locks(request.model, fs);
    let mut found: Vec<TemplateStatus> = Vec::new();
    for (path, lock) in &locks {
        let Lock::Read(entries) = lock else { continue };
        for entry in entries {
            if found.iter().any(|t| t.name == entry.name) {
                continue;
            }
            found.push(remembered(entry, path, upstream, fs));
        }
    }
    for t in &request.model.types {
        if found.iter().any(|f| f.name == t.name)
            || !upstream.set.templates.iter().any(|s| s.name == t.name)
        {
            continue;
        }
        let Some(template) = &t.template else {
            continue;
        };
        let files: Vec<(String, FileKind)> =
            std::iter::once((template.path.clone(), FileKind::Template))
                .chain(t.guide.iter().map(|g| {
                    let kind = if g.path.ends_with(".ascribe.md") {
                        FileKind::Notes
                    } else {
                        FileKind::Guide
                    };
                    (g.path.clone(), kind)
                }))
                .collect();
        let lock_path = lock_beside(&template.path);
        let (record, problem) = match locks.get(&lock_path) {
            Some(Lock::Read(_)) => (Record::NoEntry, None),
            Some(Lock::Invalid(reason)) => (Record::InvalidLock, Some(reason.clone())),
            Some(Lock::Missing) | None => (Record::NoLock, None),
        };
        found.push(unremembered(
            &t.name, &files, record, lock_path, problem, upstream, fs,
        ));
    }
    let order = |name: &str| {
        upstream
            .set
            .templates
            .iter()
            .position(|t| t.name == name)
            .unwrap_or(usize::MAX)
    };
    found.sort_by(|a, b| {
        order(&a.name)
            .cmp(&order(&b.name))
            .then_with(|| a.name.cmp(&b.name))
    });
    let not_taken = upstream
        .set
        .templates
        .iter()
        .filter(|t| !found.iter().any(|f| f.name == t.name))
        .map(|t| NotTaken {
            name: t.name.clone(),
            summary: t.summary.clone(),
        })
        .collect();
    StatusAnswer {
        schema_version: STATUS_SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        set: SetRelease {
            repository: upstream.set.repository.clone(),
            tag: upstream.set.tag.clone(),
            commit: upstream.set.commit.clone(),
        },
        upstream_changed: found.iter().any(|t| t.upstream.is_change()),
        templates: found,
        not_taken,
    }
}

/// A file's change in the set, from the project's copy to the set's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    /// The file, from the folder `ascribe.toml` is in.
    pub path: String,
    /// Whether the project changed its copy, so the diff undoes that too.
    pub project: Change,
    /// The difference, as a unified diff, empty when there's none.
    pub diff: String,
}

/// The change in the set to each file of the template `name`, as a diff
/// from the project's file to the set's text: each file the set has changed
/// since the copy, or, with nothing remembered, each that isn't the set's.
/// It's the upstream change itself when the project hasn't changed its copy.
/// `None` when the project hasn't taken the template.
pub fn diff(
    answer: &StatusAnswer,
    name: &str,
    upstream: &Upstream,
    fs: &dyn FileSystem,
) -> Option<Vec<FileDiff>> {
    let template = answer.templates.iter().find(|t| t.name == name)?;
    let mut out = Vec::new();
    for file in &template.files {
        let changed = match file.upstream {
            UpstreamChange::Changed => true,
            UpstreamChange::Unknown => file.matches_set != Some(true),
            UpstreamChange::Unchanged | UpstreamChange::Removed => false,
        };
        if !changed {
            continue;
        }
        let Some(new) = upstream.text(name, file.kind) else {
            continue;
        };
        let old = RelPath::parse(&file.path)
            .ok()
            .and_then(|p| read_text(&p, fs))
            .unwrap_or_default();
        let old = old.replace("\r\n", "\n");
        let text = similar::TextDiff::from_lines(old.as_str(), new)
            .unified_diff()
            .header(&format!("a/{}", file.path), &format!("b/{}", file.path))
            .to_string();
        out.push(FileDiff {
            path: file.path.clone(),
            project: file.project,
            diff: text,
        });
    }
    Some(out)
}

/// Every lock a template could be beside: in `templates/`, and in the
/// folder of each type's template, by path.
fn read_locks(model: &ContentModel, fs: &dyn FileSystem) -> BTreeMap<String, Lock> {
    let mut paths = vec![format!("{TEMPLATES_DIR}/{LOCK_FILE}")];
    for t in &model.types {
        if let Some(template) = &t.template {
            paths.push(lock_beside(&template.path));
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let lock = RelPath::parse(&path).map_or(Lock::Missing, |p| read_lock(&p, fs));
            (path, lock)
        })
        .collect()
}

/// The lock beside a file, from the folder `ascribe.toml` is in.
fn lock_beside(file: &str) -> String {
    RelPath::parse(file)
        .ok()
        .and_then(|p| p.parent())
        .and_then(|d| d.join(LOCK_FILE).ok())
        .map_or_else(|| LOCK_FILE.to_owned(), |p| p.to_string())
}

fn read_lock(path: &RelPath, fs: &dyn FileSystem) -> Lock {
    if fs.probe(path) != Probe::File {
        return Lock::Missing;
    }
    let doc = match fs.read_file(path) {
        Ok(bytes) => parse_lock(bytes),
        Err(e) => Err(e.to_string()),
    };
    let doc = match doc {
        Ok(doc) => doc,
        Err(reason) => return Lock::Invalid(reason),
    };
    let mut entries = Vec::new();
    for table in doc
        .get("template")
        .and_then(Item::as_array_of_tables)
        .into_iter()
        .flat_map(|t| t.iter())
    {
        let Some(name) = table.get("name").and_then(Item::as_str) else {
            continue;
        };
        let mut files = Vec::new();
        for key in ["files", "ascribe-files"] {
            let Some(listed) = table.get(key).and_then(Item::as_table_like) else {
                continue;
            };
            for (path, hash) in listed.iter() {
                let Some(hash) = hash.as_str() else { continue };
                let kind = if key == "ascribe-files" {
                    FileKind::Notes
                } else if path.ends_with(".guide.md") {
                    FileKind::Guide
                } else {
                    FileKind::Template
                };
                files.push((path.to_owned(), kind, hash.to_owned()));
            }
        }
        entries.push(Entry {
            name: name.to_owned(),
            tag: table.get("tag").and_then(Item::as_str).map(str::to_owned),
            files,
        });
    }
    Lock::Read(entries)
}

/// A template with a lock entry.
fn remembered(
    entry: &Entry,
    lock: &str,
    upstream: &Upstream,
    fs: &dyn FileSystem,
) -> TemplateStatus {
    let in_set = upstream.set.templates.iter().any(|t| t.name == entry.name);
    let files: Vec<FileStatus> = entry
        .files
        .iter()
        .map(|(path, kind, written)| {
            let theirs = upstream.text(&entry.name, *kind);
            let on_disk = RelPath::parse(path).ok().and_then(|p| read_text(&p, fs));
            let upstream_state = match theirs {
                None if in_set => UpstreamChange::Changed,
                None => UpstreamChange::Removed,
                Some(text) if &file_hash(text.as_bytes()) == written => UpstreamChange::Unchanged,
                Some(_) => UpstreamChange::Changed,
            };
            let project = match &on_disk {
                None => Change::Deleted,
                Some(text) if &file_hash(text.as_bytes()) == written => Change::Unchanged,
                Some(_) => Change::Changed,
            };
            let sections = match (upstream_state, project, theirs, &on_disk) {
                (UpstreamChange::Changed, Change::Unchanged, Some(new), Some(old)) => {
                    Some(sections(old, new))
                }
                _ => None,
            };
            FileStatus {
                path: path.clone(),
                kind: *kind,
                upstream: upstream_state,
                project,
                matches_set: matches(on_disk.as_deref(), theirs),
                sections,
            }
        })
        .collect();
    TemplateStatus {
        name: entry.name.clone(),
        record: Record::Entry,
        lock: lock.to_owned(),
        lock_problem: None,
        copied_from: entry.tag.clone(),
        upstream: if !in_set {
            UpstreamChange::Removed
        } else if files.iter().any(|f| f.upstream == UpstreamChange::Changed) {
            UpstreamChange::Changed
        } else {
            UpstreamChange::Unchanged
        },
        project: if files.iter().all(|f| f.project == Change::Unchanged) {
            Change::Unchanged
        } else {
            Change::Changed
        },
        files,
    }
}

/// A template with no lock entry, by its type's files.
fn unremembered(
    name: &str,
    files: &[(String, FileKind)],
    record: Record,
    lock: String,
    lock_problem: Option<String>,
    upstream: &Upstream,
    fs: &dyn FileSystem,
) -> TemplateStatus {
    let files: Vec<FileStatus> = files
        .iter()
        .map(|(path, kind)| {
            let on_disk = RelPath::parse(path).ok().and_then(|p| read_text(&p, fs));
            FileStatus {
                path: path.clone(),
                kind: *kind,
                upstream: UpstreamChange::Unknown,
                project: if on_disk.is_none() {
                    Change::Deleted
                } else {
                    Change::Unknown
                },
                matches_set: matches(on_disk.as_deref(), upstream.text(name, *kind)),
                sections: None,
            }
        })
        .collect();
    TemplateStatus {
        name: name.to_owned(),
        record,
        lock,
        lock_problem,
        copied_from: None,
        upstream: UpstreamChange::Unknown,
        project: if files.iter().any(|f| f.project == Change::Deleted) {
            Change::Changed
        } else {
            Change::Unknown
        },
        files,
    }
}

/// A project's file as text, with its line endings as eject wrote them, so
/// a checkout that turned them into CRLF isn't a change; `None` when it
/// isn't there or can't be read as text.
fn read_text(path: &RelPath, fs: &dyn FileSystem) -> Option<String> {
    if fs.probe(path) != Probe::File {
        return None;
    }
    let text = String::from_utf8(fs.read_file(path).ok()?).ok()?;
    Some(text.replace("\r\n", "\n"))
}

fn matches(on_disk: Option<&str>, theirs: Option<&str>) -> Option<bool> {
    Some(on_disk? == theirs?)
}

/// The sections that are different between `old` and `new`: each section of
/// `new` that `old` hasn't, or has with other text, in `new`'s order, then
/// each of `old`'s that `new` hasn't.
fn sections(old: &str, new: &str) -> Vec<SectionChange> {
    let old = split(old);
    let new = split(new);
    let mut out = Vec::new();
    for (key, text) in &new {
        match old.iter().find(|(k, _)| k == key) {
            None => out.push(SectionChange {
                heading: key.0.clone(),
                change: SectionChangeKind::Added,
            }),
            Some((_, was)) if was != text => out.push(SectionChange {
                heading: key.0.clone(),
                change: SectionChangeKind::Changed,
            }),
            Some(_) => {}
        }
    }
    for (key, _) in &old {
        if !new.iter().any(|(k, _)| k == key) {
            out.push(SectionChange {
                heading: key.0.clone(),
                change: SectionChangeKind::Removed,
            });
        }
    }
    out
}

/// A section's key: its heading, and how many sections before it have the
/// same heading, so a heading written twice is two sections.
type SectionKey = (Option<String>, usize);

/// A page's sections, by its ATX headings outside fenced code: what comes
/// before the first heading, then each heading and what follows it.
fn split(text: &str) -> Vec<(SectionKey, String)> {
    let mut out: Vec<(SectionKey, String)> = vec![((None, 0), String::new())];
    let mut fence: Option<(char, usize)> = None;
    let mut in_frontmatter = text.starts_with("---\n");
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if in_frontmatter {
            if i > 0 && line.trim_end() == "---" {
                in_frontmatter = false;
            }
        } else if let Some((ch, len)) = fence {
            let run = trimmed.chars().take_while(|c| *c == ch).count();
            if run >= len && trimmed[run * ch.len_utf8()..].trim().is_empty() {
                fence = None;
            }
        } else if let Some(ch) = trimmed.chars().next().filter(|c| *c == '`' || *c == '~') {
            let run = trimmed.chars().take_while(|c| *c == ch).count();
            if run >= 3 {
                fence = Some((ch, run));
            }
        } else if let Some(heading) = heading(line) {
            let count = out
                .iter()
                .filter(|((h, _), _)| h.as_deref() == Some(heading))
                .count();
            out.push(((Some(heading.to_owned()), count), String::new()));
            continue;
        }
        if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

/// An ATX heading's text, when `line` is one.
fn heading(line: &str) -> Option<&str> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let level = rest.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &rest[level..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim();
    let open = text.trim_end_matches('#');
    Some(if open.is_empty() {
        ""
    } else if open.ends_with([' ', '\t']) {
        open.trim_end()
    } else {
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(text: &str) -> Vec<Option<String>> {
        split(text).into_iter().map(|((h, _), _)| h).collect()
    }

    #[test]
    fn a_page_is_split_at_its_headings_outside_code_and_frontmatter() {
        let text = "---\ntitle: \"# Not\"\n---\n\nIntro.\n\n## Before you start\n\nText.\n\n```sh\n# a comment\n```\n\n## \\{Task name}\n\n### Step ##\n";
        assert_eq!(
            keys(text),
            [
                None,
                Some("Before you start".to_owned()),
                Some("\\{Task name}".to_owned()),
                Some("Step".to_owned()),
            ]
        );
    }

    #[test]
    fn sections_say_what_was_added_removed_and_changed() {
        let old = "Intro.\n\n## One\n\nA.\n\n## Two\n\nB.\n\n## Two\n\nC.\n";
        let new = "Intro.\n\n## One\n\nA, changed.\n\n## Two\n\nB.\n\n## Three\n\nD.\n";
        let got: Vec<(Option<String>, SectionChangeKind)> = sections(old, new)
            .into_iter()
            .map(|s| (s.heading, s.change))
            .collect();
        assert_eq!(
            got,
            [
                (Some("One".to_owned()), SectionChangeKind::Changed),
                (Some("Three".to_owned()), SectionChangeKind::Added),
                (Some("Two".to_owned()), SectionChangeKind::Removed),
            ]
        );
    }
}
