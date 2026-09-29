//! `tessera build`: check the project, then write each build's outputs.
//!
//! The checks run first, for every build asked for, and print exactly what
//! `tessera check` prints (the same report code on the same diagnostics). A
//! build with errors writes nothing (SPEC §8.2: "a build MUST fail on
//! errors"). Otherwise each build is resolved once (phase 12) and each
//! output is written from that resolved tree (phase 18), into a staging
//! directory that replaces the previous output only on success
//! (`project-docs/contracts/output-layout.md`).

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Args as ClapArgs, ValueEnum};
use tessera_check::{LoadError, Project};
use tessera_emit::{
    EmitContext, EmitError, Emitter, JsonEmitter, OutputDir, PlainEmitter, StoreError, emit,
};
use tessera_model::Build;
use tessera_resolve::DefaultRouter;

use crate::cli::Global;
use crate::commands::check::Format;
use crate::commands::diagnose::diagnose;
use crate::context::{Failure, load_project, stdout_is_terminal, use_color};
use crate::exit;
use crate::report::{Counts, FileTable, json, text};

/// Arguments of `tessera build`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Build only this build (repeat for several). By default, every build
    /// in tessera.toml.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// Which outputs to write, separated by commas.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [Emit::Plain, Emit::Json],
        value_name = "OUTPUTS"
    )]
    pub emit: Vec<Emit>,

    /// How to show the checks' results: the same formats as `tessera check`.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// An output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Emit {
    /// Plain markdown: fully resolved CommonMark with no HTML.
    Plain,
    /// The resolved tree as JSON.
    Json,
    /// Markdown plus web components, for a site (not built yet).
    Site,
}

/// Runs the command. Exit codes: 0 on success, 1 when the checks found errors
/// (nothing is written), 2 when it couldn't do its work: a usage error, no
/// `tessera.toml`, a content model with errors, or an output that couldn't be
/// written.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = build(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn build(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let color = use_color(global, stdout_is_terminal());
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return report_failure(failure, args, color, out, err),
    };
    // The checks, for every build asked for, before anything is written.
    let (diagnostics, builds) = match diagnose(&project, &args.build) {
        Ok(found) => found,
        Err(message) => return fail(err, &message),
    };
    // Resolved Q119: what `--emit site` does before phase 20.
    if args.emit.contains(&Emit::Site) {
        return fail(
            err,
            "the site output isn't available yet; use --emit plain,json",
        );
    }
    let files = FileTable::of_project(&project);
    let checked = project.sources().len();
    let written = match args.format {
        Format::Text => text::write(out, &files, &diagnostics, checked, color),
        Format::Json => json::write(out, &files, &diagnostics, checked, None),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the report: {e}"));
    }
    if Counts::of(&diagnostics).errors > 0 {
        let _ = writeln!(err, "error: the build failed; nothing was written");
        return exit::PROBLEMS;
    }

    match write_outputs(&project, &builds, &args.emit, err) {
        Ok(()) => exit::OK,
        Err(message) => fail(err, &message),
    }
}

/// Resolves each build and writes its outputs. Errors are ready to print.
fn write_outputs(
    project: &Project,
    builds: &[&Build],
    emit_names: &[Emit],
    err: &mut dyn Write,
) -> Result<(), String> {
    let model = project.model();
    let resolved_project = tessera_resolve::Project::load(
        Arc::new(model.clone()),
        project.layout().clone(),
        project.file_system(),
    );
    let root: &Path = project.root();
    let output_dir = root.join(&model.project.output_dir);
    let output = OutputDir::lock(&output_dir).map_err(store_message)?;
    // Resolved Q119: routes come from the default router until phase 20
    // supplies the `astro` profile's.
    let router = DefaultRouter::from_consumer(&model.consumer);

    let plain = PlainEmitter;
    let json = JsonEmitter;
    let mut emitters: Vec<&dyn Emitter> = Vec::new();
    for e in emit_names {
        let emitter: &dyn Emitter = match e {
            Emit::Plain => &plain,
            Emit::Json => &json,
            Emit::Site => continue,
        };
        if !emitters.iter().any(|x| x.name() == emitter.name()) {
            emitters.push(emitter);
        }
    }

    let mut warned: Vec<String> = Vec::new();
    for build in builds {
        let resolved = resolved_project.resolve_build(build, &router);
        let cx = EmitContext::new(&resolved_project, root, build);
        for emitter in &emitters {
            for warning in emitter.warnings(&cx) {
                if !warned.contains(&warning) {
                    let _ = writeln!(err, "warning: {warning}");
                    warned.push(warning);
                }
            }
            let emission = emit(*emitter, &cx, &resolved).map_err(emit_message)?;
            let pages = emission
                .files
                .iter()
                .filter(|f| f.kind == tessera_emit::FileKind::Page)
                .count();
            let assets = emission.files.len().saturating_sub(pages);
            let replaced = output
                .replace(&build.name, emitter.name(), &emission.files)
                .map_err(store_message)?;
            let _ = writeln!(
                err,
                "built {}/{}: {} page{}, {} asset{}{}",
                build.name,
                emitter.name(),
                pages,
                plural(pages),
                assets,
                plural(assets),
                if replaced.removed > 0 {
                    format!(
                        ", removed {} stale file{}",
                        replaced.removed,
                        plural(replaced.removed)
                    )
                } else {
                    String::new()
                }
            );
        }
    }
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn store_message(e: StoreError) -> String {
    e.to_string()
}

fn emit_message(e: EmitError) -> String {
    e.to_string()
}

fn fail(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(err, "error: {message}");
    exit::FAILURE
}

/// Reports a project that couldn't be loaded, as `tessera check` does
/// (resolved Q58): a content model with errors is a configuration failure
/// (exit code 2), and its diagnostics are shown.
fn report_failure(
    failure: Failure,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let (message, files, diagnostics) = match failure {
        Failure::Config(message) => (message, FileTable::of_model(String::new()), Vec::new()),
        Failure::Load(LoadError::Model { text, diagnostics }) => (
            format!(
                "{} has errors, so nothing can be built",
                tessera_check::MODEL_FILE
            ),
            FileTable::of_model(text),
            diagnostics,
        ),
        Failure::Load(e @ LoadError::Read { .. }) => (
            e.to_string(),
            FileTable::of_model(String::new()),
            Vec::new(),
        ),
    };
    let _ = match args.format {
        Format::Text => {
            let _ = diagnostics
                .iter()
                .try_for_each(|d| text::write_diagnostic(out, &files, d, color));
            writeln!(err, "error: {message}")
        }
        Format::Json => json::write(out, &files, &diagnostics, 0, Some(&message)),
    };
    exit::FAILURE
}
