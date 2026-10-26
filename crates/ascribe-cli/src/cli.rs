//! The command line: options every subcommand shares, and the subcommands.
//!
//! To add a subcommand, write `commands/<name>.rs` with an `Args` type
//! (`clap::Args`) and a `run(&Global, Args) -> ExitCode`, declare it in
//! `commands/mod.rs`, and add one variant to [`Command`] and one arm to
//! [`Command::run`].

use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::builder::StyledStr;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

use crate::{commands, exit};

/// A line for the end of a command's help, linking to the docs' permalink
/// for it, which forwards to wherever its section is: `docs_link!("cli-check")`.
/// A command's permalink is `cli-` and its name, with a `-` for each space.
macro_rules! docs_link {
    ($name:literal) => {
        concat!("Documentation: ", ascribe_core::docs_permalink!($name))
    };
}

/// `ascribe_core::docs_site!`, for the test that checks it.
#[cfg(test)]
pub(crate) const DOCS_SITE: &str = ascribe_core::docs_site!();

/// `ascribe_core::docs_permalinks!`, for the test that checks it.
#[cfg(test)]
pub(crate) const DOCS_PERMALINKS: &str = ascribe_core::docs_permalinks!();

/// What `ascribe --help` shows before the commands and options: examples,
/// and where an agent reads what to do first, since agents read `--help`
/// before anything else.
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
  ascribe rename phrase:product brand    Rename a phrase, and every use of it
  ascribe resolve guides/install.md --build cloud
                                         A page as one build's readers see it

For agents: `ascribe agents --help` says how to check and fix each edit.";

/// The commands `--help` lists, under a heading for each kind of work, as
/// the command reference groups them. Every command is in one group, in the
/// order [`Command`] declares them.
pub(crate) const GROUPS: &[(&str, &[&str])] = &[
    (
        "Write and check",
        &[
            "init", "check", "fmt", "build", "preview", "diff", "drift", "report",
        ],
    ),
    (
        "Ask about the project",
        &[
            "explain", "model", "outline", "link", "refs", "search", "resolve", "rename",
        ],
    ),
    (
        "Keep records",
        &[
            "baseline",
            "versions",
            "permalinks",
            "addresses",
            "sources",
            "observe",
            "templates",
            "vale",
        ],
    ),
    ("Connect tools", &["agents", "mcp", "lsp"]),
];

/// The command line, with `--help` listing the commands in [`GROUPS`]:
/// `clap` puts every subcommand under one heading, so the groups are written
/// here, from each command's description, and `clap` lists the options.
pub fn command() -> clap::Command {
    let cli = Cli::command();
    let header = *cli.get_styles().get_header();
    let literal = *cli.get_styles().get_literal();
    let width = GROUPS
        .iter()
        .flat_map(|(_, names)| names.iter())
        .map(|name| name.len())
        .max()
        .unwrap_or(0);
    let mut before = StyledStr::new();
    let _ = write!(before, "{EXAMPLES}");
    for (heading, names) in GROUPS {
        let _ = write!(before, "\n\n{header}{heading}:{header:#}");
        for name in *names {
            let about = cli
                .find_subcommand(name)
                .and_then(clap::Command::get_about)
                .map(StyledStr::to_string)
                .unwrap_or_default();
            let pad = width - name.len();
            let _ = write!(before, "\n  {literal}{name}{literal:#}{:pad$}  {about}", "");
        }
    }
    let mut template = StyledStr::new();
    let _ = write!(
        template,
        "{{about-with-newline}}\n{{usage-heading}} {{usage}}\n\n{{before-help}}\
         {header}Options:{header:#}\n{{options}}{{after-help}}"
    );
    cli.before_help(before).help_template(template)
}

/// The command line. Its about, which `--help` opens with, is the crate's
/// description: the short form in `design/words.toml`. The subcommands are
/// listed by [`command`], in groups.
#[derive(Debug, Parser)]
#[command(
    name = "ascribe",
    version = env!("ASCRIBE_VERSION"),
    about,
    long_about = None,
    arg_required_else_help = true,
    after_help = docs_link!("cli-reference"),
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
    /// `lsp`, whose project comes from the editor, `init`, which starts
    /// one, or `observe`, which runs in a folder of code.
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

/// The subcommands, in the order of [`GROUPS`]. A description, the first
/// paragraph of a doc comment, is about twelve words at most; what else a
/// command needs to say is the paragraphs after it, which only its own
/// `--help` shows.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start a project: write its first ascribe.toml and its pages' folder.
    ///
    /// It writes ascribe.addresses too, so a page's move is recorded from
    /// the first commit. With --templates, take the page templates too.
    /// Writes nothing where there's an ascribe.toml at or above the folder.
    #[command(after_help = docs_link!("cli-init"))]
    Init(commands::init::Args),
    /// Check every source file for problems, without building anything.
    #[command(after_help = docs_link!("cli-check"))]
    Check(commands::check::Args),
    /// Rewrite Ascribe constructs into canonical form.
    #[command(after_help = docs_link!("cli-fmt"))]
    Fmt(commands::fmt::Args),
    /// Build the documentation set's outputs.
    #[command(after_help = docs_link!("cli-build"))]
    Build(commands::build::Args),
    /// Serve a build's pages as a site, rendered again as they change.
    ///
    /// Renders the pages into .ascribe/preview, serves them at 127.0.0.1,
    /// and renders them again when the project's files change.
    #[command(after_help = docs_link!("cli-preview"))]
    Preview(commands::preview::Args),
    /// Show what changed since a git revision, page by page.
    ///
    /// Compares a git revision with the working tree, page by page, as
    /// readers will see it.
    #[command(after_help = docs_link!("cli-diff"))]
    Diff(commands::diff::Args),
    /// List the pages whose code examples changed since a git revision.
    ///
    /// Compares a git revision with the working tree, and says whether the
    /// words around each changed example changed too.
    #[command(after_help = docs_link!("cli-drift"))]
    Drift(commands::drift::Args),
    /// Report what state the project is in.
    ///
    /// Problems, inventory, what each build leaves out, external links, and
    /// the published site's delivery to agents.
    #[command(after_help = docs_link!("cli-report"))]
    Report(commands::report::Args),
    /// Explain a diagnostic: what it means, and how to fix it.
    ///
    /// Shows an example of the problem, and of its fix.
    #[command(
        after_help = docs_link!("cli-explain"),
        override_usage = "ascribe explain [OPTIONS] <CODE>\n       ascribe explain [OPTIONS] --list"
    )]
    Explain(commands::explain::Args),
    /// Show what the content model allows.
    ///
    /// Page types and their frontmatter, dimensions, phrases, features,
    /// glossary terms, widgets, and builds.
    #[command(after_help = docs_link!("cli-model"))]
    Model(commands::model::Args),
    /// Show a page's title, type, and headings, with their ids and lines.
    #[command(after_help = docs_link!("cli-outline"))]
    Outline(commands::outline::Args),
    /// Say whether a link target exists, and the link to write.
    ///
    /// Looks the target up as seen from a page, and gives its title.
    #[command(after_help = docs_link!("cli-link"))]
    Link(commands::link::Args),
    /// Show where a page, heading, phrase, or other entry is used.
    ///
    /// Takes a page, a heading, a fragment, a phrase, a feature, or another
    /// content model entry.
    #[command(after_help = docs_link!("cli-refs"))]
    Refs(commands::refs::Args),
    /// Show where the pages say something, as their readers read it.
    ///
    /// Searches the words as each build publishes them, through phrases,
    /// each found at the line to edit.
    #[command(after_help = docs_link!("cli-search"))]
    Search(commands::search::Args),
    /// Write a page as one build's plain Markdown output.
    ///
    /// Resolves the page for the build first.
    #[command(after_help = docs_link!("cli-resolve"))]
    Resolve(commands::resolve::Args),
    /// Rename a phrase, value, heading id, or page, and every use.
    ///
    /// Renames a phrase, a dimension's value, a heading's id, or a page, and
    /// changes everything that uses it, as the editor's rename does.
    #[command(after_help = docs_link!("cli-rename"))]
    Rename(commands::rename::Args),
    /// Put existing advice off, so `ascribe check` reports only what's new.
    #[command(after_help = docs_link!("cli-baseline"))]
    Baseline(commands::baseline::Args),
    /// Show the release lines, compare two, and release one.
    #[command(after_help = docs_link!("cli-versions"))]
    Versions(commands::versions::Args),
    /// Keep the list of permalinks, and check names against it.
    ///
    /// The list is the project's ascribe.permalinks.
    #[command(after_help = docs_link!("cli-permalinks"))]
    Permalinks(commands::permalinks::Args),
    /// Record where published addresses went, and list the redirects.
    ///
    /// The record is ascribe.addresses: record a move or a retired address,
    /// and list the redirects each build writes.
    #[command(after_help = docs_link!("cli-addresses"))]
    Addresses(commands::addresses::Args),
    /// Copy code from sources in other repositories, and move their pins.
    #[command(after_help = docs_link!("cli-sources"))]
    Sources(commands::sources::Args),
    /// Test this folder's examples, and record what they passed with.
    ///
    /// Runs the test of the ascribe.examples.toml in this folder, and when it
    /// passes, writes down what it passed and where: an observation, which
    /// pages read to say what an example was tested with.
    ///
    /// Runs the config's commands as lines of shell (`sh`, or `cmd` on
    /// Windows), as you, with nothing added: only the config in the current
    /// folder, never one in a parent folder or another project.
    #[command(after_help = docs_link!("cli-observe"))]
    Observe(commands::observe::Args),
    /// Write a page template into the project, or list the templates.
    ///
    /// Writes the template, and its guide, and declares its type; or lists
    /// the templates Ascribe carries.
    #[command(after_help = docs_link!("cli-templates"))]
    Templates(commands::templates::Args),
    /// Set up Vale, which checks the prose.
    ///
    /// Writes a preset out as the project's own configuration.
    #[command(after_help = docs_link!("cli-vale"))]
    Vale(commands::vale::Args),
    /// Write the files agents read: the project's rules, and the skill.
    ///
    /// For agents: after each edit, run `ascribe check --format concise` and
    /// fix what it reports; `ascribe explain <CODE>` says how to fix a
    /// diagnostic, with an example. Before writing frontmatter, a directive's
    /// attributes, or a phrase, read `ascribe model` for what this project
    /// allows.
    #[command(after_help = docs_link!("cli-agents"))]
    Agents(commands::agents::Args),
    /// Run the MCP server: the question commands, as tools for agents.
    ///
    /// Speaks the Model Context Protocol over standard input and output.
    #[command(after_help = docs_link!("cli-mcp"))]
    Mcp(commands::mcp::Args),
    /// Run the language server, speaking LSP over standard input and output.
    #[command(after_help = docs_link!("cli-lsp"))]
    Lsp(commands::lsp::Args),
}

impl Command {
    fn run(self, global: &Global) -> ExitCode {
        match self {
            Command::Agents(args) => commands::agents::run(global, args),
            Command::Addresses(args) => commands::addresses::run(global, args),
            Command::Baseline(args) => commands::baseline::run(global, args),
            Command::Build(args) => commands::build::run(global, args),
            Command::Check(args) => commands::check::run(global, args),
            Command::Diff(args) => commands::diff::run(global, args),
            Command::Drift(args) => commands::drift::run(global, args),
            Command::Explain(args) => commands::explain::run(global, args),
            Command::Fmt(args) => commands::fmt::run(global, args),
            Command::Init(args) => commands::init::run(global, args),
            Command::Link(args) => commands::link::run(global, args),
            Command::Lsp(args) => commands::lsp::run(global, args),
            Command::Mcp(args) => commands::mcp::run(global, args),
            Command::Model(args) => commands::model::run(global, args),
            Command::Observe(args) => commands::observe::run(global, args),
            Command::Outline(args) => commands::outline::run(global, args),
            Command::Permalinks(args) => commands::permalinks::run(global, args),
            Command::Preview(args) => commands::preview::run(global, args),
            Command::Refs(args) => commands::refs::run(global, args),
            Command::Rename(args) => commands::rename::run(global, args),
            Command::Report(args) => commands::report::run(global, args),
            Command::Resolve(args) => commands::resolve::run(global, args),
            Command::Search(args) => commands::search::run(global, args),
            Command::Sources(args) => commands::sources::run(global, args),
            Command::Templates(args) => commands::templates::run(global, args),
            Command::Vale(args) => commands::vale::run(global, args),
            Command::Versions(args) => commands::versions::run(global, args),
        }
    }
}

/// Parses the arguments and runs the subcommand. A usage error prints its
/// message and exits with 2.
pub fn run(args: impl IntoIterator<Item = OsString>) -> ExitCode {
    let cli = match command()
        .try_get_matches_from(args)
        .and_then(|matches| Cli::from_arg_matches(&matches))
    {
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

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    /// `--help` opens with the crate's description, which
    /// `scripts/design/words.test.ts` holds to the short form in
    /// `design/words.toml`, and with nothing more.
    #[test]
    fn the_help_opens_with_the_crates_description() {
        let command = Cli::command();
        assert_eq!(
            command.get_about().map(ToString::to_string).as_deref(),
            Some(env!("CARGO_PKG_DESCRIPTION"))
        );
        assert!(command.get_long_about().is_none());
    }

    /// Every command in `--help` has a description of its own: none falls
    /// back to its `Args` struct's doc comment, and no two are the same
    /// (#857).
    #[test]
    fn every_command_has_its_own_description() {
        let command = Cli::command();
        let mut seen = std::collections::BTreeMap::new();
        for sub in command.get_subcommands() {
            let name = sub.get_name();
            let about = sub
                .get_about()
                .map(ToString::to_string)
                .unwrap_or_else(|| panic!("`ascribe {name}` has no description"));
            assert!(
                !about.starts_with("Arguments of"),
                "`ascribe {name}` took its Args struct's description: {about}"
            );
            if let Some(other) = seen.insert(about.clone(), name.to_string()) {
                panic!("`ascribe {name}` and `ascribe {other}` share the description: {about}");
            }
        }
    }

    /// `--help` lists every command once, under the heading of its kind of
    /// work, in the order `Command` declares them, and each description is
    /// short enough to read down the list (#957).
    #[test]
    fn the_help_lists_every_command_in_its_group() {
        let command = Cli::command();
        let declared: Vec<&str> = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();
        let grouped: Vec<&str> = super::GROUPS
            .iter()
            .flat_map(|(_, names)| names.iter().copied())
            .collect();
        assert_eq!(grouped, declared);
        for sub in command.get_subcommands() {
            let about = sub.get_about().map(ToString::to_string).unwrap_or_default();
            let words = about.split_whitespace().count();
            assert!(
                words <= 12,
                "`ascribe {}`'s description is {words} words; move the rest to a paragraph after it: {about}",
                sub.get_name()
            );
        }
        let help = super::command().render_long_help().to_string();
        for (heading, names) in super::GROUPS {
            let at = help.find(&format!("\n{heading}:\n")).unwrap();
            for name in *names {
                assert!(
                    help[at..].contains(&format!("\n  {name} ")),
                    "{name}\n{help}"
                );
            }
        }
        assert!(help.contains("\nOptions:\n      --config"), "{help}");
    }
}
