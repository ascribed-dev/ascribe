//! Where a project's pages say something, as their readers read it: each
//! place a line of the source to edit (`ascribe_resolve::reading`), found in
//! every page each build publishes and listed once however many pages or
//! builds show it.

use ascribe_check::{Project, select_builds};
use ascribe_core::{LineIndex, WideEncoding};
use ascribe_resolve::reading::{self, At, In, Query, Reading, Through};
use ascribe_resolve::{AstroRouter, Usable};

use crate::QueryError;
use crate::lines::project_path;

/// What `ascribe search <QUERY>` answers.
#[derive(Debug)]
pub struct Searched {
    /// The query, as given.
    pub query: String,
    /// The builds searched, in the content model's order.
    pub builds: Vec<String>,
    /// The places, at most as many as asked for: headings, then the rest,
    /// each in path order and in order within a file.
    pub places: Vec<SearchPlace>,
    /// How many places match.
    pub total: usize,
    /// How many pages show a place.
    pub pages: usize,
}

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
    /// What it is: `heading`, `text`, `table`, `code`, or `phrase`, a
    /// phrase's value in `ascribe.toml`.
    pub kind: &'static str,
    /// The block's text as a reader reads it; for a phrase, its value.
    pub text: String,
    /// Where the match is in `text`: its first character and the one after
    /// its last, in Unicode characters.
    pub matched: (usize, usize),
    /// The heading the block is under, and its id; for a heading, itself.
    pub heading: Option<(String, String)>,
    /// The arm of a group the block is written in.
    pub arm: Option<String>,
    /// The phrases the match reads through, by key.
    pub phrases: Vec<String>,
    /// The pages whose titles, or headings, links with no text borrow and
    /// the match reads through: `page.md` or `page.md#id`.
    pub titles: Vec<String>,
    /// The builds it's in, when it isn't in every build searched.
    pub builds: Vec<String>,
    /// For a phrase, how many places in the source use it.
    pub uses: Option<usize>,
}

/// The places in `project` that `query` matches in the builds named in
/// `builds` (every build when none is), at most `limit` of them. `index` is
/// the project's source index.
///
/// # Errors
///
/// The query can't be read, or a build isn't one of the model's.
pub fn search(
    project: &Project,
    index: &ascribe_resolve::Project,
    query: &str,
    builds: &[String],
    limit: usize,
) -> Result<Searched, QueryError> {
    let parsed = Query::parse(query)?;
    let builds = select_builds(project, builds)?;
    let mut readings: Vec<Reading> = Vec::new();
    for build in &builds {
        // An older line's addresses are under its folder, as its output's are.
        let lined = index.for_build(build);
        let lined = lined.as_ref();
        let router = AstroRouter::from_consumer(&lined.model().consumer);
        let resolved = lined.resolve_build(build, &router);
        readings.extend(resolved.pages.iter().map(|p| Reading::of(lined, p)));
    }
    let found = reading::find(index, &readings, &parsed);
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
            for through in &place.through {
                match through {
                    Through::Phrase(key) => phrases.push(key.clone()),
                    Through::Title { page, id } => titles.push(match id {
                        Some(id) => format!("{page}#{id}"),
                        None => page.to_string(),
                    }),
                }
            }
            let uses = match &place.at {
                At::Phrase { key, .. } => Some(index.uses(&Usable::Phrase(key.clone())).len()),
                At::Source { .. } => None,
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
                    .filter(|_| place.kind != In::Phrase)
                    .map(|u| (u.text.clone(), u.id.clone())),
                arm: place.arm.clone(),
                phrases,
                titles,
                builds: if place.builds.len() == names.len() {
                    Vec::new()
                } else {
                    place.builds.clone()
                },
                uses,
            }
        })
        .collect();
    Ok(Searched {
        query: query.to_owned(),
        builds: names,
        places,
        total: found.places.len(),
        pages: found.pages,
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

    fn quill_search(query: &str, builds: &[&str], limit: usize) -> Result<Searched, QueryError> {
        let project = quill();
        let builds: Vec<String> = builds.iter().map(|b| (*b).to_owned()).collect();
        search(&project, &project.index(), query, &builds, limit)
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
}
