//! Where a project's pages say something, as their readers read it: each
//! place a line of the source to edit (`ascribe_resolve::reading`), found in
//! every page each build publishes and listed once however many pages or
//! builds show it.

use std::collections::HashSet;

use ascribe_check::{Project, Scope, select_builds};
use ascribe_core::RelPath;
use ascribe_core::{LineIndex, WideEncoding};
use ascribe_model::Build;
use ascribe_resolve::reading::{self, At, Found, In, Query, Reading, Through};
use ascribe_resolve::{AstroRouter, Usable};

use crate::QueryError;
use crate::lines::project_path;

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
    /// The most places to give.
    pub limit: usize,
}

/// What `ascribe search <QUERY>` answers.
#[derive(Debug)]
pub struct Searched {
    /// The query, as given.
    pub query: String,
    /// The builds searched, in the content model's order.
    pub builds: Vec<String>,
    /// The places, at most as many as asked for: titles, then headings,
    /// then the rest, each in path order and in order within a file.
    pub places: Vec<SearchPlace>,
    /// How many places match.
    pub total: usize,
    /// How many pages show a place.
    pub pages: usize,
    /// Each page that shows a place, from the project root, and how many
    /// places it shows: most first, then in path order.
    pub by_page: Vec<(String, usize)>,
    /// When nothing matched, what the source says: `None` when something
    /// did.
    pub nothing: Option<Nothing>,
}

/// What the source says about words nothing a build searched publishes.
#[derive(Debug, PartialEq, Eq)]
pub enum Nothing {
    /// No page, fragment, or `ascribe.toml` has them as written.
    Unwritten,
    /// They're written, but no build searched publishes them there, or a
    /// phrase's value hides them.
    Unpublished {
        /// The first places they're written, at most three: each file, from
        /// the project root, and line.
        written: Vec<(String, u32)>,
        /// How many places they're written.
        total: usize,
    },
    /// Builds that weren't searched publish them, in the content model's
    /// order.
    OtherBuilds(Vec<String>),
}

/// How many places [`Nothing::Unpublished`] names.
const WRITTEN: usize = 3;

/// A place a query matches: a line of the source to edit.
#[derive(Debug)]
pub struct SearchPlace {
    /// The file, from the project root.
    pub file: String,
    /// The line, from 1.
    pub line: u32,
    /// The column of the match's first character, from 1, in Unicode
    /// characters.
    pub column: u32,
    /// What it is: `title`, `description`, `heading`, `text`, `table`,
    /// `code`, or `phrase`, a phrase's value in `ascribe.toml`.
    pub kind: &'static str,
    /// The block's text as a reader reads it; for a phrase, its value.
    pub text: String,
    /// Where the match is in `text`: its first character and the one after
    /// its last, in Unicode characters.
    pub matched: (usize, usize),
    /// The heading the block is under, and its id; for a heading, itself.
    /// `None` for a place other pages show too, which gives each page's in
    /// `pages`.
    pub heading: Option<(String, String)>,
    /// The arm of a group the block is written in.
    pub arm: Option<String>,
    /// The phrases the match reads through, by key.
    pub phrases: Vec<String>,
    /// The pages whose titles, or headings, links with no text borrow and
    /// the match reads through: `page.md` or `page.md#id`.
    pub titles: Vec<String>,
    /// The snippet whose code it's in: its address, as written.
    pub snippet: Option<String>,
    /// For a snippet's code that can't be lined up with its file, true: the
    /// line is the first one the snippet shows, not the match's.
    pub first_line: bool,
    /// The builds it's in, when it isn't in every build searched.
    pub builds: Vec<String>,
    /// For a phrase, how many places in the source use it.
    pub uses: Option<usize>,
    /// For a place in a fragment or a snippet's code, the pages that show
    /// it, in path order.
    pub pages: Vec<SearchPage>,
    /// For a place in a fragment, what `ascribe refs` takes to list the
    /// pages that include it: the fragment's content path.
    pub refs: Option<String>,
}

/// A page that shows a place written in a fragment or a code file.
#[derive(Debug)]
pub struct SearchPage {
    /// The page's content path.
    pub page: String,
    /// The heading the place is under there, and its id.
    pub heading: Option<(String, String)>,
    /// The builds that show it there, when they aren't all of the place's.
    pub builds: Vec<String>,
}

/// The places in `project` that `asked` matches, at most as many as it
/// asks for, and when there are none, what the source says. `index` is the
/// project's source index.
///
/// # Errors
///
/// The query can't be read, a build isn't one of the model's, or a type
/// isn't one of its types.
pub fn search(
    project: &Project,
    index: &ascribe_resolve::Project,
    asked: &Search<'_>,
) -> Result<Searched, QueryError> {
    let parsed = Query::parse(asked.query)?
        .matching_case(asked.case)
        .whole_words(asked.word);
    let builds = select_builds(project, asked.builds)?;
    if let Some(name) = asked.page_type {
        known_type(project, name)?;
    }
    let found = find_in(index, &builds, &parsed, asked);
    let nothing = found
        .places
        .is_empty()
        .then(|| nothing(project, index, &builds, &parsed, asked));
    let limit = asked.limit;
    let query = asked.query;
    let names: Vec<String> = builds.iter().map(|b| b.name.clone()).collect();
    let model_lines = LineIndex::new(project.model_text());
    let places = found
        .places
        .iter()
        .take(limit)
        .map(|place| {
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
            let mut phrases = Vec::new();
            let mut titles = Vec::new();
            let mut snippet = None;
            for through in &place.through {
                match through {
                    Through::Phrase(key) => phrases.push(key.clone()),
                    Through::Title { page, id } => titles.push(match id {
                        Some(id) => format!("{page}#{id}"),
                        None => page.to_string(),
                    }),
                    Through::Code(address) => snippet = Some(address.clone()),
                }
            }
            let uses = match &place.at {
                At::Phrase { key, .. } => Some(index.uses(&Usable::Phrase(key.clone())).len()),
                At::Source { .. } | At::Code { .. } => None,
            };
            let shared = place.shared && place.kind != In::Phrase;
            let pages = if shared {
                place
                    .shown
                    .iter()
                    .map(|shown| SearchPage {
                        page: shown.page.to_string(),
                        heading: shown.under.as_ref().map(|u| (u.text.clone(), u.id.clone())),
                        builds: if shown.builds == place.builds {
                            Vec::new()
                        } else {
                            shown.builds.clone()
                        },
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let refs = match &place.at {
                At::Source { file, .. } if shared => index.path_of(*file).map(ToString::to_string),
                _ => None,
            };
            SearchPlace {
                file,
                line,
                column,
                kind: place.kind.as_str(),
                text: place.text.clone(),
                matched: (chars(place.matched.start), chars(place.matched.end)),
                heading: place
                    .under
                    .as_ref()
                    .filter(|_| place.kind != In::Phrase && !shared)
                    .map(|u| (u.text.clone(), u.id.clone())),
                arm: place.arm.clone(),
                phrases,
                titles,
                snippet,
                first_line: place.first_line,
                builds: if place.builds.len() == names.len() {
                    Vec::new()
                } else {
                    place.builds.clone()
                },
                uses,
                pages,
                refs,
            }
        })
        .collect();
    let mut by_page: Vec<(String, usize)> = Vec::new();
    for shown in found.places.iter().flat_map(|p| &p.shown) {
        let page = project_path(index, &shown.page);
        match by_page.iter_mut().find(|(p, _)| *p == page) {
            Some((_, count)) => *count += 1,
            None => by_page.push((page, 1)),
        }
    }
    by_page.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok(Searched {
        query: query.to_owned(),
        builds: names,
        places,
        total: found.places.len(),
        pages: found.pages,
        by_page,
        nothing,
    })
}

/// The places `query` matches in the pages `asked` keeps to that `builds`
/// publish.
fn find_in(
    index: &ascribe_resolve::Project,
    builds: &[&Build],
    query: &Query,
    asked: &Search<'_>,
) -> Found {
    let mut readings: Vec<Reading> = Vec::new();
    for build in builds {
        // An older line's addresses are under its folder, as its output's are.
        let lined = index.for_build(build);
        let lined = lined.as_ref();
        let router = AstroRouter::from_consumer(&lined.model().consumer);
        let resolved = lined.resolve_build(build, &router);
        readings.extend(
            resolved
                .pages
                .iter()
                .filter(|page| {
                    asked.scope.is_none_or(|scope| {
                        index
                            .path_of(page.file)
                            .is_some_and(|path| scope.contains(&index.layout().project_path(path)))
                    })
                })
                .filter(|page| {
                    asked.page_type.is_none_or(|name| {
                        lined
                            .file_by_id(page.file)
                            .and_then(|file| file.content_type(lined.model()))
                            .is_some_and(|t| t.name == name)
                    })
                })
                .map(|page| Reading::of(lined, page)),
        );
    }
    reading::find(index, &readings, query)
}

/// What the source says about words no build searched publishes: the
/// builds not searched that publish them, when there are some, or else
/// where the source writes them.
fn nothing(
    project: &Project,
    index: &ascribe_resolve::Project,
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
        let found = find_in(index, &others, query, asked);
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
            limit,
        }
    }

    fn quill_search(query: &str, builds: &[&str], limit: usize) -> Result<Searched, QueryError> {
        let project = quill();
        let builds: Vec<String> = builds.iter().map(|b| (*b).to_owned()).collect();
        search(&project, &project.index(), &asked(query, &builds, limit))
    }

    fn scope(paths: &[&str]) -> Scope {
        Scope::new(paths.iter().map(|p| RelPath::parse(p).unwrap()))
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
        assert_eq!(first.phrases, ["cloud"]);
        assert_eq!(first.arm.as_deref(), Some("deployment=cloud"));
        assert!(first.builds.is_empty(), "every build has it");
        assert_eq!(&first.text[..22], "Sign in to Quill Cloud");
        assert_eq!(first.matched, (0, 22));
    }

    #[test]
    fn a_match_wholly_in_a_value_is_the_phrase_line_with_its_uses() {
        let found = quill_search("\"quill cloud\"", &[], 50).unwrap();
        let phrase = found.places.iter().find(|p| p.kind == "phrase").unwrap();
        assert_eq!(phrase.file, "ascribe.toml");
        assert_eq!(phrase.line, 34);
        assert!(phrase.uses.unwrap() > 1);
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
        assert_eq!(fragment.heading, None, "each page gives its own");
        assert_eq!(
            fragment.refs.as_deref(),
            Some("_fragments/prerequisites.md")
        );
        let pages: Vec<(&str, Option<&str>)> = fragment
            .pages
            .iter()
            .map(|p| (p.page.as_str(), p.heading.as_ref().map(|h| h.1.as_str())))
            .collect();
        assert_eq!(pages, [("install-agent.md", Some("prerequisites"))]);
        assert!(found.places[0].pages.is_empty());
        assert_eq!(found.pages, 1);
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
        let found = search(&project, &project.index(), &asked("\"hello, \"", &[], 50)).unwrap();
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
            code.snippet.as_deref(),
            Some("code:examples/greet.ts#greet")
        );
        assert!(!code.first_line);
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
        let all = search(&project, &index, &asked("quill", &[], 500)).unwrap();
        let kept = scope(&["docs/keys.md"]);
        let keys = search(
            &project,
            &index,
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
        assert_eq!(keys.pages, 1);
        let folder = scope(&["docs"]);
        let docs = search(
            &project,
            &index,
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
        let everywhere = search(&project, &index, &asked("\"node.js 20\"", &[], 50)).unwrap();
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
        assert_eq!(one.by_page, [("docs/setup.md".to_owned(), 1)]);
    }

    #[test]
    fn a_type_keeps_to_its_pages_and_an_unknown_one_names_the_closest() {
        let project = quill();
        let index = project.index();
        let typed = search(
            &project,
            &index,
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
        let any = search(&project, &index, &asked("key", &[], 500)).unwrap();
        let word = search(
            &project,
            &index,
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
                text.get(p.matched.1).is_none_or(|c| !c.is_alphanumeric())
            }),
            "every match ends a word"
        );
        let lower = search(
            &project,
            &index,
            &Search {
                case: true,
                ..asked("quill", &[], 500)
            },
        )
        .unwrap();
        let folded = quill_search("quill", &[], 500).unwrap();
        assert!(lower.total > 0 && lower.total < folded.total);
        assert!(lower.places.iter().all(|p| {
            let text: String = p.text.chars().skip(p.matched.0).take(5).collect();
            text == "quill"
        }));
    }

    #[test]
    fn pages_are_counted_most_first_then_by_path() {
        let found = quill_search("quill", &[], 0).unwrap();
        assert!(found.places.is_empty(), "the limit cuts the places");
        assert_eq!(found.by_page.len(), found.pages);
        assert!(
            found
                .by_page
                .windows(2)
                .all(|w| w[0].1 > w[1].1 || (w[0].1 == w[1].1 && w[0].0 < w[1].0)),
            "{:?}",
            found.by_page
        );
        assert!(found.by_page.iter().map(|(_, n)| n).sum::<usize>() >= found.total);
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
                "site@3.4".to_owned(),
                "self-managed-3.3".to_owned()
            ]))
        );
    }

    fn scoped(query: &str, paths: &[&str]) -> Searched {
        let project = quill();
        let kept = scope(paths);
        search(
            &project,
            &project.index(),
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
}
