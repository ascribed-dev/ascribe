//! `ascribe refs`: where a page, a heading, a fragment, an image, or a
//! content model entry is used.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ascribe_core::RelPath;
use ascribe_query::{Asked, QueryError, Refs};
use clap::Args as ClapArgs;

use crate::answer::{self, Format, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::exit;
use crate::shell::quote;

/// How many places text output lists, unless `--limit` says otherwise.
const TEXT_LIMIT: usize = 50;
/// How many places JSON lists, unless `--limit` says otherwise.
const JSON_LIMIT: usize = 500;

/// Arguments of `ascribe refs`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// What to find the uses of: a page or fragment (`keys.md`), a heading
    /// (`keys.md#rotate-keys`), an image (`images/keys.png`),
    /// `phrase:<key>`, `feature:<key>`, `term:<id>`, `dimension:<name>`,
    /// `note:<type>`, or `widget:<name>`.
    ///
    /// A path is from the current directory, or from the content root.
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// List at most this many places.
    ///
    /// By default, 50 in text and 500 in JSON.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 when the target exists, whether or not
/// anything uses it, 1 when it doesn't exist, 2 when it couldn't run.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = refs(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn refs(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let refs = match answer(&FromDisk, global, args) {
        Ok(refs) => refs,
        Err(stop) => return stop.report(err),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &refs),
        Format::Text => write_text(out, &refs),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if refs.exists {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

/// The places the command shows, at most as many as `--limit` (or the
/// format's default) allows: what `--format json` writes.
///
/// # Errors
///
/// No project, or a target that can't be read.
pub fn answer(projects: &dyn Projects, global: &Global, args: &Args) -> Result<Refs, Stop> {
    let entry = Asked::entry(&args.target);
    // A path target finds its project from the path; an entry from the
    // current directory. `--config` says otherwise for either.
    let (path_part, _) = args
        .target
        .split_once('#')
        .unwrap_or((args.target.as_str(), ""));
    let near = match entry {
        None if !path_part.is_empty() => Some(Path::new(path_part)),
        _ => None,
    };
    let loaded = answer::load(projects, global, near)?;
    let target = match entry {
        Some(asked) => asked.map(Target::Asked).map_err(QueryError::from),
        None => path_target(&loaded.project, loaded.index(), &args.target),
    }?;
    let limit = args.limit.unwrap_or(match args.format {
        Format::Text => TEXT_LIMIT,
        Format::Json => JSON_LIMIT,
    });
    let index = loaded.index();
    let memos = loaded.project.all_memos();
    let mut refs = match &target {
        Target::Asked(asked) => ascribe_query::refs(index, asked, &args.target, limit, memos),
        Target::Image(image) => ascribe_query::image_refs(index, image, &args.target, limit, memos),
    };
    if refs.truncated {
        // The same command, with the limit that lists them all, each word
        // quoted as a shell needs it.
        let total = refs.total.to_string();
        let mut words = vec!["ascribe".to_owned(), "refs".to_owned(), quote(&args.target)];
        words.extend(["--limit".to_owned(), total]);
        if let Some(config) = &global.config {
            words.extend(["--config".to_owned(), quote(&config.to_string_lossy())]);
        }
        if args.format == Format::Json {
            words.extend(["--format".to_owned(), "json".to_owned()]);
        }
        let next = words.join(" ");
        refs.next_command = Some(next);
    }
    Ok(refs)
}

/// What a target names, once its path is read.
enum Target {
    /// A source file, a heading, or a content model entry.
    Asked(Asked),
    /// An image, by its path from the content root.
    Image(RelPath),
}

/// A path target, `page.md`, `page.md#id`, or an image, with the path read
/// as a page argument is; one that names no file is still a target, which
/// doesn't exist.
///
/// # Errors
///
/// It isn't a path, or it names a file that's neither a source nor an
/// image.
fn path_target(
    project: &ascribe_check::Project,
    index: &ascribe_resolve::Project,
    given: &str,
) -> Result<Target, QueryError> {
    let (path, id) = match given.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (given, None),
    };
    let Ok(source) = answer::source_path(project, Path::new(path)) else {
        let readings = answer::content_paths(project, Path::new(path));
        if id.is_none()
            && let Some(image) = readings
                .iter()
                .find(|content| ascribe_check::image_file(project, index, content))
        {
            return Ok(Target::Image(image.clone()));
        }
        let there = |content: &RelPath| {
            project
                .content_root()
                .join(content.as_str())
                .is_ok_and(|p| project.file_system().probe(&p) == ascribe_resolve::Probe::File)
        };
        // Outside FileSystem: a path outside the content root, such as
        // `ascribe.toml`, is a file as the command line names it, which the
        // project's files don't list.
        if readings.iter().any(there) || Path::new(path).is_file() {
            return Err(QueryError::BadTarget {
                given: given.to_owned(),
                reason: "it's a file, but not a page, a fragment, or an image of the project; \
                    `refs` takes those, a heading (`page.md#id`), or an entry such as \
                    `phrase:product`"
                    .to_owned(),
            });
        }
        return Ok(Target::Asked(Asked::path(given)?));
    };
    let content = match id {
        Some(id) => format!("{source}#{id}"),
        None => source.to_string(),
    };
    Ok(Target::Asked(Asked::path(&content)?))
}

/// Writes the places as text, as the command does without `--format json`.
pub fn write_text(out: &mut dyn Write, r: &Refs) -> io::Result<()> {
    if !r.exists && r.places.is_empty() {
        return writeln!(out, "{} doesn't exist.", r.target);
    }
    for place in &r.places {
        let personal = if place.personal { ", personal" } else { "" };
        writeln!(
            out,
            "{}:{}:{}: {}{personal}",
            place.file, place.line, place.column, place.use_kind
        )?;
    }
    if !r.places.is_empty() {
        writeln!(out)?;
    }
    if !r.exists {
        return writeln!(out, "{} doesn't exist; the memos above name it.", r.target);
    }
    let places = if r.total == 1 {
        "place uses"
    } else {
        "places use"
    };
    writeln!(out, "{} {places} {}.", r.total, r.target)?;
    if let Some(next) = &r.next_command {
        writeln!(out, "Showing {} of them; for all: `{next}`", r.shown)?;
    }
    Ok(())
}
