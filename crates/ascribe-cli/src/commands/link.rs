//! `ascribe link`: whether a link target exists as seen from a page, its
//! title, and the link to write.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ascribe_query::LinkAnswer;
use clap::Args as ClapArgs;

use crate::answer::{self, Format};
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

fn link(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match answer::load(global, Some(&args.from)) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let answered = answer::source_path(&project, &args.from).and_then(|from| {
        ascribe_query::link(&project.index(), project.file_system(), &from, &args.target)
    });
    let link = match answered {
        Ok(link) => link,
        Err(e) => return answer::fail(err, &e),
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
        let text = l.title.as_deref().unwrap_or("");
        if let Some(href) = &l.href {
            writeln!(out, "Write: [{text}]({href})")?;
        }
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
