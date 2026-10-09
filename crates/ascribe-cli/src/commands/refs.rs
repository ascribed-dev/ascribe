//! `ascribe refs`: where a page, a heading, a fragment, or a content model
//! entry is used.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ascribe_query::{Asked, QueryError, Refs};
use clap::Args as ClapArgs;

use crate::answer::{self, Format};
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
    /// (`keys.md#rotate-keys`), `phrase:<key>`, `feature:<key>`,
    /// `term:<id>`, `dimension:<name>`, `note:<type>`, or `widget:<name>`.
    ///
    /// A path is from the current directory, or from the content root.
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// List at most this many places.
    ///
    /// By default, 50 in text and 500 in JSON.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,

    /// A file or folder in the project, to find its `ascribe.toml` from, for
    /// a target that isn't a path.
    ///
    /// By default, the current directory.
    #[arg(long, value_name = "PATH")]
    pub project: Option<std::path::PathBuf>,

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
    let entry = Asked::entry(&args.target);
    // A path target finds its project from the path; an entry from
    // `--project`, or the current directory.
    let (path_part, _) = args
        .target
        .split_once('#')
        .unwrap_or((args.target.as_str(), ""));
    let near = match (&entry, &args.project) {
        (_, Some(project)) => Some(project.as_path()),
        (None, None) if !path_part.is_empty() => Some(Path::new(path_part)),
        _ => None,
    };
    let project = match answer::load(global, near) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let asked = match entry {
        Some(asked) => asked,
        None => path_target(&project, &args.target),
    };
    let asked = match asked {
        Ok(asked) => asked,
        Err(e) => return answer::fail(err, &e),
    };
    let limit = args.limit.unwrap_or(match args.format {
        Format::Text => TEXT_LIMIT,
        Format::Json => JSON_LIMIT,
    });
    let mut refs = ascribe_query::refs(&project.index(), &asked, &args.target, limit);
    if refs.truncated {
        // The same command, with the limit that lists them all, each word
        // quoted as a shell needs it.
        let total = refs.total.to_string();
        let mut words = vec!["ascribe".to_owned(), "refs".to_owned(), quote(&args.target)];
        words.extend(["--limit".to_owned(), total]);
        if let Some(project) = &args.project {
            words.extend(["--project".to_owned(), quote(&project.to_string_lossy())]);
        }
        if let Some(config) = &global.config {
            words.extend(["--config".to_owned(), quote(&config.to_string_lossy())]);
        }
        if args.format == Format::Json {
            words.extend(["--format".to_owned(), "json".to_owned()]);
        }
        let next = words.join(" ");
        refs.next_command = Some(next);
    }
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

/// A path target, `page.md` or `page.md#id`, with the path read as a page
/// argument is; one that names no source file is still a target, which
/// doesn't exist.
fn path_target(project: &ascribe_check::Project, given: &str) -> Result<Asked, QueryError> {
    let (path, id) = match given.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (given, None),
    };
    let Ok(source) = answer::source_path(project, Path::new(path)) else {
        return Asked::path(given);
    };
    let content = match id {
        Some(id) => format!("{source}#{id}"),
        None => source.to_string(),
    };
    Asked::path(&content)
}

fn write_text(out: &mut dyn Write, r: &Refs) -> io::Result<()> {
    if !r.exists {
        return writeln!(out, "{} doesn't exist.", r.target);
    }
    for place in &r.places {
        writeln!(
            out,
            "{}:{}:{}: {}",
            place.file, place.line, place.column, place.use_kind
        )?;
    }
    if !r.places.is_empty() {
        writeln!(out)?;
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
