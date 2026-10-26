//! `ascribe resolve`: a page resolved for one build and written as the
//! build's plain or site output writes it, to standard output; or, for `go:<name>`,
//! the permalink's help, as the build's `_ascribe/help.json` has it.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args as ClapArgs, ValueEnum};

use ascribe_query::{PermalinkHelp, Resolved, ResolvedPermalink};

use crate::answer::{self, Format, FromDisk, Projects, Stop};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe resolve`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The page: a path from the current directory, or from the content
    /// root. Or `go:` and a permalink's name, for the help a product shows
    /// for it: its title, summary, and text, as the build's
    /// `_ascribe/help.json` has them, and its address.
    #[arg(value_name = "PAGE")]
    pub page: PathBuf,

    /// The build whose reader to show it as.
    ///
    /// Needed when `ascribe.toml` has more than one build.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// Which output to write the page as, one of `plain` (the default) or
    /// `site`, as `ascribe build --emit` names them. One output, not a list.
    ///
    /// Not for `go:<name>`, whose help has one form.
    #[arg(long, value_enum, value_name = "OUTPUT")]
    pub emit: Option<Emit>,

    /// Write the page's frontmatter first, between `---` lines: under
    /// `--emit site`, the site output's. Nothing for `go:<name>`, which has
    /// no frontmatter.
    #[arg(long)]
    pub frontmatter: bool,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// An output a page can be written as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Emit {
    /// Fully resolved CommonMark with no HTML.
    Plain,
    /// Markdown plus web components, for an Astro site.
    Site,
}

impl Emit {
    fn query(self) -> ascribe_query::Emit {
        match self {
            Emit::Plain => ascribe_query::Emit::Plain,
            Emit::Site => ascribe_query::Emit::Site,
        }
    }
}

/// What the command answers.
#[derive(Debug)]
pub enum Answer {
    /// A page.
    Page(Resolved),
    /// A permalink's help, for `go:<name>`.
    Help {
        /// The name, as asked for.
        name: String,
        /// The build, as named.
        build: String,
        /// What the build has for it.
        help: PermalinkHelp,
    },
}

impl Answer {
    /// Why there's no help to show, as the command reports it after
    /// `error: `: a name the build doesn't publish, or a retired one.
    pub fn no_help(&self) -> Option<String> {
        let Answer::Help { name, build, help } = self else {
            return None;
        };
        match help {
            PermalinkHelp::Found(_) => None,
            PermalinkHelp::Retired { reason } => {
                Some(format!("permalink `{name}` is retired: {reason}"))
            }
            PermalinkHelp::Moved { to } => Some(format!(
                "permalink `{name}` was moved, and has no entry of its own: its address leads to `{to}`"
            )),
            PermalinkHelp::Missing { closest } => {
                let mut text = format!("build `{build}` publishes no permalink `{name}`");
                match closest.as_slice() {
                    [] => text.push_str("; `ascribe permalinks list` lists them"),
                    [one] => text.push_str(&format!("; did you mean `{one}`?")),
                    _ => text.push_str(&format!("; the closest are: {}", closest.join(", "))),
                }
                Some(text)
            }
        }
    }
}

/// Runs the command. Exit codes: 0 with the page or the help, 1 when the
/// build doesn't publish the page or the permalink, or the permalink is
/// retired (the reason goes to standard error), 2 when it couldn't run: no
/// project, a path that isn't a page, no build named where one is needed,
/// or `--emit` with `go:<name>`.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = resolve(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

/// The permalink's name, when the page argument is `go:<name>`.
pub fn permalink_name(page: &std::path::Path) -> Option<&str> {
    page.to_str()?.strip_prefix("go:")
}

/// The page, or the permalink's help, as the command shows it: what
/// `--format json` writes.
///
/// # Errors
///
/// No project, a path that isn't one of its pages, no build named where
/// one is needed, or an output asked for a permalink's help.
pub fn answer(projects: &dyn Projects, global: &Global, args: &Args) -> Result<Answer, Stop> {
    let name = permalink_name(&args.page);
    if name.is_some() && args.emit.is_some() {
        return Err(Stop::Usage(
            "a permalink's help has one form, help.json's; leave out --emit",
        ));
    }
    // A permalink's project is the current directory's, or `--config`'s.
    let loaded = answer::load(projects, global, name.is_none().then_some(&*args.page))?;
    let project = &loaded.project;
    let build = answer::one_build(project.model(), args.build.as_deref())?;
    if let Some(name) = name {
        return Ok(Answer::Help {
            name: name.to_owned(),
            build: build.name.clone(),
            help: ascribe_query::resolve::permalink(loaded.index(), project.root(), name, build),
        });
    }
    let path = answer::source_path(project, &args.page)?;
    Ok(Answer::Page(ascribe_query::resolve(
        loaded.index(),
        project.root(),
        &path,
        build,
        args.emit.map_or_else(Default::default, Emit::query),
        args.frontmatter,
    )?))
}

/// The help as text: the title, the summary when the text doesn't start
/// with it, the text, and "Read more" with the address.
pub fn write_help(out: &mut dyn Write, help: &ResolvedPermalink) -> io::Result<()> {
    let entry = &help.entry;
    writeln!(out, "{}", entry.title)?;
    if let Some(summary) = &entry.summary
        && !entry.text.starts_with(summary.as_str())
    {
        writeln!(out, "{summary}")?;
        if !entry.text.is_empty() {
            writeln!(out)?;
        }
    }
    if !entry.text.is_empty() {
        writeln!(out, "{}", entry.text)?;
    }
    if let Some(address) = &entry.address {
        writeln!(out, "\nRead more: {address}")?;
    }
    Ok(())
}

fn resolve(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let answer = match answer(&FromDisk, global, args) {
        Ok(answer) => answer,
        Err(stop) => return stop.report(err),
    };
    let resolved = match &answer {
        Answer::Page(resolved) => resolved,
        Answer::Help { help, .. } => {
            let PermalinkHelp::Found(help) = help else {
                let _ = writeln!(err, "error: {}", answer.no_help().unwrap_or_default());
                return exit::PROBLEMS;
            };
            let result = match args.format {
                Format::Json => answer::write_json(out, help),
                Format::Text => write_help(out, help),
            };
            return answer::written(result, err).unwrap_or(exit::OK);
        }
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, resolved),
        Format::Text => out.write_all(resolved.text.as_bytes()),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    match &resolved.not_published {
        Some(reason) => {
            if args.format == Format::Text {
                let _ = writeln!(err, "error: {reason}");
            }
            exit::PROBLEMS
        }
        None => exit::OK,
    }
}
