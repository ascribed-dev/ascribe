//! `ascribe render`: a page as a reader of one build sees it, as plain
//! Markdown, written to standard output.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args as ClapArgs;

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe render`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The page: a path from the current directory, or from the content
    /// root.
    #[arg(value_name = "PAGE")]
    pub page: PathBuf,

    /// The build whose reader to show it as.
    ///
    /// Needed when `ascribe.toml` has more than one build.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// Write the page's frontmatter first, between `---` lines.
    #[arg(long)]
    pub frontmatter: bool,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 with the page, 1 when the build doesn't
/// publish it (the reason goes to standard error), 2 when it couldn't run:
/// no project, a path that isn't a page, or no build named where one is
/// needed.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = render(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn render(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match answer::load(global, Some(&args.page)) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let answered = answer::source_path(&project, &args.page).and_then(|path| {
        let build = answer::one_build(project.model(), args.build.as_deref())?;
        ascribe_query::render(
            &project.index(),
            project.root(),
            &path,
            build,
            args.frontmatter,
        )
    });
    let rendered = match answered {
        Ok(rendered) => rendered,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &rendered),
        Format::Text => out.write_all(rendered.text.as_bytes()),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    match &rendered.not_published {
        Some(reason) => {
            if args.format == Format::Text {
                let _ = writeln!(err, "error: {reason}");
            }
            exit::PROBLEMS
        }
        None => exit::OK,
    }
}
