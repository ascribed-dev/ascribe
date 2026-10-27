//! `ascribe init`: starts a project, writing its first `ascribe.toml` and
//! `ascribe.addresses`, and with `--templates`, the page templates too
//! (`ascribe_templates::init`). In a folder of pages already there, it
//! checks them, and says what to run next.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{MODEL_FILE, Severity};
use ascribe_core::path::normalize;
use ascribe_core::{Applicability, diagnostics};
use ascribe_model::ADDRESSES_FILE;
use ascribe_templates::{InitAnswer, InitError};
use clap::Args as ClapArgs;

use crate::answer::{self, Format, FromDisk, from_here, path_from_here};
use crate::cli::Global;
use crate::commands::check;
use crate::commands::templates::{write_added, write_refused};
use crate::exit;
use crate::report::{Counts, tally, text};

/// Arguments of `ascribe init`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The folder to start the project in, from the current folder. It's
    /// made if it isn't there. By default, the current folder.
    #[arg(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Take the page templates too, as `ascribe templates eject` with no
    /// name does: six types, beside the type `page`.
    #[arg(long)]
    pub templates: bool,

    /// The folder the pages are in, from the project's folder, with `/`
    /// between its parts. By default, `docs`.
    ///
    /// When it holds Markdown already, `ascribe.toml` declares no page types,
    /// so no page needs `type:`, and `init` checks the pages and says what to
    /// run next. `--templates` declares types, so it's refused there.
    #[arg(long, value_name = "DIR")]
    pub content_root: Option<String>,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 when the project is there; 2 when it
/// wrote nothing (there's a project at or above the folder, or a template
/// was refused), or it couldn't run.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = run_init(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn run_init(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    // The project is the one being started, so a `--config` would be
    // ignored; refuse it rather than look as if it were used.
    if global.config.is_some() {
        let _ = writeln!(
            err,
            "error: `ascribe init` doesn't take --config: it starts a project in the current folder, or in the folder DIR names"
        );
        return exit::FAILURE;
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => return answer::fail(err, &ascribe_workspace::LocateError::CurrentDir(e)),
    };
    let root = normalize(&cwd.join(args.dir.as_deref().unwrap_or(Path::new("."))));
    // `--content-root` is written as `ascribe.toml` writes it, with `/`.
    let content_root = args.content_root.as_deref().map(|given| {
        if cfg!(windows) {
            given.replace('\\', "/")
        } else {
            given.to_owned()
        }
    });
    let request = ascribe_templates::Init {
        root: &root,
        templates: args.templates,
        content_root: content_root.as_deref(),
        ascribe_version: env!("ASCRIBE_VERSION"),
    };
    let started = match ascribe_templates::init(&request) {
        Ok(started) => started,
        // The project found, as it's typed from here.
        Err(InitError::Project { found }) => {
            let found = PathBuf::from(path_from_here(&found));
            return answer::fail(err, &InitError::Project { found });
        }
        Err(e) => return answer::fail(err, &e),
    };
    let refused = started
        .templates
        .as_ref()
        .is_some_and(|t| !t.refused.is_empty());
    let result = match args.format {
        Format::Json => answer::write_json(out, &started),
        Format::Text => write_init(out, err, global, &root, &started),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if refused { exit::FAILURE } else { exit::OK }
}

fn write_init(
    out: &mut dyn Write,
    err: &mut dyn Write,
    global: &Global,
    root: &Path,
    started: &InitAnswer,
) -> io::Result<()> {
    if let Some(ejected) = started.templates.as_ref().filter(|t| !t.refused.is_empty()) {
        return write_refused(err, ejected);
    }
    for path in &started.written {
        if path == ADDRESSES_FILE {
            writeln!(
                out,
                "wrote {}, where a page's move is recorded, so the addresses it published keep working",
                from_here(root, path)
            )?;
        } else {
            writeln!(out, "wrote {}", from_here(root, path))?;
        }
    }
    if started.content_root_made {
        writeln!(
            out,
            "made {}/, the folder the pages go in",
            from_here(root, &started.content_root)
        )?;
    }
    if let Some(ejected) = &started.templates {
        write_added(out, root, ejected, false)?;
    }
    if started.pages > 0 {
        write_adopted(out, err, global, root, started)?;
    } else {
        writeln!(
            out,
            "Next, write a first page, {}, and run `ascribe check`",
            from_here(root, &format!("{}/index.md", started.content_root))
        )?;
    }
    writeln!(
        out,
        "Then, to publish the pages as a site with Astro, read {}",
        started.next
    )
}

/// What `init` says of a folder of pages it adopted: how many it found, how
/// many have no title and how to give them one, what `ascribe check` finds,
/// by code, and the commands to run next.
fn write_adopted(
    out: &mut dyn Write,
    err: &mut dyn Write,
    global: &Global,
    root: &Path,
    started: &InitAnswer,
) -> io::Result<()> {
    let pages = started.pages;
    writeln!(
        out,
        "found {} in {}/; with no page types declared in ascribe.toml, no page needs `type:`",
        pages_text(pages),
        from_here(root, &started.content_root)
    )?;
    let global = Global {
        config: Some(root.join(MODEL_FILE)),
        color: global.color,
    };
    let args = check_args();
    let outcome = match check::run_check(&FromDisk, &global, &args, &mut io::empty()) {
        Ok(outcome) => outcome,
        Err(stopped) => {
            // The project is there; only the check of it isn't.
            check::report_stopped(stopped, &args, false, &mut io::sink(), err);
            return Ok(());
        }
    };
    let (reported, files, _) = outcome.parts();
    let safe = |d: &ascribe_check::Diagnostic| {
        d.fixes
            .iter()
            .any(|f| f.applicability == Applicability::Safe)
    };
    // With no page types, `title` is the only field a page must have.
    let untitled: Vec<_> = reported
        .iter()
        .map(|r| &r.diagnostic)
        .filter(|d| d.slug == diagnostics::FRONTMATTER_MISSING_FIELD)
        .collect();
    if !untitled.is_empty() {
        let fixable = untitled.iter().filter(|d| safe(d)).count();
        let how = match fixable {
            0 if untitled.len() == 1 => "give it `title:` in its frontmatter".to_owned(),
            0 => "give each one `title:` in its frontmatter".to_owned(),
            1 if untitled.len() == 1 => {
                "`ascribe check --fix` takes the heading it opens with as its title".to_owned()
            }
            f if f == untitled.len() => {
                "`ascribe check --fix` takes the heading each one opens with as its title"
                    .to_owned()
            }
            f => format!(
                "`ascribe check --fix` takes the opening heading as the title of {} of them, and the rest need `title:` written in their frontmatter",
                f
            ),
        };
        let has = if untitled.len() == 1 { "has" } else { "have" };
        writeln!(
            out,
            "{} {has} no `title`; {how}",
            pages_text(untitled.len())
        )?;
    }
    if reported.is_empty() {
        return writeln!(out, "`ascribe check` finds no problems");
    }
    let counts = Counts::of_reported(reported);
    writeln!(out, "`ascribe check` finds {}:", text::counted(counts))?;
    let (codes, _) = tally(files, reported);
    text::write_codes(out, &codes)?;
    let fixes = reported.iter().filter(|r| safe(&r.diagnostic)).count();
    // The most common code that isn't advice, and that `--fix` doesn't fix.
    let left = codes.iter().find(|c| {
        c.severity != Severity::Advice
            && reported
                .iter()
                .any(|r| r.diagnostic.code == c.code && !safe(&r.diagnostic))
    });
    match (fixes, left) {
        (0, Some(left)) => writeln!(out, "Next, run `ascribe check --code {}`", left.code)?,
        (fixes, Some(left)) => writeln!(
            out,
            "Next, run `ascribe check --fix`, which fixes {fixes} of them, then `ascribe check --code {}`",
            left.code
        )?,
        (0, None) => {}
        (fixes, None) => writeln!(
            out,
            "Next, run `ascribe check --fix`, which fixes {fixes} of them"
        )?,
    }
    if counts.advice > 0 {
        writeln!(
            out,
            "To put the advice off, so the check reports only what's new: `ascribe baseline write`"
        )?;
    }
    Ok(())
}

/// `1 page`, or `3 pages`.
fn pages_text(n: usize) -> String {
    if n == 1 {
        "1 page".to_owned()
    } else {
        format!("{n} pages")
    }
}

/// The arguments of the `ascribe check` that `init` runs on the pages it
/// found: the whole project, every build, without the addresses' base.
fn check_args() -> check::Args {
    check::Args {
        paths: Vec::new(),
        stdin: false,
        path: None,
        build: Vec::new(),
        editor_build: false,
        code: Vec::new(),
        summary: false,
        format: check::Format::Text,
        limit: None,
        exit_code: None,
        deny_warnings: false,
        base: None,
        base_exact: false,
        addresses: check::Addresses::NotCompared,
        fix: false,
        guard: crate::guard::Options::default(),
        vale: false,
        no_baseline: false,
        vale_timeout: None,
        vale_runs: check::ValeRuns::Never,
    }
}
