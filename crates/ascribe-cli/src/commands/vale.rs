//! `ascribe vale`: the project's Vale setup (`[checks.vale]`). `eject`
//! writes a preset out as a configuration the project owns.

use std::io::{self, Write};
use std::process::ExitCode;

use ascribe_report::prose::{Ejected, eject};
use clap::{Args as ClapArgs, Subcommand};
use serde::Serialize;

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::{Failure, locate};
use crate::{exit, guard};

/// Arguments of `ascribe vale`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe vale`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write the preset `[checks.vale]` names out as the project's own
    /// `.vale.ini` and `.vale/` styles folder, with the rules `off` names
    /// turned off, and switch `[checks.vale]` to `config = ".vale.ini"`.
    ///
    /// Writes nothing when `.vale.ini` or `.vale` is there already.
    #[command(after_help = docs_link!("cli-vale-eject"))]
    Eject(EjectArgs),
}

/// Arguments of `ascribe vale eject`.
#[derive(Debug, ClapArgs)]
pub struct EjectArgs {
    /// `--dry-run` and `--force`, for `ascribe.toml`, which eject changes.
    #[command(flatten)]
    pub guard: guard::Options,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// The version of `eject --format json`'s schema.
const EJECT_SCHEMA_VERSION: u32 = 1;

/// What `ascribe vale eject --format json` writes: the preset written out,
/// and the files written.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct EjectJson<'a> {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The preset written out.
    preset: &'a str,
    /// The files written, from the project's folder, `/`-separated, in the
    /// order they were written: `ascribe.toml` last. With `--dry-run`, the
    /// files it would write.
    written: &'a [String],
    /// Whether it ran with `--dry-run`, and wrote nothing.
    dry_run: bool,
}

/// Runs the command. Exit codes: 0 when the preset was written out, 2 when
/// it wasn't.
pub fn run(global: &Global, args: Args) -> ExitCode {
    match args.command {
        Command::Eject(args) => run_eject(global, &args),
    }
}

fn run_eject(global: &Global, args: &EjectArgs) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();
    let loaded = locate(global).map_err(Failure::Config).and_then(|config| {
        ascribe_check::LOADER
            .load_model(&config)
            .map_err(Failure::Load)
    });
    let file = match loaded {
        Ok(file) => file,
        Err(e) => return exit::code(exit::fail(&mut err, &e)),
    };
    // A dry run first: what it would write, which the guard is asked about.
    let mut ejected = match eject(&file.root, &file.model, &file.text, true) {
        Ok(ejected) => ejected,
        Err(e) => return exit::code(exit::fail(&mut err, &e)),
    };
    let paths: Vec<_> = ejected.written.iter().map(|p| file.root.join(p)).collect();
    if let Err(e) = args.guard.allow(&paths) {
        return exit::code(exit::fail(&mut err, &e));
    }
    if !args.guard.dry_run {
        ejected = match eject(&file.root, &file.model, &file.text, false) {
            Ok(ejected) => ejected,
            Err(e) => return exit::code(exit::fail(&mut err, &e)),
        };
    }
    let result = match args.format {
        Format::Text => write_eject(&mut out, &ejected, args.guard.dry_run),
        Format::Json => answer::write_json(
            &mut out,
            &EjectJson {
                schema_version: EJECT_SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                preset: &ejected.preset,
                written: &ejected.written,
                dry_run: args.guard.dry_run,
            },
        ),
    };
    exit::code(answer::written(result, &mut err).unwrap_or(exit::OK))
}

fn write_eject(out: &mut dyn Write, ejected: &Ejected, dry_run: bool) -> io::Result<()> {
    if dry_run {
        for path in &ejected.written {
            writeln!(out, "would write {path}")?;
        }
        return Ok(());
    }
    for path in &ejected.written {
        writeln!(out, "wrote {path}")?;
    }
    writeln!(
        out,
        "The `{}` preset is now the project's own: `[checks.vale]` reads .vale.ini.",
        ejected.preset
    )
}
