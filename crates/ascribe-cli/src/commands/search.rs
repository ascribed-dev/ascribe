//! `ascribe search`: where the pages say something, as their readers read
//! it, and the line to edit for each.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{Scope, ScopeError};
use ascribe_query::{Nothing, QueryError, Search, SearchPage, SearchPlace, Searched};
use clap::Args as ClapArgs;

use crate::answer::{self, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;
use crate::report::text::TALLY_FILES;
use crate::shell::quote;

/// How many places the text lists, unless `--limit` says otherwise.
const TEXT_LIMIT: usize = 50;
/// How many characters of a block a place shows, around the match.
const WIDTH: usize = 80;
/// How many of the pages that show a fragment's or a snippet's place the
/// text names.
const PAGES: usize = 3;

/// Arguments of `ascribe search`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// What to find: words, which must all be in one block in any order, or
    /// a phrase in double quotes, matched as written. Quote several words
    /// together: `ascribe search "rate limit"`, and a phrase inside them:
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
    /// By default, 50 places, or 20 pages.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
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
    let (searched, next) = match answer(&FromDisk, global, args) {
        Ok(answer) => answer,
        Err(stop) => return stop.report(err),
    };
    let written = if args.summary {
        write_summary(out, &searched, limit(args), next.as_deref())
    } else {
        write_text(out, &searched, next.as_deref())
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

/// The places the command shows, at most as many as `--limit` allows, and
/// the command that shows them all when they're cut; with `--summary`, the
/// pages.
///
/// # Errors
///
/// No project, a path that names no page or fragment of it, a query that
/// can't be read, or a build or a type that isn't one.
pub fn answer(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
) -> Result<(Searched, Option<String>), Stop> {
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
    let limit = limit(args);
    let searched = ascribe_query::search(
        &loaded.project,
        loaded.index(),
        &Search {
            query: &args.query,
            builds: &args.builds,
            scope: scope.as_ref(),
            page_type: args.page_type.as_deref(),
            case: args.case,
            word: args.word,
            // A summary counts every page, and lists none of the places.
            limit: if args.summary { 0 } else { limit },
        },
    )?;
    let (shown, all) = if args.summary {
        (searched.by_page.len().min(limit), searched.by_page.len())
    } else {
        (searched.places.len(), searched.total)
    };
    let next = (shown < all).then(|| {
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
        words.join(" ")
    });
    Ok((searched, next))
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

/// The most places, or with `--summary` pages, the text lists.
fn limit(args: &Args) -> usize {
    args.limit.unwrap_or(if args.summary {
        TALLY_FILES
    } else {
        TEXT_LIMIT
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
pub fn write_text(out: &mut dyn Write, s: &Searched, next: Option<&str>) -> io::Result<()> {
    if let Some(nothing) = &s.nothing {
        return write_nothing(out, nothing);
    }
    for place in &s.places {
        writeln!(out, "{}:{}: {}", place.file, place.line, excerpt(place))?;
        let about = about(place);
        if !about.is_empty() {
            writeln!(out, "    {}", about.join(" · "))?;
        }
    }
    writeln!(out)?;
    write_total(out, s)?;
    if let Some(next) = next {
        writeln!(out, "Showing {} of them; for all: `{next}`", s.places.len())?;
    }
    Ok(())
}

/// Writes the pages that show a place and how many each shows, most first,
/// at most `limit` of them, then the total: what `--summary` shows. `next`
/// is the command that lists them all, when they're cut.
pub fn write_summary(
    out: &mut dyn Write,
    s: &Searched,
    limit: usize,
    next: Option<&str>,
) -> io::Result<()> {
    if let Some(nothing) = &s.nothing {
        return write_nothing(out, nothing);
    }
    let shown = s.by_page.len().min(limit);
    let width = s
        .by_page
        .first()
        .map_or(1, |(_, count)| count.to_string().len());
    writeln!(out, "By page:")?;
    for (page, count) in s.by_page.iter().take(shown) {
        writeln!(out, "  {count:>width$}  {page}")?;
    }
    if let Some(next) = next.filter(|_| shown < s.by_page.len()) {
        let more = s.by_page.len() - shown;
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
    let pages = if s.pages == 1 { "page" } else { "pages" };
    writeln!(out, "{} {places}, on {} {pages}.", s.total, s.pages)
}

/// Writes that no place matches, and what the source says about the words.
fn write_nothing(out: &mut dyn Write, nothing: &Nothing) -> io::Result<()> {
    writeln!(out, "No place matches.")?;
    match nothing {
        Nothing::Unwritten => writeln!(out, "The words aren't in any page."),
        Nothing::Unpublished { written, total } => {
            let at: Vec<String> = written
                .iter()
                .map(|(file, line)| format!("`{file}:{line}`"))
                .collect();
            let mut text = format!(
                "No build searched publishes them. They're written at {}",
                at.first().map_or("", String::as_str)
            );
            let more = total.saturating_sub(1);
            if more > 0 {
                let places = if more == 1 { "place" } else { "places" };
                text.push_str(&format!(" and {more} more {places}"));
            }
            writeln!(out, "{text}.")
        }
        Nothing::OtherBuilds(builds) => {
            let builds: Vec<String> = builds.iter().map(|b| format!("`{b}`")).collect();
            writeln!(
                out,
                "Builds not searched publish them: {}.",
                builds.join(", ")
            )
        }
    }
}

/// Where a reader meets a place: the heading it's under, its arm, what it
/// reads through, and its builds when it isn't in all of them; for a
/// phrase, how many places use it; for a fragment's or a snippet's, the
/// pages that show it.
fn about(place: &SearchPlace) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some((text, id)) = &place.heading {
        parts.push(if place.kind == "heading" {
            format!("#{id}")
        } else {
            format!("{text} #{id}")
        });
    }
    if let Some(arm) = &place.arm {
        parts.push(format!("{arm} arm"));
    }
    for key in &place.phrases {
        if place.kind == "phrase" {
            parts.push(format!("phrase {key}"));
        } else {
            parts.push(format!("through phrase {key}"));
        }
    }
    for title in &place.titles {
        parts.push(format!("through {title}'s title"));
    }
    if let Some(snippet) = &place.snippet {
        parts.push(format!("through {snippet}"));
        if place.first_line {
            parts.push("at the first line it shows".to_owned());
        }
    }
    if let (Some(uses), Some(key)) = (place.uses, place.phrases.first()) {
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
    if !place.pages.is_empty() {
        parts.push(pages(place));
    }
    parts
}

/// The pages that show a place: the first few, each with the heading it's
/// under and its builds when they aren't all the place's, then how many
/// more and the command that lists them.
fn pages(place: &SearchPlace) -> String {
    let count = place.pages.len();
    let noun = if count == 1 { "page" } else { "pages" };
    let named: Vec<String> = place.pages.iter().take(PAGES).map(page).collect();
    let mut text = format!("on {count} {noun}: {}", named.join(", "));
    if count > PAGES {
        text.push_str(&format!(", and {} more", count - PAGES));
        if let Some(target) = &place.refs {
            text.push_str(&format!(": `ascribe refs {}`", quote(target)));
        }
    }
    text
}

fn page(shown: &SearchPage) -> String {
    let mut text = shown.page.clone();
    if let Some((_, id)) = &shown.heading {
        text.push_str(&format!(" #{id}"));
    }
    if !shown.builds.is_empty() {
        let builds = if shown.builds.len() == 1 {
            "build"
        } else {
            "builds"
        };
        text.push_str(&format!(" (in {builds} {})", shown.builds.join(", ")));
    }
    text
}

/// The block's text, cut to about [`WIDTH`] characters around the match at
/// word boundaries, with `…` where it's cut.
fn excerpt(place: &SearchPlace) -> String {
    let chars: Vec<char> = place.text.chars().collect();
    if chars.len() <= WIDTH {
        return place.text.clone();
    }
    let (first, last) = place.matched;
    let mut start = first.saturating_sub(WIDTH / 4);
    if last > start + WIDTH {
        start = first;
    }
    // Start at a word, and end before one that doesn't fit.
    while start > 0 && start < first && chars.get(start - 1).is_some_and(|c| !c.is_whitespace()) {
        start += 1;
    }
    let mut end = (start + WIDTH).min(chars.len()).max(last.min(chars.len()));
    while end < chars.len() && end > last && chars.get(end).is_some_and(|c| !c.is_whitespace()) {
        end -= 1;
    }
    let text: String = chars[start..end].iter().collect();
    let text = text.trim();
    let before = if start > 0 { "…" } else { "" };
    let after = if end < chars.len() { "…" } else { "" };
    format!("{before}{text}{after}")
}
