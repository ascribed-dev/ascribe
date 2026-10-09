//! The command line: options every subcommand shares, and the subcommands.
//!
//! To add a subcommand, write `commands/<name>.rs` with an `Args` type
//! (`clap::Args`) and a `run(&Global, Args) -> ExitCode`, declare it in
//! `commands/mod.rs`, and add one variant to [`Command`] and one arm to
//! [`Command::run`].

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::{commands, exit};

/// A line for the end of a command's help, linking to its page on the docs
/// site (`ascribe_core::docs_site!`): `docs_page!("reference/cli/#ascribe-check")`.
macro_rules! docs_page {
    ($path:literal) => {
        concat!("Documentation: ", ascribe_core::docs_site!(), "/", $path)
    };
}

/// `ascribe_core::docs_site!`, for the test that checks it.
#[cfg(test)]
pub(crate) const DOCS_SITE: &str = ascribe_core::docs_site!();

/// What `ascribe --help` shows before the commands and options: examples,
/// and the commands an agent needs first, since agents read `--help` before
/// anything else.
const EXAMPLES: &str = "\
Examples:
  ascribe check                          Check the project in this folder or above
  ascribe build                          Check it, then build every output
  ascribe explain ASC036                 What a diagnostic means, and how to fix it
  ascribe model                          What the content model allows
  ascribe outline guides/install.md      A page's headings, with the ids links use
  ascribe link keys.md --from guides/install.md
                                         Whether a link works, and what to write
  ascribe refs phrase:product            Where a phrase is used
  ascribe render guides/install.md --build cloud
                                         A page as one build's readers see it

For agents: after each edit, run `ascribe check --format concise` and fix
what it reports; `ascribe explain <CODE>` says how to fix a diagnostic, with
an example. Before writing frontmatter, a directive's attributes, or a
phrase, read `ascribe model` for what this project allows.";

/// Ascribe: check, build, format, and serve documentation written as code.
#[derive(Debug, Parser)]
#[command(
    name = "ascribe",
    version = env!("ASCRIBE_VERSION"),
    arg_required_else_help = true,
    help_template = "{about-with-newline}\n{usage-heading} {usage}\n\n{before-help}{all-args}{after-help}",
    before_help = EXAMPLES,
    after_help = docs_page!("reference/cli/"),
)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,

    #[command(subcommand)]
    pub command: Command,
}

/// Options every subcommand accepts, before or after its name.
#[derive(Debug, clap::Args)]
pub struct Global {
    /// The content model, `ascribe.toml`, or a directory that holds one.
    ///
    /// By default, the nearest `ascribe.toml` in the current directory or a
    /// parent, so the commands work from anywhere inside a project. Not for
    /// `lsp`, whose project comes from the editor.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// When to color text output.
    #[arg(long, global = true, value_enum, default_value_t = Color::Auto, value_name = "WHEN")]
    pub color: Color,
}

/// When to color output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Color {
    /// When writing to a terminal, unless `NO_COLOR` is set.
    Auto,
    /// Always.
    Always,
    /// Never.
    Never,
}

/// The subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build the documentation set's outputs.
    #[command(after_help = docs_page!("reference/cli/#ascribe-build"))]
    Build(commands::build::Args),
    /// Check every source file for problems, without building anything.
    #[command(after_help = docs_page!("reference/cli/#ascribe-check"))]
    Check(commands::check::Args),
    /// Show what changed between a git revision and the working tree, page
    /// by page, as readers will see it.
    #[command(after_help = docs_page!("reference/cli/#ascribe-diff"))]
    Diff(commands::diff::Args),
    /// List the pages whose code examples changed between a git revision and
    /// the working tree, and whether the words around them changed too.
    #[command(after_help = docs_page!("reference/cli/#ascribe-drift"))]
    Drift(commands::drift::Args),
    /// Explain a diagnostic: what it means, how to fix it, and an example.
    #[command(after_help = docs_page!("reference/cli/#ascribe-explain"))]
    Explain(commands::explain::Args),
    /// Rewrite Ascribe constructs into canonical form.
    #[command(after_help = docs_page!("reference/cli/#ascribe-fmt"))]
    Fmt(commands::fmt::Args),
    /// Say whether a link target exists as seen from a page, its title, and
    /// the link to write.
    #[command(after_help = docs_page!("reference/cli/#ascribe-link"))]
    Link(commands::link::Args),
    /// Run the language server, speaking LSP over standard input and output.
    #[command(after_help = docs_page!("reference/cli/#ascribe-lsp"))]
    Lsp(commands::lsp::Args),
    /// Show what the content model allows: page types and their
    /// frontmatter, dimensions, phrases, features, glossary terms, widgets,
    /// and builds.
    #[command(after_help = docs_page!("reference/cli/#ascribe-model"))]
    Model(commands::model::Args),
    /// Show a page's title, type, and headings, with their ids and lines.
    #[command(after_help = docs_page!("reference/cli/#ascribe-outline"))]
    Outline(commands::outline::Args),
    /// Show where a page, a heading, a fragment, a phrase, a feature, or
    /// another content model entry is used.
    #[command(after_help = docs_page!("reference/cli/#ascribe-refs"))]
    Refs(commands::refs::Args),
    /// Show a page as a reader of one build sees it, as plain Markdown.
    #[command(after_help = docs_page!("reference/cli/#ascribe-render"))]
    Render(commands::render::Args),
    /// Copy code from sources in other repositories, and move their pins.
    #[command(after_help = docs_page!("reference/cli/#ascribe-sources"))]
    Sources(commands::sources::Args),
}

impl Command {
    fn run(self, global: &Global) -> ExitCode {
        match self {
            Command::Build(args) => commands::build::run(global, args),
            Command::Check(args) => commands::check::run(global, args),
            Command::Diff(args) => commands::diff::run(global, args),
            Command::Drift(args) => commands::drift::run(global, args),
            Command::Explain(args) => commands::explain::run(global, args),
            Command::Fmt(args) => commands::fmt::run(global, args),
            Command::Link(args) => commands::link::run(global, args),
            Command::Lsp(args) => commands::lsp::run(global, args),
            Command::Model(args) => commands::model::run(global, args),
            Command::Outline(args) => commands::outline::run(global, args),
            Command::Refs(args) => commands::refs::run(global, args),
            Command::Render(args) => commands::render::run(global, args),
            Command::Sources(args) => commands::sources::run(global, args),
        }
    }
}

/// Parses the arguments and runs the subcommand. A usage error prints its
/// message and exits with 2.
pub fn run(args: impl IntoIterator<Item = OsString>) -> ExitCode {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // `--help` and `--version` are successes; a mistake is a usage
            // error, exit code 2 (clap prints the message to the right stream).
            let _ = e.print();
            return exit::code(if e.use_stderr() {
                exit::FAILURE
            } else {
                exit::OK
            });
        }
    };
    cli.command.run(&cli.global)
}
