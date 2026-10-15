//! `ascribe link`: whether a link target exists as seen from a page, its
//! title, and the link to write.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ascribe_query::LinkAnswer;
use clap::Args as ClapArgs;

use crate::answer::{self, Format, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe link`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The link's destination, as the page would write it: `keys.md`,
    /// `keys.md#rotate-keys`, `#install`.
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// The page the link is on: a path from the current directory, or from
    /// the content root.
    #[arg(long, value_name = "PAGE")]
    pub from: PathBuf,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 when the target exists, 1 when it
/// doesn't, 2 when it couldn't run: no project, or a page that isn't one of
/// its pages or fragments.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = link(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

/// The answer the command shows: what `--format json` writes.
///
/// # Errors
///
/// No project, or a page that isn't one of its own.
pub fn answer(projects: &dyn Projects, global: &Global, args: &Args) -> Result<LinkAnswer, Stop> {
    let loaded = answer::load(projects, global, Some(&args.from))?;
    let project = &loaded.project;
    let from = answer::source_path(project, &args.from)?;
    // The page is named back as `check` names a file: from the folder the
    // command was run in, whichever form it was given in.
    let in_project = project
        .content_root()
        .join(from.as_str())
        .map_or_else(|_| from.to_string(), |path| path.to_string());
    let named = answer::from_here(project.root(), &in_project);
    Ok(ascribe_query::link(
        loaded.index(),
        project.file_system(),
        &from,
        &named,
        &args.target,
    )?)
}

fn link(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let link = match answer(&FromDisk, global, args) {
        Ok(link) => link,
        Err(stop) => return stop.report(err),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &link),
        Format::Text => write_text(out, &link),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if link.exists {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

fn write_text(out: &mut dyn Write, l: &LinkAnswer) -> io::Result<()> {
    if l.exists {
        let what = match (&l.kind, &l.path) {
            (ascribe_query::LinkKind::External, _) => {
                "an external URL; it isn't checked".to_owned()
            }
            (ascribe_query::LinkKind::Heading, Some(path)) => format!("a heading on {path}"),
            (ascribe_query::LinkKind::File, Some(path)) => format!("the file {path}"),
            (_, Some(path)) => format!("the page {path}"),
            (_, None) => "it".to_owned(),
        };
        writeln!(out, "{} exists: {what}.", l.target)?;
        let titled = matches!(
            l.kind,
            ascribe_query::LinkKind::Page | ascribe_query::LinkKind::Heading
        );
        match (&l.href, l.title.as_deref()) {
            // Empty link text shows the target's title (SPEC §5.2), so the
            // empty form is as good, and a smaller edit to a link that has
            // none.
            (Some(href), Some(text)) if titled => writeln!(
                out,
                "Write: [{text}]({href}), or []({href}), which shows the same title"
            )?,
            (Some(href), text) => writeln!(out, "Write: [{}]({href})", text.unwrap_or(""))?,
            (None, _) => {}
        }
        write_lines(out, l)?;
        return Ok(());
    }
    writeln!(
        out,
        "{} doesn't work from {}: {}.",
        l.target,
        l.from,
        l.problem.as_deref().unwrap_or("it doesn't exist")
    )?;
    if !l.closest.is_empty() {
        writeln!(out, "Closest:")?;
        for s in &l.closest {
            writeln!(out, "  [{}]({})", s.title.as_deref().unwrap_or(""), s.href)?;
        }
    }
    Ok(())
}

/// Where the link goes in each line of a versioned build (SPEC §5.2), when
/// it goes anywhere but the target in one of them.
fn write_lines(out: &mut dyn Write, l: &LinkAnswer) -> io::Result<()> {
    let path = l.path.as_deref().unwrap_or_default();
    let elsewhere = l
        .lines
        .iter()
        .any(|line| line.page.as_deref() != Some(path) || line.leads_to.is_some());
    if !elsewhere {
        return Ok(());
    }
    writeln!(out, "In each line:")?;
    for line in &l.lines {
        let goes = match (&line.page, &line.leads_to) {
            (None, _) => "nowhere: no live line has it (link-page-dropped)".to_owned(),
            (Some(page), Some(to)) => format!("{page} in {to}, a link out of the line"),
            (Some(page), None) if page == path => page.clone(),
            (Some(page), None) => format!("{page}, which takes its place"),
        };
        writeln!(out, "  {} ({}): {goes}", line.line, line.build)?;
    }
    Ok(())
}
