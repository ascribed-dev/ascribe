//! `ascribe search`: where the pages say something, as their readers read
//! it, and the line to edit for each.

use std::io::{self, Write};
use std::process::ExitCode;

use ascribe_query::{SearchPage, SearchPlace, Searched};
use clap::Args as ClapArgs;

use crate::answer::{self, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::exit;
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
    /// Case is ignored, and a word matches inside a longer one.
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// More words, which the query takes only quoted together with it.
    #[arg(hide = true, value_name = "MORE")]
    pub more: Vec<String>,

    /// Search the pages of this build; repeat it for several.
    ///
    /// By default, every build.
    #[arg(long = "build", value_name = "NAME")]
    pub builds: Vec<String>,

    /// List at most this many places.
    ///
    /// By default, 50.
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
    if !args.more.is_empty() {
        let words = std::iter::once(&args.query)
            .chain(&args.more)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        let _ = writeln!(
            err,
            "error: search takes one query; quote several words together: ascribe search {}",
            quote(&words)
        );
        return exit::FAILURE;
    }
    let (searched, next) = match answer(&FromDisk, global, args) {
        Ok(answer) => answer,
        Err(stop) => return stop.report(err),
    };
    if let Some(code) = answer::written(write_text(out, &searched, next.as_deref()), err) {
        return code;
    }
    if searched.total > 0 {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

/// The places the command shows, at most as many as `--limit` allows, and
/// the command that shows them all when they're cut.
///
/// # Errors
///
/// No project, a query that can't be read, or a build that isn't one.
pub fn answer(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
) -> Result<(Searched, Option<String>), Stop> {
    let loaded = answer::load(projects, global, None)?;
    let limit = args.limit.unwrap_or(TEXT_LIMIT);
    let searched = ascribe_query::search(
        &loaded.project,
        loaded.index(),
        &args.query,
        &args.builds,
        limit,
    )?;
    let next = (searched.places.len() < searched.total).then(|| {
        // The same command, with the limit that lists them all, each word
        // quoted as a shell needs it.
        let mut words = vec![
            "ascribe".to_owned(),
            "search".to_owned(),
            quote(&args.query),
        ];
        for build in &args.builds {
            words.extend(["--build".to_owned(), quote(build)]);
        }
        words.extend(["--limit".to_owned(), searched.total.to_string()]);
        if let Some(config) = &global.config {
            words.extend(["--config".to_owned(), quote(&config.to_string_lossy())]);
        }
        words.join(" ")
    });
    Ok((searched, next))
}

/// Writes the places as text: each `file:line:` and its block as a reader
/// reads it, cut around the match, then where a reader meets it.
pub fn write_text(out: &mut dyn Write, s: &Searched, next: Option<&str>) -> io::Result<()> {
    if s.total == 0 {
        return writeln!(out, "No place matches.");
    }
    for place in &s.places {
        writeln!(out, "{}:{}: {}", place.file, place.line, excerpt(place))?;
        let about = about(place);
        if !about.is_empty() {
            writeln!(out, "    {}", about.join(" · "))?;
        }
    }
    writeln!(out)?;
    let places = if s.total == 1 {
        "place matches"
    } else {
        "places match"
    };
    let pages = if s.pages == 1 { "page" } else { "pages" };
    writeln!(out, "{} {places}, on {} {pages}.", s.total, s.pages)?;
    if let Some(next) = next {
        writeln!(out, "Showing {} of them; for all: `{next}`", s.places.len())?;
    }
    Ok(())
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
