//! `ascribe search`: where the pages say something, as their readers read
//! it, and the line to edit for each.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{Scope, ScopeError};
use ascribe_query::{
    Nothing, QueryError, Search, SearchPage, SearchPlace, SearchThrough, Searched,
};
use clap::Args as ClapArgs;

use crate::answer::{self, Format, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;
use crate::report::text::TALLY_FILES;
use crate::shell::quote;

/// How many places the text lists, unless `--limit` says otherwise.
const TEXT_LIMIT: usize = 50;
/// How many places JSON lists, unless `--limit` says otherwise.
const JSON_LIMIT: usize = 500;
/// How many characters of a block a place shows, around the match.
const WIDTH: usize = 80;

/// Arguments of `ascribe search`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// What to find: words, which must all be in one block in any order,
    /// apart from common words such as `how` and `the`, or a phrase in
    /// double quotes, matched as written. Quote several words together:
    /// `ascribe search "rate limit"`, and a phrase inside them:
    /// `ascribe search '"rate limit"'`.
    ///
    /// Case is ignored unless `--case`, and a word matches inside a longer
    /// one unless `--word`.
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Search only the pages in these folders, or these pages, from the
    /// current folder.
    ///
    /// A fragment's text counts for the pages here that show it. By
    /// default, every page.
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,

    /// Search only the pages of this content type.
    #[arg(long = "type", value_name = "NAME")]
    pub page_type: Option<String>,

    /// Match case as written.
    #[arg(long)]
    pub case: bool,

    /// Match a word or a phrase only where it isn't inside a longer word:
    /// where the characters on either side aren't letters, digits, or `_`.
    #[arg(long)]
    pub word: bool,

    /// Show the pages and how many places each has, most first, instead of
    /// the places.
    #[arg(long)]
    pub summary: bool,

    /// Search the pages of this build; repeat it for several.
    ///
    /// By default, every build.
    #[arg(long = "build", value_name = "NAME")]
    pub builds: Vec<String>,

    /// List at most this many places, or with `--summary`, pages.
    ///
    /// By default, 50 places in text and 500 in JSON, or 20 pages.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 when something matched, 1 when nothing
/// did, 2 when it couldn't run.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = search(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn search(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let searched = match answer(&FromDisk, global, args) {
        Ok(answer) => answer,
        Err(stop) => return stop.report(err),
    };
    let written = match args.format {
        Format::Json => answer::write_json(out, &searched),
        Format::Text if args.summary => write_summary(out, &searched),
        Format::Text => write_text(out, &searched),
    };
    if let Some(code) = answer::written(written, err) {
        return code;
    }
    if searched.total > 0 {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

/// The places the command shows, at most as many as `--limit` (or the
/// format's default) allows, with the command that shows them all when
/// they're cut; with `--summary`, the pages: what `--format json` writes.
///
/// # Errors
///
/// No project, a path that names no page or fragment of it, a query that
/// can't be read, or a build or a type that isn't one.
pub fn answer(projects: &dyn Projects, global: &Global, args: &Args) -> Result<Searched, Stop> {
    let config = answer::locate(global, args.paths.first().map(PathBuf::as_path))
        .map_err(|e| Stop::Load(Failure::Config(e)))?;
    let loaded = projects
        .load(&config)
        .map_err(|e| Stop::Load(Failure::Load(e)))?;
    let scope = if args.paths.is_empty() {
        None
    } else {
        Some(scope(&loaded.project, &config, args)?)
    };
    let mut searched = ascribe_query::search(
        &loaded.project,
        loaded.index(),
        loaded.readings(),
        &Search {
            query: &args.query,
            builds: &args.builds,
            scope: scope.as_ref(),
            page_type: args.page_type.as_deref(),
            case: args.case,
            word: args.word,
            summary: args.summary,
            limit: limit(args),
        },
    )?;
    let all = if args.summary {
        searched.pages_matched
    } else {
        searched.total
    };
    searched.next_command = searched.truncated.then(|| {
        // The same command, with the limit that lists them all, each word
        // quoted as a shell needs it.
        let mut words = vec![
            "ascribe".to_owned(),
            "search".to_owned(),
            quote(&args.query),
        ];
        words.extend(args.paths.iter().map(|p| quote(&p.to_string_lossy())));
        for build in &args.builds {
            words.extend(["--build".to_owned(), quote(build)]);
        }
        if let Some(name) = &args.page_type {
            words.extend(["--type".to_owned(), quote(name)]);
        }
        for (on, flag) in [
            (args.case, "--case"),
            (args.word, "--word"),
            (args.summary, "--summary"),
        ] {
            if on {
                words.push(flag.to_owned());
            }
        }
        words.extend(["--limit".to_owned(), all.to_string()]);
        if let Some(config) = &global.config {
            words.extend(["--config".to_owned(), quote(&config.to_string_lossy())]);
        }
        if args.format == Format::Json {
            words.extend(["--format".to_owned(), "json".to_owned()]);
        }
        words.join(" ")
    });
    Ok(searched)
}

/// The folders and pages the paths after the query name, each of which
/// must have a page or a fragment of the project at it or under it.
fn scope(project: &ascribe_check::Project, config: &Path, args: &Args) -> Result<Scope, Stop> {
    // Words meant for the query, not paths: say to quote them with it.
    let query = args
        .paths
        .iter()
        .all(|p| p.to_str().is_some_and(is_plain_word))
        .then(|| {
            let words: Vec<String> = std::iter::once(args.query.clone())
                .chain(args.paths.iter().map(|p| p.to_string_lossy().into_owned()))
                .collect();
            quote(&words.join(" "))
        });
    let scope = Scope::of_paths(project, config, &args.paths, true).map_err(|e| match e {
        ScopeError::Missing { path } => QueryError::NoSource {
            path: path.display().to_string(),
            missing: true,
            query: query.clone(),
        },
        e => QueryError::Scope(e),
    })?;
    for (given, path) in args.paths.iter().zip(scope.paths()) {
        if Scope::new([path.clone()]).sources(project).is_empty() {
            return Err(QueryError::NoSource {
                path: given.display().to_string(),
                missing: false,
                query,
            }
            .into());
        }
    }
    Ok(scope)
}

/// The most places, or with `--summary` pages, the command lists.
fn limit(args: &Args) -> usize {
    args.limit.unwrap_or(match args.format {
        _ if args.summary => TALLY_FILES,
        Format::Text => TEXT_LIMIT,
        Format::Json => JSON_LIMIT,
    })
}

/// Whether an argument reads as a word, not a path: letters, digits, `-`,
/// and `'`.
fn is_plain_word(arg: &str) -> bool {
    !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '\'')
}

/// Writes the places as text: each `file:line:` and its block as a reader
/// reads it, cut around the match, then where a reader meets it.
pub fn write_text(out: &mut dyn Write, s: &Searched) -> io::Result<()> {
    if let Some(nothing) = &s.nothing {
        return write_nothing(out, nothing, s);
    }
    for place in &s.places {
        writeln!(out, "{}:{}: {}", place.file, place.line, excerpt(place))?;
        let about = about(place, &s.builds);
        if !about.is_empty() {
            writeln!(out, "    {}", about.join(" · "))?;
        }
    }
    writeln!(out)?;
    write_total(out, s)?;
    if let Some(next) = &s.next_command {
        writeln!(out, "Showing {} of them; for all: `{next}`", s.places.len())?;
    }
    Ok(())
}

/// Writes the pages that show a place and how many each shows, most first,
/// then the total, and the command that lists them all when they're cut:
/// what `--summary` shows.
pub fn write_summary(out: &mut dyn Write, s: &Searched) -> io::Result<()> {
    if let Some(nothing) = &s.nothing {
        return write_nothing(out, nothing, s);
    }
    let pages = s.pages.as_deref().unwrap_or_default();
    let width = pages.first().map_or(1, |p| p.places.to_string().len());
    writeln!(out, "By page:")?;
    for page in pages {
        writeln!(out, "  {:>width$}  {}", page.places, page.page)?;
    }
    if let Some(next) = &s.next_command {
        let more = s.pages_matched - pages.len();
        let pages = if more == 1 { "page" } else { "pages" };
        writeln!(out, "  and {more} more {pages}: `{next}`")?;
    }
    writeln!(out)?;
    write_total(out, s)
}

/// `79 places match, on 28 pages.`
fn write_total(out: &mut dyn Write, s: &Searched) -> io::Result<()> {
    let places = if s.total == 1 {
        "place matches"
    } else {
        "places match"
    };
    let pages = if s.pages_matched == 1 {
        "page"
    } else {
        "pages"
    };
    writeln!(out, "{} {places}, on {} {pages}.", s.total, s.pages_matched)?;
    write_stop_words(out, s)
}

/// `A place needn't have the common words how, do, I.`, when the query
/// has some.
fn write_stop_words(out: &mut dyn Write, s: &Searched) -> io::Result<()> {
    if s.stop_words.is_empty() {
        return Ok(());
    }
    writeln!(
        out,
        "A place needn't have the common words {}.",
        s.stop_words.join(", ")
    )
}

/// Writes that no place matches, what the source says about the words,
/// and the places that have some of them: when the source doesn't have
/// them all together either, those places say so in its place.
fn write_nothing(out: &mut dyn Write, nothing: &Nothing, s: &Searched) -> io::Result<()> {
    writeln!(out, "No place matches.")?;
    let Some(first) = s.partial_places.first() else {
        writeln!(out, "{}", nothing.said())?;
        return write_stop_words(out, s);
    };
    if *nothing != Nothing::Unwritten {
        writeln!(out, "{}", nothing.said())?;
    }
    let words = first.has + first.lacks.len();
    let all = if words == 2 {
        "both words".to_owned()
    } else {
        format!("all {words} words")
    };
    let fewer = if s.partial_places.iter().all(|p| p.has == first.has) {
        ""
    } else {
        "at most "
    };
    writeln!(
        out,
        "No block has {all}. These have {fewer}{} of {words}:",
        first.has
    )?;
    for partial in &s.partial_places {
        let place = &partial.place;
        writeln!(out, "{}:{}: {}", place.file, place.line, excerpt(place))?;
        let mut about = about(place, &s.builds);
        about.push(format!("lacks {}", partial.lacks.join(", ")));
        writeln!(out, "    {}", about.join(" · "))?;
    }
    write_stop_words(out, s)
}

/// Where a reader meets a place: the heading it's under, its arm, what it
/// reads through, and its builds when it isn't in all of them; for a
/// phrase, how many places use it; for a fragment's or a snippet's, the
/// pages that show it.
fn about(place: &SearchPlace, searched: &[String]) -> Vec<String> {
    let mut parts = Vec::new();
    let page = place
        .pages
        .first()
        .filter(|_| !place.shared && place.kind != "phrase");
    if let Some((text, id)) = page.and_then(|p| p.heading.as_ref().zip(p.id.as_ref())) {
        parts.push(if place.kind == "heading" {
            format!("#{id}")
        } else {
            format!("{text} #{id}")
        });
    }
    if let Some(arm) = &place.arm {
        parts.push(format!("{arm} arm"));
    }
    let mut phrase = None;
    for through in &place.through {
        if let SearchThrough::Phrase(key) = through {
            phrase.get_or_insert(key);
            if place.kind == "phrase" {
                parts.push(format!("phrase {key}"));
            } else {
                parts.push(format!("through phrase {key}"));
            }
        }
    }
    for through in &place.through {
        if let SearchThrough::Title(title) = through {
            parts.push(format!("through {title}'s title"));
        }
    }
    for through in &place.through {
        if let SearchThrough::Code(address) = through {
            parts.push(format!("through {address}"));
            if place.end_line.is_some() {
                parts.push("at the first line it shows".to_owned());
            }
        }
    }
    if let (true, Some(key)) = (place.kind == "phrase", phrase) {
        let uses = place.pages_total;
        let places = if uses == 1 { "place" } else { "places" };
        parts.push(format!(
            "{uses} {places} use it: `ascribe refs phrase:{key}`"
        ));
    }
    if !place.builds.is_empty() {
        let builds = if place.builds.len() == 1 {
            "build"
        } else {
            "builds"
        };
        parts.push(format!("in {builds} {}", place.builds.join(", ")));
    }
    if place.shared && !place.pages.is_empty() {
        parts.push(pages(place, searched));
    }
    parts
}

/// The pages that show a place: the first few, each with the heading it's
/// under and its builds when they aren't all the place's, then how many
/// more and the command that lists them.
fn pages(place: &SearchPlace, searched: &[String]) -> String {
    let count = place.pages_total;
    let noun = if count == 1 { "page" } else { "pages" };
    let named: Vec<String> = place
        .pages
        .iter()
        .map(|p| page(place, p, searched))
        .collect();
    let mut text = format!("on {count} {noun}: {}", named.join(", "));
    if count > place.pages.len() {
        text.push_str(&format!(", and {} more", count - place.pages.len()));
        if let Some(refs) = &place.refs_command {
            text.push_str(&format!(": `{refs}`"));
        }
    }
    text
}

fn page(place: &SearchPlace, shown: &SearchPage, searched: &[String]) -> String {
    let mut text = shown.page.clone();
    if let Some(id) = &shown.id {
        text.push_str(&format!(" #{id}"));
    }
    // A place in every build searched names none.
    let builds = if place.builds.is_empty() {
        searched
    } else {
        &place.builds
    };
    if shown.builds != builds {
        let builds = if shown.builds.len() == 1 {
            "build"
        } else {
            "builds"
        };
        text.push_str(&format!(" (in {builds} {})", shown.builds.join(", ")));
    }
    text
}

/// The block's text, cut to about [`WIDTH`] characters around the match's
/// first character at word boundaries, with `…` where it's cut.
fn excerpt(place: &SearchPlace) -> String {
    ascribe_query::excerpt(&place.text, place.matched, WIDTH).0
}
