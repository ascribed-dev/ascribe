//! `ascribe outline`: a page's title, type, and headings, with their ids and
//! lines, so a `page.md#id` link is right the first time.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ascribe_query::Outline;
use clap::Args as ClapArgs;

use crate::answer::{self, Format, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe outline`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The page or fragment: a path from the current directory, or from the
    /// content root.
    #[arg(value_name = "PAGE")]
    pub page: PathBuf,

    /// Show only the headings this build publishes.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 with the outline, 1 when `--build`
/// doesn't publish the page, 2 when it couldn't run: no project, or a path
/// that isn't one of its pages or fragments.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = outline(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

/// The outline the command shows: what `--format json` writes.
///
/// # Errors
///
/// No project, or a page or build that isn't one of its own.
pub fn answer(projects: &dyn Projects, global: &Global, args: &Args) -> Result<Outline, Stop> {
    let loaded = answer::load(projects, global, Some(&args.page))?;
    let project = &loaded.project;
    let path = answer::source_path(project, &args.page)?;
    let build = answer::build(project.model(), args.build.as_deref())?;
    Ok(ascribe_query::outline(loaded.index(), &path, build)?)
}

fn outline(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let outline = match answer(&FromDisk, global, args) {
        Ok(outline) => outline,
        Err(stop) => return stop.report(err),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &outline),
        Format::Text => write_text(out, &outline),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    match &outline.not_published {
        Some(reason) => {
            if args.format == Format::Text {
                let _ = writeln!(err, "error: {reason}");
            }
            exit::PROBLEMS
        }
        None => exit::OK,
    }
}

fn write_text(out: &mut dyn Write, o: &Outline) -> io::Result<()> {
    if o.not_published.is_some() {
        return Ok(());
    }
    writeln!(out, "{}", o.title.as_deref().unwrap_or("(no title)"))?;
    let what = match (&o.content_type, o.fragment) {
        (_, true) => "a fragment".to_owned(),
        (Some(t), false) => format!("type {t}"),
        (None, false) => "no single type".to_owned(),
    };
    let build = o
        .build
        .as_ref()
        .map_or(String::new(), |b| format!(", as build {b} publishes it"));
    writeln!(out, "{}, {what}{build}", o.file)?;
    if o.headings.is_empty() {
        writeln!(out)?;
        return writeln!(out, "No headings a link can name.");
    }
    writeln!(out)?;
    for h in &o.headings {
        let from = h
            .fragment
            .as_ref()
            .map_or(String::new(), |f| format!("  (from {f})"));
        writeln!(
            out,
            "{}:{}: {} {}  #{}{from}",
            h.file,
            h.line,
            "#".repeat(usize::from(h.level)),
            h.text,
            h.id
        )?;
    }
    writeln!(out)?;
    writeln!(out, "Link to a heading as `{}#<id>`.", o.page)
}
