//! Where a project's pages say something, as their readers read it: each
//! place a line of the source to edit (`ascribe_resolve::reading`), found in
//! every page each build publishes and listed once however many pages or
//! builds show it.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use ascribe_check::prompt::shell_word;
use ascribe_check::{Project, Scope, select_builds};
use ascribe_core::RelPath;
use ascribe_core::{LineIndex, WideEncoding};
use ascribe_model::Build;
use ascribe_resolve::reading::{self, At, Found, In, Nothing, Query, Reading, Through};
use ascribe_resolve::{AstroRouter, Usable};
use serde::Serialize;

use crate::lines::project_path;
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe search` is asked.
#[derive(Clone, Copy, Debug)]
pub struct Search<'a> {
    /// The query, as given.
    pub query: &'a str,
    /// The builds to search; every build when none is named.
    pub builds: &'a [String],
    /// The folders and pages to keep to, from the project root; every page
    /// when `None`.
    pub scope: Option<&'a Scope>,
    /// The content type to keep to.
    pub page_type: Option<&'a str>,
    /// Whether case is matched as written.
    pub case: bool,
    /// Whether a word or a phrase matches only where it isn't inside a
    /// longer word.
    pub word: bool,
    /// Whether to list the pages and how many places each shows, instead
    /// of the places.
    pub summary: bool,
    /// The most places, or with `summary`, pages, to give.
    pub limit: usize,
}

/// What `ascribe search <QUERY>` answers: what `--format json` writes.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Searched {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The query, as given.
    pub query: String,
    /// The builds searched, in the content model's order.
    pub builds: Vec<String>,
    /// The places, at most `shown` of them: titles, then headings, then the
    /// rest; in each, the places written in pages, then those in fragments
    /// and code files, each in path order and in order within a file.
    /// Empty with `--summary`.
    pub places: Vec<SearchPlace>,
    /// With `--summary`, in place of the places: each page that shows a
    /// place and how many it shows, most first, then in path order; at
    /// most `shown` of them. `null` without it.
    pub pages: Option<Vec<PageCount>>,
    /// How many pages show a place.
    pub pages_matched: usize,
    /// How many places match.
    pub total: usize,
    /// How many places are listed, or with `--summary`, pages.
    pub shown: usize,
    /// Whether the list was cut: `shown` is less than `total`, or with
    /// `--summary`, than `pages_matched`.
    pub truncated: bool,
    /// The command that lists them all, when the list was cut.
    pub next_command: Option<String>,
    /// When nothing matched, the first places the source writes the words
    /// as given, at most three, though no build searched publishes them
    /// there; empty when something matched, or when the words aren't
    /// written.
    pub source_places: Vec<SourcePlace>,
    /// When nothing matched, the builds that weren't searched that publish
    /// the words, in the content model's order; empty otherwise.
    pub not_searched_builds: Vec<String>,
    /// When nothing matched, what the source says: `None` when something
    /// did.
    #[serde(skip)]
    pub nothing: Option<Nothing>,
}

/// How many places [`Nothing::Unpublished`] names.
const WRITTEN: usize = 3;

/// How many of the pages that show a place it names.
const PAGES: usize = 3;

/// A place a query matches: a line of the source to edit.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SearchPlace {
    /// The file, from the project root, as `ascribe check` reports it: a
    /// page, a fragment, a snippet's code file, or `ascribe.toml` for a
    /// phrase.
    pub file: String,
    /// The line, from 1.
    pub line: u32,
    /// The column of the match's first character, from 1, in Unicode
    /// characters.
    pub column: u32,
    /// For a snippet's code that can't be lined up with its file, the last
    /// line the snippet shows: the place is its lines, from `line`, and
    /// not the line the match is on. `null` for every other place.
    pub end_line: Option<u32>,
    /// What it is: `title`, `description`, `heading`, `text`, `table`,
    /// `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`.
    #[serde(rename = "in")]
    pub kind: &'static str,
    /// The block's text as a reader reads it, whole; for a phrase, its
    /// value.
    pub text: String,
    /// Where the match is in `text`: its first character and the one after
    /// its last, in Unicode characters.
    #[serde(rename = "match")]
    pub matched: [usize; 2],
    /// What the match reads through that the line doesn't spell, in order.
    pub through: Vec<SearchThrough>,
    /// The arm of a group the block is written in, as its attributes are
    /// written (`deployment=cloud`), or else its title.
    pub arm: Option<String>,
    /// The pages that show it, in path order, at most three.
    pub pages: Vec<SearchPage>,
    /// How many pages show it; for a phrase, how many places in the source
    /// use it.
    pub pages_total: usize,
    /// The `ascribe refs` command that lists where it's used, for a phrase,
    /// and for a fragment's place when `pages` was cut.
    pub refs_command: Option<String>,
    /// Whether it's written where other pages can show it too: in a
    /// fragment, or in a snippet's code file.
    #[serde(skip)]
    pub shared: bool,
    /// The builds it's in, when it isn't in every build searched.
    #[serde(skip)]
    pub builds: Vec<String>,
}

/// What a match reads through that its line doesn't spell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SearchThrough {
    /// A phrase's value, by its key.
    Phrase(String),
    /// The title of a page a link with no text takes, or the text of the
    /// heading it names: `page.md` or `page.md#id`.
    Title(String),
    /// A snippet's code: its address, as written (`code:app.py#main`).
    Code(String),
}

/// A page that shows a place.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SearchPage {
    /// The page's content path.
    pub page: String,
    /// Its title, as a reader reads it in the first build searched that
    /// publishes the page.
    pub title: Option<String>,
    /// The heading the place is under there, as a reader reads it; for a
    /// heading, the heading itself.
    pub heading: Option<String>,
    /// That heading's id.
    pub id: Option<String>,
    /// The builds that show it there, in the content model's order.
    pub builds: Vec<String>,
}

/// A page and how many places it shows: what `--summary` lists.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PageCount {
    /// The page, from the project root.
    pub page: String,
    /// How many places it shows.
    pub places: usize,
}

/// A place the source writes words no build searched publishes.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SourcePlace {
    /// The file, from the project root.
    pub file: String,
    /// The line, from 1.
    pub line: u32,
}

/// Each build's readings of its pages, made when a search first asks for
/// the build and kept, so a second search is the scan alone. A project
/// kept loaded keeps them with it, and drops them with it.
#[derive(Debug, Default)]
pub struct Readings {
    builds: Mutex<HashMap<String, Arc<[Read]>>>,
}

/// A page's reading, with what a search keeps to.
#[derive(Debug)]
struct Read {
    reading: Reading,
    /// The page's content path in the project's index.
    path: Option<RelPath>,
    /// The page's content type.
    page_type: Option<String>,
}

impl Readings {
    /// The readings of every page `build` publishes, made the first time.
    fn of(&self, index: &ascribe_resolve::Project, build: &Build) -> Arc<[Read]> {
        let mut builds = match self.builds.lock() {
            Ok(builds) => builds,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(kept) = builds.get(&build.name) {
            return Arc::clone(kept);
        }
        // An older line's addresses are under its folder, as its output's are.
        let lined = index.for_build(build);
        let lined = lined.as_ref();
        let router = AstroRouter::from_consumer(&lined.model().consumer);
        let resolved = lined.resolve_build(build, &router);
        let read: Arc<[Read]> = resolved
            .pages
            .iter()
            .map(|page| Read {
                reading: Reading::of(lined, page),
                path: index.path_of(page.file).cloned(),
                page_type: lined
                    .file_by_id(page.file)
                    .and_then(|file| file.content_type(lined.model()))
                    .map(|t| t.name.clone()),
            })
            .collect();
        builds.insert(build.name.clone(), Arc::clone(&read));
        read
    }

    /// The builds read so far, by name, in no order.
    pub fn kept(&self) -> Vec<String> {
        match self.builds.lock() {
            Ok(builds) => builds.keys().cloned().collect(),
            Err(poisoned) => poisoned.into_inner().keys().cloned().collect(),
        }
    }
}

/// The places in `project` that `asked` matches, at most as many as it
/// asks for, and when there are none, what the source says. `index` is the
/// project's source index, and `readings` the readings of its builds made
/// so far, which it adds to.
///
/// # Errors
///
/// The query can't be read, a build isn't one of the model's, or a type
/// isn't one of its types.
pub fn search(
    project: &Project,
    index: &ascribe_resolve::Project,
    readings: &Readings,
    asked: &Search<'_>,
) -> Result<Searched, QueryError> {
    let parsed = Query::parse(asked.query)?
        .matching_case(asked.case)
        .whole_words(asked.word);
    let builds = select_builds(project, asked.builds)?;
    if let Some(name) = asked.page_type {
        known_type(project, name)?;
    }
    let read: Vec<Arc<[Read]>> = builds.iter().map(|b| readings.of(index, b)).collect();
    let found = find_in(index, &read, &parsed, asked);
    let nothing = found
        .places
        .is_empty()
        .then(|| nothing(project, index, readings, &builds, &parsed, asked));
    let names: Vec<String> = builds.iter().map(|b| b.name.clone()).collect();
    // Each page's title, as the first build that publishes it reads it.
    let mut titles: HashMap<&RelPath, &str> = HashMap::new();
    for page in read.iter().flat_map(|r| r.iter()).map(|r| &r.reading) {
        if let Some(title) = page.blocks.iter().find(|b| b.kind == In::Title) {
            titles.entry(&page.page).or_insert(&title.text);
        }
    }
    let model_lines = LineIndex::new(project.model_text());
    let limit = if asked.summary { 0 } else { asked.limit };
    let places: Vec<SearchPlace> = found
        .places
        .iter()
        .take(limit)
        .map(|place| place_of(index, &model_lines, &titles, names.len(), place))
        .collect();
    let mut by_page: Vec<PageCount> = Vec::new();
    for shown in found.places.iter().flat_map(|p| &p.shown) {
        let page = project_path(index, &shown.page);
        match by_page.iter_mut().find(|p| p.page == page) {
            Some(count) => count.places += 1,
            None => by_page.push(PageCount { page, places: 1 }),
        }
    }
    by_page.sort_by(|a, b| b.places.cmp(&a.places).then_with(|| a.page.cmp(&b.page)));
    let (shown, all, pages) = if asked.summary {
        let all = by_page.len();
        by_page.truncate(asked.limit);
        (by_page.len(), all, Some(by_page))
    } else {
        (places.len(), found.places.len(), None)
    };
    let (source_places, not_searched_builds) = match &nothing {
        Some(Nothing::Unpublished { written, .. }) => (
            written
                .iter()
                .map(|(file, line)| SourcePlace {
                    file: file.clone(),
                    line: *line,
                })
                .collect(),
            Vec::new(),
        ),
        Some(Nothing::OtherBuilds(builds)) => (Vec::new(), builds.clone()),
        Some(Nothing::Unwritten) | None => (Vec::new(), Vec::new()),
    };
    Ok(Searched {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        query: asked.query.to_owned(),
        builds: names,
        places,
        pages,
        pages_matched: found.pages,
        total: found.places.len(),
        shown,
        truncated: shown < all,
        next_command: None,
        source_places,
        not_searched_builds,
        nothing,
    })
}

/// What a place `find` found is, as the command gives it. `titles` are the
/// pages' titles, and `builds` how many builds were searched.
fn place_of(
    index: &ascribe_resolve::Project,
    model_lines: &LineIndex,
    titles: &HashMap<&RelPath, &str>,
    builds: usize,
    place: &reading::Place,
) -> SearchPlace {
    let (file, line, column) = match &place.at {
        At::Source {
            file, line, column, ..
        } => (
            index
                .path_of(*file)
                .map(|p| project_path(index, p))
                .unwrap_or_default(),
            *line,
            *column,
        ),
        At::Code {
            path, line, column, ..
        } => (path.clone(), *line, *column),
        At::Phrase { span, .. } => {
            let (line, column) = model_lines
                .wide_line_col(WideEncoding::Utf32, span.start())
                .map_or((1, 1), |p| (p.line + 1, p.col + 1));
            (ascribe_check::MODEL_FILE.to_owned(), line, column)
        }
    };
    let chars = |byte: usize| place.text[..byte].chars().count();
    let through = place
        .through
        .iter()
        .map(|through| match through {
            Through::Phrase(key) => SearchThrough::Phrase(key.clone()),
            Through::Title { page, id } => SearchThrough::Title(match id {
                Some(id) => format!("{page}#{id}"),
                None => page.to_string(),
            }),
            Through::Code(address) => SearchThrough::Code(address.clone()),
        })
        .collect();
    let pages = place
        .shown
        .iter()
        .take(PAGES)
        .map(|shown| SearchPage {
            page: shown.page.to_string(),
            title: titles.get(&shown.page).map(|t| (*t).to_owned()),
            heading: shown.under.as_ref().map(|u| u.text.clone()),
            id: shown.under.as_ref().map(|u| u.id.clone()),
            builds: shown.builds.clone(),
        })
        .collect();
    let (pages_total, refs_command) = match &place.at {
        At::Phrase { key, .. } => (
            index.uses(&Usable::Phrase(key.clone())).len(),
            Some(format!(
                "ascribe refs {}",
                shell_word(&format!("phrase:{key}"))
            )),
        ),
        At::Source { file, .. } if place.shared && place.shown.len() > PAGES => (
            place.shown.len(),
            index
                .path_of(*file)
                .map(|path| format!("ascribe refs {}", shell_word(path.as_str()))),
        ),
        At::Source { .. } | At::Code { .. } => (place.shown.len(), None),
    };
    SearchPlace {
        file,
        line,
        column,
        end_line: place.end_line,
        kind: place.kind.as_str(),
        text: place.text.clone(),
        matched: [chars(place.matched.start), chars(place.matched.end)],
        through,
        arm: place.arm.clone(),
        pages,
        pages_total,
        refs_command,
        shared: place.shared && place.kind != In::Phrase,
        builds: if place.builds.len() == builds {
            Vec::new()
        } else {
            place.builds.clone()
        },
    }
}

/// The places `query` matches in the pages of `read`, each build's
/// readings, that `asked` keeps to.
fn find_in(
    index: &ascribe_resolve::Project,
    read: &[Arc<[Read]>],
    query: &Query,
    asked: &Search<'_>,
) -> Found {
    let kept = read.iter().flat_map(|r| r.iter()).filter(|r| {
        asked.scope.is_none_or(|scope| {
            r.path
                .as_ref()
                .is_some_and(|path| scope.contains(&index.layout().project_path(path)))
        }) && asked
            .page_type
            .is_none_or(|name| r.page_type.as_deref() == Some(name))
    });
    reading::find(index, kept.map(|r| &r.reading), query)
}

/// What the source says about words no build searched publishes: the
/// builds not searched that publish them, when there are some, or else
/// where the source writes them.
fn nothing(
    project: &Project,
    index: &ascribe_resolve::Project,
    readings: &Readings,
    searched: &[&Build],
    query: &Query,
    asked: &Search<'_>,
) -> Nothing {
    let others: Vec<&Build> = project
        .model()
        .builds
        .iter()
        .filter(|b| !searched.iter().any(|s| s.name == b.name))
        .collect();
    if !others.is_empty() {
        let read: Vec<Arc<[Read]>> = others.iter().map(|b| readings.of(index, b)).collect();
        let found = find_in(index, &read, query, asked);
        let mut publish: Vec<String> = Vec::new();
        for build in &others {
            if found.places.iter().any(|p| p.builds.contains(&build.name)) {
                publish.push(build.name.clone());
            }
        }
        if !publish.is_empty() {
            return Nothing::OtherBuilds(publish);
        }
    }
    let only = narrowed(index, asked);
    let written = reading::scan(index, project.model_text(), query, only.as_ref());
    if written.is_empty() {
        return Nothing::Unwritten;
    }
    Nothing::Unpublished {
        total: written.len(),
        written: written
            .into_iter()
            .take(WRITTEN)
            .map(|w| (w.file.to_string(), w.line))
            .collect(),
    }
}

/// When `asked` keeps to paths or a type, the content paths of the pages it
/// keeps to and of the fragments they include, at any depth: what the scan
/// of the source reads, so it says nothing about words written elsewhere.
fn narrowed(index: &ascribe_resolve::Project, asked: &Search<'_>) -> Option<HashSet<RelPath>> {
    if asked.scope.is_none() && asked.page_type.is_none() {
        return None;
    }
    let mut kept = HashSet::new();
    let mut next: Vec<&ascribe_resolve::FileIndex> = index
        .pages()
        .filter(|page| {
            asked
                .scope
                .is_none_or(|scope| scope.contains(&index.layout().project_path(&page.path)))
        })
        .filter(|page| {
            asked.page_type.is_none_or(|name| {
                page.content_type(index.model())
                    .is_some_and(|t| t.name == name)
            })
        })
        .collect();
    while let Some(file) = next.pop() {
        if !kept.insert(file.path.clone()) {
            continue;
        }
        next.extend(
            file.includes
                .iter()
                .filter_map(|include| include.target.as_ref())
                .filter_map(|target| index.files().find(|f| &f.path == target)),
        );
    }
    Some(kept)
}

/// Checks that `name` is one of the content model's types.
fn known_type(project: &Project, name: &str) -> Result<(), QueryError> {
    let types: Vec<&str> = project
        .model()
        .types
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    if types.contains(&name) {
        return Ok(());
    }
    Err(QueryError::UnknownType {
        given: name.to_owned(),
        closest: ascribe_model::suggest(name, types.iter().copied()).map(str::to_owned),
        types: types.iter().map(|t| (*t).to_owned()).collect(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::Path;

    use super::*;

    fn quill() -> Project {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
        Project::load(&root.join("ascribe.toml")).expect("Quill loads")
    }

    fn asked<'a>(query: &'a str, builds: &'a [String], limit: usize) -> Search<'a> {
        Search {
            query,
            builds,
            scope: None,
            page_type: None,
            case: false,
            word: false,
            summary: false,
            limit,
        }
    }

    fn quill_search(query: &str, builds: &[&str], limit: usize) -> Result<Searched, QueryError> {
        let project = quill();
        let builds: Vec<String> = builds.iter().map(|b| (*b).to_owned()).collect();
        search(
            &project,
            &project.index(),
            &Readings::default(),
            &asked(query, &builds, limit),
        )
    }

    fn scope(paths: &[&str]) -> Scope {
        Scope::new(paths.iter().map(|p| RelPath::parse(p).unwrap()))
    }

    fn summarized(
        project: &Project,
        index: &ascribe_resolve::Project,
        kept: &Scope,
        query: &str,
    ) -> Searched {
        search(
            project,
            index,
            &Readings::default(),
            &Search {
                scope: Some(kept),
                summary: true,
                ..asked(query, &[], 50)
            },
        )
        .unwrap()
    }

    /// Quill with a second page that includes its prerequisites.
    fn quill_with_second_page() -> Project {
        quill().with_source(
            &RelPath::parse("setup.md").unwrap(),
            "---\ntype: page\ntitle: Set up\n---\n\n## Before\n\n@include: _fragments/prerequisites.md\n"
                .to_owned(),
        )
    }

    fn files(found: &Searched) -> Vec<(&str, u32)> {
        found
            .places
            .iter()
            .map(|p| (p.file.as_str(), p.line))
            .collect()
    }

    #[test]
    fn the_thread_query_names_line_62_through_the_phrase() {
        let found = quill_search("\"sign in to quill cloud\"", &[], 50).unwrap();
        let first = &found.places[0];
        assert_eq!(
            (first.file.as_str(), first.line),
            ("docs/install-agent.md", 62)
        );
        assert_eq!(first.through, [SearchThrough::Phrase("cloud".into())]);
        assert_eq!(first.arm.as_deref(), Some("deployment=cloud"));
        assert!(first.builds.is_empty(), "every build has it");
        assert_eq!(&first.text[..22], "Sign in to Quill Cloud");
        assert_eq!(first.matched, [0, 22]);
    }

    #[test]
    fn a_match_wholly_in_a_value_is_the_phrase_line_with_its_uses() {
        let found = quill_search("\"quill cloud\"", &[], 50).unwrap();
        let phrase = found.places.iter().find(|p| p.kind == "phrase").unwrap();
        assert_eq!(phrase.file, "ascribe.toml");
        assert_eq!(phrase.line, 34);
        assert!(phrase.pages_total > 1);
        assert_eq!(
            phrase.refs_command.as_deref(),
            Some("ascribe refs phrase:cloud")
        );
    }

    #[test]
    fn the_limit_cuts_the_places_but_not_the_total() {
        let found = quill_search("quill", &["cloud"], 3).unwrap();
        assert_eq!(found.places.len(), 3);
        assert!(found.total > 3, "{}", found.total);
        assert_eq!(found.builds, ["cloud"]);
    }

    #[test]
    fn a_fragment_line_is_one_place_with_the_pages_that_show_it() {
        let found = quill_search("docs repository", &[], 50).unwrap();
        let places: Vec<(&str, u32)> = found
            .places
            .iter()
            .map(|p| (p.file.as_str(), p.line))
            .collect();
        assert_eq!(
            places,
            [
                ("docs/install-agent.md", 8),
                ("docs/_fragments/prerequisites.md", 4)
            ]
        );
        let fragment = &found.places[1];
        assert!(fragment.shared);
        assert_eq!(fragment.refs_command, None, "every page is named");
        let pages: Vec<(&str, Option<&str>)> = fragment
            .pages
            .iter()
            .map(|p| (p.page.as_str(), p.id.as_deref()))
            .collect();
        assert_eq!(pages, [("install-agent.md", Some("prerequisites"))]);
        assert_eq!(fragment.pages_total, 1);
        assert!(!found.places[0].shared);
        assert_eq!(found.pages_matched, 1);
    }

    #[test]
    fn a_title_comes_first_at_its_key() {
        let found = quill_search("\"install the quill agent\"", &[], 50).unwrap();
        let first = &found.places[0];
        assert_eq!(
            (first.file.as_str(), first.line, first.kind),
            ("docs/install-agent.md", 3, "title")
        );
    }

    #[test]
    fn a_snippet_match_is_at_the_code_file_line() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tested");
        let project = Project::load(&root.join("ascribe.toml")).expect("the example loads");
        let found = search(
            &project,
            &project.index(),
            &Readings::default(),
            &asked("\"hello, \"", &[], 50),
        )
        .unwrap();
        let code = found
            .places
            .iter()
            .find(|p| p.file.ends_with("greet.ts"))
            .unwrap();
        assert_eq!(
            (code.file.as_str(), code.line),
            ("code/examples/greet.ts", 3)
        );
        assert_eq!(
            code.through,
            [SearchThrough::Code("code:examples/greet.ts#greet".into())]
        );
        assert_eq!(code.end_line, None);
        assert_eq!(code.pages[0].page, "index.md");
    }

    #[test]
    fn nothing_matched_is_an_answer_and_a_bad_query_an_error() {
        assert_eq!(quill_search("zebra", &[], 50).unwrap().total, 0);
        assert!(matches!(
            quill_search("\"open", &[], 50),
            Err(QueryError::BadQuery(_))
        ));
        assert!(matches!(
            quill_search("quill", &["nope"], 50),
            Err(QueryError::UnknownBuild(_))
        ));
    }

    #[test]
    fn a_folder_or_a_page_keeps_the_search_to_its_pages() {
        let project = quill();
        let index = project.index();
        let all = search(
            &project,
            &index,
            &Readings::default(),
            &asked("quill", &[], 500),
        )
        .unwrap();
        let kept = scope(&["docs/keys.md"]);
        let keys = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                scope: Some(&kept),
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        assert!(keys.total > 0 && keys.total < all.total);
        assert!(
            keys.places
                .iter()
                .all(|p| p.file == "docs/keys.md" || p.file == "ascribe.toml"),
            "{:?}",
            files(&keys)
        );
        assert_eq!(keys.pages_matched, 1);
        let folder = scope(&["docs"]);
        let docs = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                scope: Some(&folder),
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        assert_eq!(docs.total, all.total);
    }

    #[test]
    fn a_fragment_counts_only_for_the_pages_in_the_scope_that_show_it() {
        let project = quill_with_second_page();
        let index = project.held_index();
        let everywhere = search(
            &project,
            &index,
            &Readings::default(),
            &asked("\"node.js 20\"", &[], 50),
        )
        .unwrap();
        let pages: Vec<&str> = everywhere.places[0]
            .pages
            .iter()
            .map(|p| p.page.as_str())
            .collect();
        assert_eq!(pages, ["install-agent.md", "setup.md"]);
        let kept = scope(&["docs/setup.md"]);
        let one = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                scope: Some(&kept),
                ..asked("\"node.js 20\"", &[], 50)
            },
        )
        .unwrap();
        assert_eq!(files(&one), [("docs/_fragments/prerequisites.md", 3)]);
        let pages: Vec<&str> = one.places[0]
            .pages
            .iter()
            .map(|p| p.page.as_str())
            .collect();
        assert_eq!(pages, ["setup.md"]);
        assert_eq!(
            summarized(&project, &index, &kept, "\"node.js 20\"").pages,
            Some(vec![PageCount {
                page: "docs/setup.md".to_owned(),
                places: 1
            }])
        );
    }

    #[test]
    fn a_type_keeps_to_its_pages_and_an_unknown_one_names_the_closest() {
        let project = quill();
        let index = project.index();
        let typed = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                page_type: Some("page"),
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        assert_eq!(typed.total, quill_search("quill", &[], 500).unwrap().total);
        let error = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                page_type: Some("pgae"),
                ..asked("quill", &[], 50)
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "no content type is `pgae`; did you mean page?"
        );
        let error = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                page_type: Some("tutorial"),
                ..asked("quill", &[], 50)
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "no content type is `tutorial`; the types are page"
        );
    }

    #[test]
    fn case_and_whole_words_narrow_what_matches() {
        let project = quill();
        let index = project.index();
        let any = search(
            &project,
            &index,
            &Readings::default(),
            &asked("key", &[], 500),
        )
        .unwrap();
        let word = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                word: true,
                ..asked("key", &[], 500)
            },
        )
        .unwrap();
        assert!(word.total < any.total, "`keys` matches only without --word");
        assert!(
            word.places.iter().all(|p| {
                let text: Vec<char> = p.text.chars().collect();
                text.get(p.matched[1]).is_none_or(|c| !c.is_alphanumeric())
            }),
            "every match ends a word"
        );
        let lower = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                case: true,
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        let folded = quill_search("quill", &[], 500).unwrap();
        assert!(lower.total > 0 && lower.total < folded.total);
        assert!(lower.places.iter().all(|p| {
            let text: String = p.text.chars().skip(p.matched[0]).take(5).collect();
            text == "quill"
        }));
    }

    #[test]
    fn pages_are_counted_most_first_then_by_path() {
        let project = quill();
        let index = project.index();
        let found = search(
            &project,
            &index,
            &Readings::default(),
            &Search {
                summary: true,
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        assert!(found.places.is_empty(), "a summary lists no places");
        let pages = found.pages.unwrap();
        assert_eq!(pages.len(), found.pages_matched);
        assert_eq!((found.shown, found.truncated), (pages.len(), false));
        assert!(
            pages.windows(2).all(|w| w[0].places > w[1].places
                || (w[0].places == w[1].places && w[0].page < w[1].page)),
            "{pages:?}"
        );
        assert!(pages.iter().map(|p| p.places).sum::<usize>() >= found.total);
    }

    #[test]
    fn nothing_written_is_in_no_page() {
        let found = quill_search("zebra", &[], 50).unwrap();
        assert_eq!(found.nothing, Some(Nothing::Unwritten));
        assert_eq!(quill_search("quill", &[], 50).unwrap().nothing, None);
    }

    #[test]
    fn words_only_in_a_memo_are_in_no_page() {
        let found = quill_search("revocation", &[], 50).unwrap();
        assert_eq!(found.nothing, Some(Nothing::Unwritten));
    }

    #[test]
    fn words_no_build_publishes_are_written_at_their_lines() {
        // The source spells the phrase's key, not its value.
        let found = quill_search("\"{cloud}\"", &[], 50).unwrap();
        let Some(Nothing::Unpublished { written, total }) = found.nothing else {
            panic!("{:?}", found.nothing);
        };
        assert!(total > 3, "{total}");
        assert_eq!(written.len(), 3);
        assert!(written.windows(2).all(|w| w[0] <= w[1]), "{written:?}");
    }

    #[test]
    fn words_another_build_publishes_name_it() {
        let found = quill_search("\"point the agent at your server\"", &["cloud"], 50).unwrap();
        assert_eq!(found.total, 0);
        assert_eq!(
            found.nothing,
            Some(Nothing::OtherBuilds(vec![
                "site".to_owned(),
                "site@3.4".to_owned()
            ]))
        );
    }

    fn scoped(query: &str, paths: &[&str]) -> Searched {
        let project = quill();
        let kept = scope(paths);
        search(
            &project,
            &project.index(),
            &Readings::default(),
            &Search {
                scope: Some(&kept),
                ..asked(query, &[], 50)
            },
        )
        .unwrap()
    }

    #[test]
    fn a_narrowed_search_says_nothing_of_words_written_outside_it() {
        // Published on other pages, which the paths leave out.
        let found = scoped("webhook", &["docs/install-agent.md"]);
        assert_eq!(found.total, 0);
        assert_eq!(found.nothing, Some(Nothing::Unwritten));
        // Written in the page kept to, and on others, which aren't named.
        let found = scoped("\"{cloud}\"", &["docs/keys.md"]);
        assert_eq!(
            found.nothing,
            Some(Nothing::Unpublished {
                written: vec![("docs/keys.md".to_owned(), 10)],
                total: 1
            })
        );
    }

    #[test]
    fn a_narrowed_search_reads_the_fragments_its_pages_include() {
        let found = scoped("\"{product} can read\"", &["docs/install-agent.md"]);
        assert_eq!(
            found.nothing,
            Some(Nothing::Unpublished {
                written: vec![("docs/_fragments/prerequisites.md".to_owned(), 4)],
                total: 1
            })
        );
        let found = scoped("\"{product} can read\"", &["docs/keys.md"]);
        assert_eq!(found.nothing, Some(Nothing::Unwritten));
    }

    #[test]
    fn the_thread_query_is_the_proposals_document() {
        let found = quill_search("\"sign in to quill cloud\"", &[], 500).unwrap();
        let json = serde_json::to_value(&found).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["query"], "\"sign in to quill cloud\"");
        assert_eq!(
            (&json["total"], &json["shown"], &json["truncated"]),
            (&2.into(), &2.into(), &false.into())
        );
        assert_eq!(json["pages_matched"], 2, "keys.md says it too");
        assert_eq!(json["pages"], serde_json::Value::Null);
        assert_eq!(json["source_places"], serde_json::json!([]));
        let place = &json["places"][0];
        assert_eq!(place["file"], "docs/install-agent.md");
        assert_eq!(place["line"], 62);
        assert_eq!(place["column"], 1);
        assert_eq!(place["end_line"], serde_json::Value::Null);
        assert_eq!(place["in"], "text");
        assert_eq!(place["match"], serde_json::json!([0, 22]));
        assert_eq!(place["through"], serde_json::json!([{ "phrase": "cloud" }]));
        assert_eq!(place["arm"], "deployment=cloud");
        assert_eq!(place["pages_total"], 1);
        assert_eq!(place["refs_command"], serde_json::Value::Null);
        let page = &place["pages"][0];
        assert_eq!(page["page"], "install-agent.md");
        assert_eq!(page["title"], "Install the Quill agent");
        assert_eq!(page["heading"], "Connect to Quill");
        assert_eq!(page["id"], "connect");
        assert_eq!(page["builds"], serde_json::json!(found.builds));
        let keys: Vec<&String> = place.as_object().unwrap().keys().collect();
        assert!(
            !keys.iter().any(|k| *k == "shared" || *k == "builds"),
            "{keys:?}"
        );
    }

    #[test]
    fn nothing_matched_says_where_the_words_are_written() {
        let found = quill_search("\"{cloud}\"", &[], 500).unwrap();
        let json = serde_json::to_value(&found).unwrap();
        assert_eq!(json["source_places"].as_array().unwrap().len(), 3);
        assert_eq!(json["not_searched_builds"], serde_json::json!([]));
        let found = quill_search("\"point the agent at your server\"", &["cloud"], 500).unwrap();
        let json = serde_json::to_value(&found).unwrap();
        assert_eq!(json["source_places"], serde_json::json!([]));
        assert_eq!(json["not_searched_builds"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn a_second_search_reads_no_build_again() {
        let project = quill();
        let index = project.index();
        let readings = Readings::default();
        let cloud = vec!["cloud".to_owned()];
        search(&project, &index, &readings, &asked("quill", &cloud, 50)).unwrap();
        assert_eq!(readings.kept(), ["cloud"]);
        let build = project
            .model()
            .builds
            .iter()
            .find(|b| b.name == "cloud")
            .unwrap();
        let first = readings.of(&index, build);
        search(&project, &index, &readings, &asked("agent", &cloud, 50)).unwrap();
        assert!(Arc::ptr_eq(&first, &readings.of(&index, build)), "kept");
        search(&project, &index, &readings, &asked("agent", &[], 50)).unwrap();
        assert_eq!(readings.kept().len(), project.model().builds.len());
    }
}
